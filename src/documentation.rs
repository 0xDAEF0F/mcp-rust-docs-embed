use crate::{data_store::DataStore, doc_loader};
use anyhow::{Context, Result};
use async_openai::{Client, config::OpenAIConfig, types::CreateEmbeddingRequestArgs};
use futures::stream::{self, StreamExt};
use tracing::{info, trace};

/// generates the `DocItem`s and creates embeddings for them
pub async fn generate_and_embed_docs(
	crate_name: &str,
	version: &str,
	features: &[String],
) -> Result<()> {
	info!(
		"Generating documentation for crate: {crate_name} (version: {version}) with \
		 features: [{}]",
		if features.is_empty() {
			"no features".to_string()
		} else {
			features.join(", ")
		}
	);

	// Generate docs and get both the documents and resolved version
	let (doc_items, resolved_version) =
		doc_loader::load_documents(crate_name, version, features)
			.context("Failed to load documents")?;

	info!("Loaded {} documentation items", doc_items.len());
	info!("Resolved version: {resolved_version}");

	// Create or reset data store
	let data_store = DataStore::try_new(crate_name, &resolved_version).await?;
	data_store.reset().await?;

	// Create chunks from doc items with actual source code
	let chunks: Vec<String> = doc_items.iter().map(|item| item.to_string()).collect();
	info!("Created {} chunks for embedding", chunks.len());

	// Embed chunks
	embed_chunks(&data_store, chunks).await?;

	info!("Documentation generation and embedding complete");

	Ok(())
}

async fn embed_chunks(data_store: &DataStore, chunks: Vec<String>) -> Result<()> {
	// Initialize OpenAI client
	let config = OpenAIConfig::new();
	let client = Client::with_config(config);

	// Process chunks in batches
	const BATCH_SIZE: usize = 50;
	const CONCURRENT_BATCHES: usize = 5;

	let batches: Vec<Vec<String>> = chunks
		.chunks(BATCH_SIZE)
		.map(|chunk| chunk.to_vec())
		.collect();

	let results = stream::iter(batches)
		.map(|batch| {
			let client = &client;
			async move {
				info!("Embedding batch of {} chunks", batch.len());

				let request = CreateEmbeddingRequestArgs::default()
					.model("text-embedding-3-small")
					.input(batch.clone())
					.build()?;

				let response = client
					.embeddings()
					.create(request)
					.await
					.context("Failed to create embeddings")?;

				// Pair each chunk with its embedding
				let mut batch_results = Vec::new();
				for (i, embedding_data) in response.data.into_iter().enumerate() {
					if let Some(chunk) = batch.get(i) {
						batch_results.push((chunk.clone(), embedding_data.embedding));
					}
				}

				Ok::<Vec<(String, Vec<f32>)>, anyhow::Error>(batch_results)
			}
		})
		.buffer_unordered(CONCURRENT_BATCHES)
		.collect::<Vec<_>>()
		.await;

	// Store all embeddings
	for result in results {
		let batch_results = result?;
		for (content, embedding) in batch_results {
			let row_id = data_store
				.add_embedding_with_content(&content, embedding)
				.await?;
			trace!("Added embedding with id: {row_id}");
		}
	}

	info!("Finished embedding all chunks");

	Ok(())
}
