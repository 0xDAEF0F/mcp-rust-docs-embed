#![allow(clippy::uninlined_format_args)]

use anyhow::Result;
use clap::Parser as _;
use embed_anything_rs::{
	commands::{Cli, Commands},
	config::AppConfig,
	services::{DocumentationService, EmbeddingService, QueryService},
};

#[tokio::main]
async fn main() -> Result<()> {
	// dotenvy::dotenv().ok(); // use this for production
	dotenvy::dotenv_override().ok();
	thin_logger::build(None).init();

	let config = AppConfig::from_env()?;
	let cli = Cli::parse();

	match cli.command {
		Commands::GenDocs {
			crate_name,
			features,
			version,
		} => {
			let doc_service = DocumentationService::new(config);
			doc_service
				.generate_docs(&crate_name, &version, &features)
				.await?;
		}
		Commands::Embed {
			crate_name,
			version,
		} => {
			let embedding_service = EmbeddingService::new(config);
			embedding_service
				.embed_directory(&crate_name, &version)
				.await?;
		}
		Commands::Query {
			query,
			crate_name,
			version,
			limit,
		} => {
			let query_service = QueryService::new(config);
			let results = query_service
				.query_embeddings(&query, &crate_name, &version, limit)
				.await?;
			QueryService::print_results(&results);
		}
	}

	Ok(())
}
