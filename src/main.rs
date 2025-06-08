#![allow(clippy::uninlined_format_args)]

use anyhow::Result;
use clap::Parser as _;
use embed_anything_rs::{
	commands::{Cli, Commands},
	services::{DocumentationService, EmbeddingService, QueryService},
};

#[tokio::main]
async fn main() -> Result<()> {
	// dotenvy::dotenv().ok(); // use this for production
	dotenvy::dotenv_override().ok();
	thin_logger::build(None).init();

	let cli = Cli::parse();

	match cli.command {
		Commands::GenDocs {
			crate_name,
			features,
			version,
		} => {
			DocumentationService::generate_docs(&crate_name, &version, &features).await?;
		}
		Commands::Embed {
			crate_name,
			version,
		} => {
			EmbeddingService::embed_crate(&crate_name, &version).await?;
		}
		Commands::Query {
			query,
			crate_name,
			version,
			limit,
		} => {
			let results =
				QueryService::query_embeddings(&query, &crate_name, &version, limit)
					.await?;
			QueryService::print_results(&results);
		}
	}

	Ok(())
}
