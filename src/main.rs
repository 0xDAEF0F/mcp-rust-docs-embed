use anyhow::Result;
use clap::{Parser, Subcommand};
use embed_anything::{
	config::{SplittingStrategy, TextEmbedConfig},
	embed_file, embed_query,
	embeddings::{embed::Embedder, local::text_embedding::ONNXModel},
};
use embed_anything_rs::{data_store::DataStore, doc_loader, utils::find_md_files};
use htmd::{
	HtmlToMarkdown,
	options::{HeadingStyle, Options},
};
use std::sync::Arc;
use thin_logger::log;

#[derive(Parser)]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands {
	/// Create embeddings for all markdown files in a directory
	Embed {
		/// Directory path to search for markdown files
		directory: String,
		/// Database name (default: "test")
		#[arg(long, default_value = "test")]
		db_name: String,
		/// Collection name (default: "test")
		#[arg(long, default_value = "test")]
		collection: String,
	},
	/// Query for similar embeddings
	Query {
		/// Query string to search for
		query: String,
		/// Database name (default: "test")
		#[arg(long, default_value = "test")]
		db_name: String,
		/// Collection name (default: "test")
		#[arg(long, default_value = "test")]
		collection: String,
		/// Number of results to return (default: 5)
		#[arg(long, default_value = "5")]
		limit: u64,
	},
	/// Generate documentation for a crate
	GenDocs {
		/// Crate name to generate docs for
		crate_name: String,
		/// Optional features to enable
		#[arg(long)]
		features: Vec<String>,
		/// Crate version requirement (default: "*")
		#[arg(long, default_value = "*")]
		version: String,
	},
}

#[tokio::main]
async fn main() -> Result<()> {
	dotenvy::dotenv().ok();
	thin_logger::build(None).init();

	let cli = Cli::parse();

	match cli.command {
		Commands::Embed {
			directory,
			db_name,
			collection,
		} => {
			embed_directory(&directory, &db_name, &collection).await?;
		}
		Commands::Query {
			query,
			db_name,
			collection,
			limit,
		} => {
			query_embeddings(&query, &db_name, &collection, limit).await?;
		}
		Commands::GenDocs {
			crate_name,
			features,
			version,
		} => {
			gen_docs(&crate_name, &version, &features).await?;
		}
	}

	Ok(())
}

async fn gen_docs(crate_name: &str, version: &str, features: &[String]) -> Result<()> {
	log::info!("Generating documentation for crate: {crate_name} (version: {version})");

	let features_vec = features.to_vec();
	let features_option = if features_vec.is_empty() {
		None
	} else {
		Some(&features_vec)
	};

	let documents = doc_loader::load_documents(crate_name, version, features_option)
		.map_err(|e| anyhow::anyhow!("Failed to load documents: {}", e))?;

	log::info!("Loaded {} documents", documents.len());

	// Create HTML to markdown converter
	let converter = HtmlToMarkdown::builder()
		.skip_tags(vec!["script", "style", "meta", "head"])
		.options(Options {
			heading_style: HeadingStyle::Atx, // Use # for headings
			..Default::default()
		})
		.build();

	let docs_dir = format!("docs/{crate_name}");
	std::fs::create_dir_all(&docs_dir)?;

	for doc in documents {
		let safe_path = doc.path.replace(['/', '\\'], "_");
		let file_path = format!("{docs_dir}/{safe_path}.md");

		// Convert HTML to markdown
		let markdown_content = converter
			.convert(&doc.html_content)
			.map_err(|e| anyhow::anyhow!("Failed to convert HTML to markdown: {}", e))?;

		std::fs::write(&file_path, &markdown_content)?;
		log::info!("Saved documentation to: {file_path}");
	}

	log::info!("Documentation generation complete");
	Ok(())
}

async fn embed_directory(directory: &str, db_name: &str, collection: &str) -> Result<()> {
	log::info!("Starting embedding process for directory: {directory}");

	let data_store = DataStore::try_new(db_name, "test.db").await?;

	// reset both the sqlite db and the qdrant collection
	data_store.reset(db_name, collection).await?;

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

	let files_to_embed = find_md_files(directory)?;

	log::info!("proceeding to embed {} files", files_to_embed.len());

	for file in files_to_embed {
		log::info!("embedding file: {file:?}");

		for embedding in embed_file(file, &embedder, Some(&config), None)
			.await?
			.expect("no data to embed?")
		{
			let contents = embedding.text.expect("expected text");
			let vec_e = embedding.embedding.to_dense()?;

			let row_id = data_store
				.add_embedding_with_content(db_name, collection, &contents, vec_e)
				.await?;

			log::trace!("added embedding with id: {row_id}");
		}
	}

	log::info!("finished embedding all files");
	Ok(())
}

async fn query_embeddings(
	query: &str,
	db_name: &str,
	collection: &str,
	limit: u64,
) -> Result<()> {
	log::info!("querying for: {query}");

	let data_store = DataStore::try_new(db_name, "test.db").await?;

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

	let query_embeddings = embed_query(&[query], &embedder, Some(&config)).await?;

	if query_embeddings.is_empty() {
		anyhow::bail!("failed to generate query embedding");
	}

	let q_vec = query_embeddings[0].embedding.to_dense()?;

	let results = data_store
		.query_with_content(db_name, collection, q_vec, limit)
		.await?;

	if results.is_empty() {
		log::info!("no results found for query: {query}");
		return Ok(());
	}

	log::info!("found {} results for query: {}", results.len(), query);

	for (i, (score, content)) in results.iter().enumerate() {
		println!("\n--- Result {} (score: {:.4}) ---", i + 1, score);
		println!("{content}");
	}

	Ok(())
}
