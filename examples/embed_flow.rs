use anyhow::Result;
use embed_anything::{
	config::{SplittingStrategy, TextEmbedConfig},
	embed_file, embed_query,
	embeddings::{embed::Embedder, local::text_embedding::ONNXModel},
};
use embed_anything_rs::{data_store::DataStore, utils::find_md_files};
use std::sync::Arc;
use thin_logger::log;

#[tokio::main]
async fn main() -> Result<()> {
	dotenvy::dotenv().ok(); // load .env file
	thin_logger::build(None).init(); // init logging

	log::info!("Starting embedding process...");

	let data_store = DataStore::try_new("test", "test.db").await?;

	// reset both the sqlite db and the qdrant collection
	data_store.reset("test", "test").await?;

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

	let files_to_embed = find_md_files("tap-md")?;

	log::info!("proceeding to embed {} files", files_to_embed.len());

	for file in files_to_embed {
		for embedding in embed_file(file, &embedder, Some(&config), None)
			.await?
			.expect("no data to embed?")
		{
			let contents = embedding.text.expect("expected text");
			let vec_e = embedding.embedding.to_dense()?;

			let row_id = data_store
				.add_embedding_with_content("test", "test", &contents, vec_e)
				.await?;

			log::info!("added embedding with id: {row_id}");
		}
	}

	let query = embed_query(&["dot call"], &embedder, Some(&config)).await?;

	assert!(query.len() == 1, "expected 1 query");

	let q_vec = query[0].embedding.to_dense()?;

	let results = data_store
		.query_with_content("test", "test", q_vec, 1)
		.await?;

	assert!(results.len() == 1, "expected 1 result");

	let (score, content) = &results[0];

	log::info!("search result (score: {score}): {content}");

	Ok(())
}
