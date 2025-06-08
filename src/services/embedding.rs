use crate::{data_store::DataStore, query_embedder::QueryEmbedder, utils::find_md_files};
use anyhow::Result;
use embed_anything::embed_file;
use thin_logger::log;

pub struct EmbeddingService;

impl EmbeddingService {
	pub async fn embed_directory(crate_name: &str, version: &str) -> Result<()> {
		let directory = format!("docs/{crate_name}/{version}");

		if !std::path::Path::new(&directory).exists() {
			anyhow::bail!(
				"Documentation directory '{directory}' does not exist. Please run \
				 'GenDocs' first to generate documentation."
			);
		}

		log::info!("Starting embedding process for directory: {directory}");

		let data_store = DataStore::try_new(crate_name, version).await?;
		data_store.reset().await?;

		let query_embedder = QueryEmbedder::new()?;
		let embedder = query_embedder.get_embedder();
		let config = query_embedder.get_config();

		let files_to_embed = find_md_files(directory)?;
		log::info!("proceeding to embed {} files", files_to_embed.len());

		for file in files_to_embed {
			log::info!("embedding file: {file:?}");

			for embedding in embed_file(file, &embedder, Some(config), None)
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
