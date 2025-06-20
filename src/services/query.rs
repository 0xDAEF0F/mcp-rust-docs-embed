use crate::data_store::DataStore;
use anyhow::{Context, Result};
use async_openai::{Client, config::OpenAIConfig, types::CreateEmbeddingRequestArgs};
use thin_logger::log;

pub struct QueryService {
	client: Client<OpenAIConfig>,
}

impl QueryService {
	pub fn new() -> Result<Self> {
		// Check for OpenAI API key
		dotenvy::var("OPENAI_API_KEY").context("OPENAI_API_KEY not set")?;

		let config = OpenAIConfig::new();
		let client = Client::with_config(config);

		Ok(Self { client })
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
		let request = CreateEmbeddingRequestArgs::default()
			.model("text-embedding-3-small")
			.input(vec![query])
			.build()?;

		let response = self
			.client
			.embeddings()
			.create(request)
			.await
			.context("Failed to create query embedding")?;

		anyhow::ensure!(
			!response.data.is_empty(),
			"failed to generate query embedding"
		);

		Ok(response.data[0].embedding.clone())
	}

	pub async fn embed_crate(&self, crate_name: &str, version: &str) -> Result<()> {
		// This functionality is now handled by generate_and_embed_docs in
		// documentation.rs
		use crate::services::generate_and_embed_docs;

		generate_and_embed_docs(crate_name, version, &[]).await?;

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
