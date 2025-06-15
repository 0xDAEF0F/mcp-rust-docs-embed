use crate::{data_store::DataStore, utils::find_md_files};
use anyhow::{Context, Result};
use embed_anything::{
	config::TextEmbedConfig, embed_file, embed_query, embeddings::embed::Embedder,
};
use futures::stream::{self, StreamExt};
use std::{path::Path, sync::Arc};
use thin_logger::log;

pub struct QueryService {
	embedder: Arc<Embedder>,
	config: Arc<TextEmbedConfig>,
}

impl QueryService {
	pub fn new() -> Result<Self> {
		let openai_key =
			dotenvy::var("OPENAI_API_KEY").context("OPENAI_API_KEY not set")?;

		let embedder = Arc::new(Embedder::from_pretrained_cloud(
			"OpenAI",
			"text-embedding-3-small",
			Some(openai_key),
		)?);

		let config = Arc::new(
			TextEmbedConfig::default()
				.with_chunk_size(1000, Some(0.0))
				.with_batch_size(32),
		);

		Ok(Self { embedder, config })
	}

	pub async fn query_embeddings(
		&self,
		query: &str,
		crate_name: &str,
		version: &str,
		limit: u64,
	) -> Result<Vec<(f32, String)>> {
		log::info!("querying for: {query}");

		let data_store = DataStore::try_new(crate_name, version).await?;
		let q_vec = self.embed_query(query).await?;

		let results = data_store.query_with_content(q_vec, limit).await?;

		if results.is_empty() {
			log::info!("no results found for query: {query}");
			return Ok(vec![]);
		}

		log::info!("found {} results for query: {}", results.len(), query);
		Ok(results)
	}

	pub async fn embed_query(&self, query: &str) -> Result<Vec<f32>> {
		let query_embeddings =
			embed_query(&[query], &self.embedder, Some(&*self.config)).await?;

		anyhow::ensure!(
			!query_embeddings.is_empty(),
			"failed to generate query embedding"
		);

		let q_vec = query_embeddings[0].embedding.to_dense()?;

		Ok(q_vec)
	}

	pub async fn embed_crate(&self, crate_name: &str, version: &str) -> Result<()> {
		let directory = format!("docs/{crate_name}/{version}");

		anyhow::ensure!(
			Path::new(&directory).exists(),
			"Documentation directory '{directory}' does not exist. Please run 'GenDocs' \
			 first to generate documentation."
		);

		log::info!("Starting embedding process for directory: {directory}");

		let data_store = DataStore::try_new(crate_name, version).await?;
		data_store.reset().await?;

		let files_to_embed = find_md_files(directory)?;
		log::info!("proceeding to embed {} files", files_to_embed.len());

		const CONCURRENT_FILES: usize = 10;

		let results = stream::iter(files_to_embed)
			.map(|file| {
				let embedder = self.embedder.clone();
				let config = self.config.clone();
				async move {
					log::info!("embedding file: {file:?}");
					embed_file(&file, &embedder, Some(&*config), None)
						.await
						.with_context(|| format!("failed to embed file: {file:?}"))
						.map(|embeddings| (file, embeddings))
				}
			})
			.buffer_unordered(CONCURRENT_FILES)
			.collect::<Vec<_>>()
			.await;

		for result in results {
			let (_file, embeddings) = result?;
			let embeddings = embeddings.context("no data to embed")?;

			for embedding in embeddings {
				let contents = embedding.text.context("expected text")?;
				let embedding = embedding.embedding.to_dense()?;
				let row_id = data_store
					.add_embedding_with_content(&contents, embedding)
					.await?;

				log::trace!("added embedding with id: {row_id}");
			}
		}

		log::info!("finished embedding all files");

		Ok(())
	}

	pub fn print_results(results: &[(f32, String)]) {
		for (i, (score, content)) in results.iter().enumerate() {
			println!("\n--- Result {} (score: {:.4}) ---", i + 1, score);
			println!("{content}");
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_print_results_empty() {
		let results = vec![];
		QueryService::print_results(&results);
	}

	#[test]
	fn test_print_results_with_data() {
		let results = vec![
			(0.95, "Test content 1".to_string()),
			(0.85, "Test content 2".to_string()),
		];
		QueryService::print_results(&results);
	}
}
