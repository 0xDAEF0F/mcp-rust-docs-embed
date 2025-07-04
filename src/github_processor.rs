use crate::{chunk_repo::process_github_repo, data_store::DataStore};
use anyhow::{Context, Result};
use async_openai::{Client, config::OpenAIConfig, types::CreateEmbeddingRequestArgs};
use futures::stream::{self, StreamExt};
use tracing::{info, trace};

/// Processes a GitHub repository and embeds its documentation
pub async fn process_and_embed_github_repo(repo_url: &str) -> Result<()> {
	info!("Processing GitHub repository: {repo_url}");

	// Process the GitHub repository using chunker_rs
	let chunks_map = process_github_repo(repo_url)
		.await
		.context("Failed to process GitHub repository")?;

	// Flatten all chunks from all files into a single vector
	let chunks: Vec<_> = chunks_map
		.into_iter()
		.flat_map(|(_, file_chunks)| file_chunks)
		.collect();

	info!("Processed repository into {} chunks", chunks.len());

	// Create or reset data store for repository
	let data_store = DataStore::new(repo_url).await?;
	data_store.reset().await?;

	// Convert chunks to strings
	let chunk_strings: Vec<String> =
		chunks.into_iter().map(|chunk| chunk.content).collect();

	let doc_count = chunk_strings.len();
	info!("Created {} chunks for embedding", doc_count);

	// Embed chunks
	embed_chunks(&data_store, chunk_strings).await?;

	// Store metadata about this embedding
	data_store.store_metadata(doc_count).await?;

	info!("Repository processing and embedding complete with metadata");

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
