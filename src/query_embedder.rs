use crate::{data_store::DataStore, utils::find_md_files};
use anyhow::Result;
use embed_anything::{
	config::{SplittingStrategy, TextEmbedConfig},
	embed_file, embed_query,
	embeddings::{embed::Embedder, local::text_embedding::ONNXModel},
};
use std::{path::Path, sync::Arc};
use thin_logger::log;

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

		for file in files_to_embed {
			log::info!("embedding file: {file:?}");

			for embedding in embed_file(file, &self.embedder, Some(&self.config), None)
				.await?
				.expect("no data to embed?")
			{
				let contents = embedding.text.expect("expected text");
				let vec_e = embedding.embedding.to_dense()?;

				let row_id = data_store
					.add_embedding_with_content(&contents, vec_e)
					.await?;

				log::trace!("added embedding with id: {row_id}");
			}
		}

		log::info!("finished embedding all files");

		Ok(())
	}
}
