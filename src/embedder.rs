use anyhow::{Context, Result};
use async_openai::{
	Client,
	config::OpenAIConfig,
	types::{CreateEmbeddingRequest, EmbeddingInput},
};

pub struct Embedder {
	client: Client<OpenAIConfig>,
	model: String,
}

impl Embedder {
	pub fn new(api_key: String, model: String) -> Result<Self> {
		let config = OpenAIConfig::new().with_api_key(api_key);
		let client = Client::with_config(config);
		Ok(Self { client, model })
	}

	pub async fn embed_texts(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
		let request = CreateEmbeddingRequest {
			model: self.model.clone(),
			input: EmbeddingInput::StringArray(texts),
			encoding_format: None,
			user: None,
			dimensions: None,
		};

		let response = self
			.client
			.embeddings()
			.create(request)
			.await
			.context("failed to create embeddings")?;

		Ok(response.data.into_iter().map(|e| e.embedding).collect())
	}

	pub async fn embed_query(&self, query: &str) -> Result<Vec<f32>> {
		let embeddings = self.embed_texts(vec![query.to_string()]).await?;
		embeddings
			.into_iter()
			.next()
			.context("no embedding returned for query")
	}
}

#[derive(Clone)]
pub struct EmbedConfig {
	pub chunk_size: usize,
	pub chunk_overlap: usize,
}

impl Default for EmbedConfig {
	fn default() -> Self {
		Self {
			chunk_size: 3000,
			chunk_overlap: 200,
		}
	}
}
