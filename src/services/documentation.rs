use crate::{
	data_store::DataStore,
	doc_loader,
	my_types::DocItem,
};
use anyhow::{Context, Result};
use async_openai::{Client, config::OpenAIConfig, types::CreateEmbeddingRequestArgs};
use futures::stream::{self, StreamExt};
use std::{fs, path::Path};
use thin_logger::log;

/// generates the `DocItem`s and creates embeddings for them
pub async fn generate_and_embed_docs(
	crate_name: &str,
	version: &str,
	features: &[String],
) -> Result<()> {
	log::info!(
		"Generating documentation for crate: {crate_name} (version: {version}) with \
		 features: [{}]",
		features.join(", ")
	);

	// Generate docs and get both the documents and resolved version
	let (doc_items, resolved_version, temp_dir) =
		doc_loader::load_documents(crate_name, version, features)
			.context("Failed to load documents")?;

	log::info!("Loaded {} documentation items", doc_items.len());
	log::info!("Resolved version: {resolved_version}");

	// Create or reset data store
	let data_store = DataStore::try_new(crate_name, &resolved_version).await?;
	data_store.reset().await?;

	// Create chunks from doc items with actual source code
	let chunks = create_source_code_chunks(&doc_items, temp_dir.path())?;
	log::info!("Created {} chunks for embedding", chunks.len());

	// Embed chunks
	embed_chunks(&data_store, chunks).await?;

	log::info!("Documentation generation and embedding complete");
	Ok(())
}

fn create_source_code_chunks(
	doc_items: &[DocItem],
	temp_dir: &Path,
) -> Result<Vec<String>> {
	let mut chunks = Vec::new();

	for item in doc_items {
		// Build the full path to the source file
		let source_path = temp_dir.join(&item.filename);

		// Read the source file
		let source_content = fs::read_to_string(&source_path).with_context(|| {
			format!("Failed to read source file: {}", source_path.display())
		})?;

		// Split into lines for easy access
		let lines: Vec<&str> = source_content.lines().collect();

		// Extract the relevant code using the line range
		// Line numbers in the JSON are 1-based
		let start_line = (item.file_range.start.0 as usize).saturating_sub(1);
		let end_line = (item.file_range.end.0 as usize).min(lines.len());

		if start_line >= lines.len() {
			log::warn!(
				"Invalid line range for {:?} in {}: start={}, total lines={}",
				item.name,
				item.filename,
				item.file_range.start.0,
				lines.len()
			);
			continue;
		}

		// Extract the code chunk
		let code_lines = &lines[start_line..end_line];
		let code_chunk = code_lines.join("\n");

		// Create chunk with doc string (if any) and source code
		let mut chunk = String::new();
		
		// Add documentation if available
		if let Some(doc_string) = &item.doc_string {
			chunk.push_str(doc_string);
			chunk.push_str("\n\n");
		}
		
		// Add the source code
		chunk.push_str("```rust\n");
		chunk.push_str(&code_chunk);
		chunk.push_str("\n```");

		chunks.push(chunk);
	}

	Ok(chunks)
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
				log::info!("Embedding batch of {} chunks", batch.len());

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
			log::trace!("Added embedding with id: {row_id}");
		}
	}

	log::info!("Finished embedding all chunks");
	Ok(())
}
