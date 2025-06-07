#![allow(unused)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use embed_anything_rs::{qdrant_w::QdrantW, utils::find_md_files};
use nestruct::flatten;

#[derive(Parser)]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands {
	/// Queries for an embedding
	Query { query: String },
}

fn main() -> Result<()> {
	let cli = Cli::parse();

	Ok(())
}
