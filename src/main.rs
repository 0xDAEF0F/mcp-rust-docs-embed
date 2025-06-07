use anyhow::Result;
use clap::{Parser, Subcommand};
use embed_anything::embed_file;
use embed_anything_rs::{
	data_store::DataStore, doc_loader, query_embedder::QueryEmbedder,
	utils::find_md_files,
};
use htmd::{
	HtmlToMarkdown,
	options::{HeadingStyle, Options},
};
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
		/// Crate name
		crate_name: String,
		/// Crate version
		#[arg(long, short)]
		version: String,
	},
	/// Query for similar embeddings
	Query {
		/// Query string to search for
		query: String,
		/// Crate name to query for
		#[arg(long, short)]
		crate_name: String,
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

#[tokio::main]
async fn main() -> Result<()> {
	dotenvy::dotenv().ok();
	thin_logger::build(None).init();

	let cli = Cli::parse();

	match cli.command {
		Commands::Embed {
			crate_name,
			version,
		} => {
			embed_directory(&crate_name, &version).await?;
		}
		Commands::Query {
			query,
			crate_name,
			version,
			limit,
		} => {
			query_embeddings(&query, &crate_name, &version, limit).await?;
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

	let docs_dir = format!("docs/{crate_name}/{version}");
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

async fn embed_directory(crate_name: &str, version: &str) -> Result<()> {
	let directory = format!("docs/{crate_name}/{version}");
	
	// Check if docs directory exists
	if !std::path::Path::new(&directory).exists() {
		eprintln!("Documentation directory '{}' does not exist. Please run 'GenDocs' first to generate documentation.", directory);
		std::process::exit(1);
	}
	
	log::info!("Starting embedding process for directory: {directory}");

	let data_store = DataStore::try_new(crate_name, version).await?;

	// reset both the sqlite db and the qdrant collection
	data_store.reset().await?;

	let query_embedder = QueryEmbedder::new()?;
	let embedder = query_embedder.get_embedder();
	let config = query_embedder.get_config();

	let files_to_embed = find_md_files(directory)?;

	log::info!("proceeding to embed {} files", files_to_embed.len());

	for file in files_to_embed {
		log::info!("embedding file: {file:?}");

		for embedding in embed_file(file, &embedder, Some(config), None)
			.await?
			.expect("no data to embed?")
		{
			let contents = embedding.text.expect("expected text");
			let vec_e = embedding.embedding.to_dense()?;

			let row_id = data_store
				.add_embedding_with_content(&contents, vec_e)
				.await?;

			log::trace!("added embedding with id: {row_id}");
		}
	}

	log::info!("finished embedding all files");
	Ok(())
}

async fn query_embeddings(
	query: &str,
	crate_name: &str,
	version: &str,
	limit: u64,
) -> Result<()> {
	log::info!("querying for: {query}");

	let data_store = DataStore::try_new(crate_name, version).await?;

	let query_embedder = QueryEmbedder::new()?;
	let q_vec = query_embedder.embed_query(query).await?;

	let results = data_store.query_with_content(q_vec, limit).await?;

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
