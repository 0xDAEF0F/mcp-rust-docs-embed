#![allow(clippy::uninlined_format_args)]

use anyhow::Result;
use clap::Parser as _;
use embed_anything_rs::{
	commands::{Cli, Commands},
	services::{generate_md_docs, query::QueryService},
};

#[tokio::main]
async fn main() -> Result<()> {
	dotenvy::dotenv_override().ok();
	thin_logger::build(None).init();

	let cli = Cli::parse();

	match cli.command {
		Commands::GenDocs {
			crate_name,
			features,
			version,
		} => {
			generate_md_docs(&crate_name, &version, &features)?;
		}
		Commands::Embed {
			crate_name,
			version,
		} => {
			let query_service = QueryService::new()?;
			query_service.embed_crate(&crate_name, &version).await?;
		}
		Commands::Query {
			query,
			crate_name,
			version,
			limit,
		} => {
			let query_service = QueryService::new()?;
			let results = query_service
				.query_embeddings(&query, &crate_name, &version, limit)
				.await?;
			QueryService::print_results(&results);
		}
	}

	Ok(())
}
