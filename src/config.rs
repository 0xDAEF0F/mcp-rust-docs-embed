use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
	pub qdrant_url: String,
	pub openai_api_key: String,
}

#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
	pub vector_size: u64,
	pub chunk_size: usize,
	pub chunk_overlap: f32,
	pub batch_size: usize,
}

impl Default for EmbeddingConfig {
	fn default() -> Self {
		Self {
			vector_size: 1536, // openai text-embedding-3-small dimensions
			chunk_size: 1000,
			chunk_overlap: 0.0,
			batch_size: 32,
		}
	}
}
