#![allow(unused)]

use anyhow::Result;
use clap::{Parser, Subcommand};
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
