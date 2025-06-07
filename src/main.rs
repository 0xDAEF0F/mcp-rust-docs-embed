// #![allow(unused, clippy::all)]
#![feature(let_chains, try_blocks)]

/*
	other useful crates:
	cargo add derive_more --features full # derive macros for more traits
	cargo add variantly # introspection for enum variants
	cargo add strum strum_macros # set of macros and traits for working with enums and strings
	cargo add nestruct # nested structs
*/

mod qdrant_w;
mod utils;

use crate::{qdrant_w::QdrantW, utils::find_md_files};
use anyhow::Result;
use embed_anything::{
	config::{SplittingStrategy, TextEmbedConfig},
	embed_file, embed_query,
	embeddings::{embed::Embedder, local::text_embedding::ONNXModel},
};
use qdrant_client::qdrant::point_id::PointIdOptions;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use thin_logger::log;

#[tokio::main]
async fn main() -> Result<()> {
	dotenvy::dotenv().ok(); // load .env file
	thin_logger::build(None).init(); // init logging

	log::info!("Starting embedding process...");

	let pool = SqlitePoolOptions::new().connect("test.db").await?;

	let qdrant_w = QdrantW::try_new("test").await?;

	// reset the sqlite db and the qdrant collection
	sqlx::query!("DELETE FROM test").execute(&pool).await?;

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
			let row_id =
				sqlx::query!("INSERT into test (contents) VALUES (?1)", contents)
					.execute(&pool)
					.await?
					.last_insert_rowid();
			log::info!("the id of the row is: {row_id}",);
			let vec_e = embedding.embedding.to_dense()?;
			qdrant_w
				.add_embedding("test", row_id.try_into()?, vec_e)
				.await?;
		}
	}

	let query = embed_query(&["dot call"], &embedder, Some(&config)).await?;

	assert!(query.len() == 1, "expected 1 query");

	let q_vec = query[0].embedding.to_dense()?;

	let res = qdrant_w.query_embedding("test", q_vec, 1).await?;

	assert!(res.result.len() == 1, "expected 1 result");

	let res_id = res.result[0]
		.clone()
		.id
		.expect("expected id")
		.point_id_options
		.expect("no point id");

	let PointIdOptions::Num(n) = res_id else {
		panic!("expected num id");
	};
	let n = n as i64;

	let res_contents = sqlx::query!("SELECT contents FROM test WHERE id = ?", n)
		.fetch_one(&pool)
		.await?;

	log::info!("the result is: {res_contents:?}",);

	Ok(())
}
