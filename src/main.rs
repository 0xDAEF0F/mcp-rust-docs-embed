// #![allow(unused, clippy::all)]
#![feature(let_chains, try_blocks)]

/*
	other useful crates:
	cargo add derive_more --features full # derive macros for more traits
	cargo add variantly # introspection for enum variants
	cargo add validator # validation library
	cargo add bon # builder pattern
	cargo add strum strum_macros # set of macros and traits for working with enums and strings
	cargo add nestruct # nested structs
	cargo add reqwest # http client
	cargo add itertools # iterators
*/

mod qdrant_w;
mod utils;

use crate::{qdrant_w::QdrantW, utils::find_md_files};
use anyhow::Result;
use embed_anything::{
	config::{SplittingStrategy, TextEmbedConfig},
	embed_file,
	embeddings::{embed::Embedder, local::text_embedding::ONNXModel},
};
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use thin_logger::log;

#[tokio::main]
async fn main() -> Result<()> {
	dotenvy::dotenv().ok(); // load .env file
	thin_logger::build(None).init(); // init logging

	log::info!("Starting embedding process...");

	let pool = SqlitePoolOptions::new().connect("test.db").await?;

	let qdrant_w = QdrantW::try_new().await?;

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
			qdrant_w.add_embedding(row_id as u64, vec_e).await?;
		}
	}

	log::info!("Embedding process completed!");

	Ok(())
}
