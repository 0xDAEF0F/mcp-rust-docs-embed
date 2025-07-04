use crate::data_store::DataStore;
use anyhow::{Context, Result};
use async_openai::{Client, config::OpenAIConfig, types::CreateEmbeddingRequestArgs};
use tracing::info;

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
		repo_url: &str,
		limit: u64,
	) -> Result<Vec<(f32, String)>> {
		info!("querying for: {query} in repository: {repo_url}");

		let data_store = DataStore::new(repo_url).await?;
		let q_vec = self.embed_query(query).await?;

		let results = data_store.query_with_content(q_vec, limit).await?;

		if results.is_empty() {
			info!("no results found for query: {query}");
			return Ok(vec![]);
		}

		info!("found {} results for query: {}", results.len(), query);
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
}
