#![allow(unused, clippy::all)]
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

use anyhow::Result;
use embed_anything::{
	config::{SplittingStrategy, TextEmbedConfig},
	embed_file,
	embeddings::{
		embed::{Embedder, EmbedderBuilder},
		local::text_embedding::ONNXModel,
	},
};
use std::sync::Arc;
use thin_logger::log;

#[tokio::main]
async fn main() -> Result<()> {
	dotenvy::dotenv().ok(); // load .env file
	thin_logger::build(None).init(); // init logging

	log::info!("Starting embedding process...");

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

	let Some(embed_data_vec) =
		embed_file("tap-md/index.md", &embedder, Some(&config), None).await?
	else {
		log::warn!("No embed data found for file: tap-md/index.md");
		return Ok(());
	};

	for embed_data in &embed_data_vec {
		if let Some(text) = &embed_data.text {
			println!("{}", text);
			println!("{}", "---".repeat(20));
		}
	}

	log::info!("Embedding process completed!");

	Ok(())
}
