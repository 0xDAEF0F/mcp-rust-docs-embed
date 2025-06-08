use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
	pub qdrant_url: String,
	pub sqlite_url: String,
}

impl AppConfig {
	pub fn from_env() -> Result<Self> {
		Ok(envy::from_env::<Self>()?)
	}
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
			vector_size: 1024,
			chunk_size: 1000,
			chunk_overlap: 0.0,
			batch_size: 32,
		}
	}
}
