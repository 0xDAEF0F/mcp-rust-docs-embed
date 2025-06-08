use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
	#[command(subcommand)]
	pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
	/// Create embeddings for all markdown files in a directory
	Embed {
		/// Crate name
		crate_name: String,
		/// Crate version
		#[arg(long, short)]
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
		#[arg(long, short)]
		version: String,
		/// Number of results to return (default: 5)
		#[arg(long, short, default_value = "5")]
		limit: u64,
	},
	/// Generate documentation for a crate
	GenDocs {
		/// Crate name to generate docs for
		crate_name: String,
		/// Optional features to enable
		#[arg(long, short)]
		features: Vec<String>,
		/// Crate version requirement
		#[arg(long, short)]
		version: String,
	},
}
