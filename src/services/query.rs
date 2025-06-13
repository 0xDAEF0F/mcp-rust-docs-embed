use crate::{config::AppConfig, data_store::DataStore, query_embedder::QueryEmbedder};
use anyhow::Result;
use thin_logger::log;

pub struct QueryService;

impl QueryService {
	pub async fn query_embeddings(
		query: &str,
		crate_name: &str,
		version: &str,
		limit: u64,
	) -> Result<Vec<(f32, String)>> {
		log::info!("querying for: {query}");

		let data_store = DataStore::try_new(crate_name, version).await?;
		let config = envy::from_env::<AppConfig>()?;
		let query_embedder = QueryEmbedder::new(config.openai_api_key)?;
		let q_vec = query_embedder.embed_query(query).await?;

		let results = data_store.query_with_content(q_vec, limit).await?;

		if results.is_empty() {
			log::info!("no results found for query: {query}");
			return Ok(vec![]);
		}

		log::info!("found {} results for query: {}", results.len(), query);
		Ok(results)
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
