use anyhow::Result;
use embed_anything::{
	config::{SplittingStrategy, TextEmbedConfig},
	embed_query,
	embeddings::{embed::Embedder, local::text_embedding::ONNXModel},
};
use std::sync::Arc;

pub struct QueryEmbedder {
	embedder: Arc<Embedder>,
	config: TextEmbedConfig,
}

impl QueryEmbedder {
	pub fn new() -> Result<Self> {
		let embedder = Arc::new(Embedder::from_pretrained_onnx(
			"jina",
			Some(ONNXModel::JINAV3),
			None,
			None,
			None,
			None,
		)?);

		let config = TextEmbedConfig::default()
			.with_chunk_size(1000, Some(0.0))
			.with_batch_size(32)
			.with_splitting_strategy(SplittingStrategy::Semantic {
				semantic_encoder: embedder.clone(),
			});

		Ok(Self { embedder, config })
	}

	pub async fn embed_query(&self, query: &str) -> Result<Vec<f32>> {
		let query_embeddings =
			embed_query(&[query], &self.embedder, Some(&self.config)).await?;

		if query_embeddings.is_empty() {
			anyhow::bail!("failed to generate query embedding");
		}

		let q_vec = query_embeddings[0].embedding.to_dense()?;
		Ok(q_vec)
	}

	pub fn get_embedder(&self) -> Arc<Embedder> {
		self.embedder.clone()
	}

	pub fn get_config(&self) -> &TextEmbedConfig {
		&self.config
	}
}
