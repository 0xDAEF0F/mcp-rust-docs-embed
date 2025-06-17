use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
	#[command(subcommand)]
	pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
	/// Create embeddings for a crate by cloning its repository
	Embed {
		/// Crate name (e.g. "dtolnay/anyhow")
		crate_name: String,
		/// Crate version for storage purposes
		#[arg(long, short, default_value = "latest")]
		version: String,
	},
	/// Query for similar embeddings
	Query {
		/// Crate name to query for
		crate_name: String,
		/// Query string to search for
		#[arg(long, short)]
		query: String,
		/// Crate version
		#[arg(long, short, default_value = "latest")]
		version: String,
		/// Number of results to return (default: 10)
		#[arg(long, short, default_value = "10")]
		limit: u64,
	},
}
