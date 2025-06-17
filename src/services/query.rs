use crate::{
	data_store::DataStore,
	embedder::{EmbedConfig, Embedder},
};
use anyhow::{Context, Result};
use futures::stream::{self, StreamExt};
use repo_glob::RepoGlob;
use std::sync::Arc;
use text_splitter::{ChunkConfig, CodeSplitter, MarkdownSplitter};
use thin_logger::log;
use tiktoken_rs::cl100k_base;

pub struct QueryService {
	embedder: Arc<Embedder>,
	config: Arc<EmbedConfig>,
}

impl QueryService {
	pub fn new() -> Result<Self> {
		let openai_key =
			dotenvy::var("OPENAI_API_KEY").context("OPENAI_API_KEY not set")?;

		let embedder = Arc::new(Embedder::new(
			openai_key,
			"text-embedding-3-small".to_string(),
		)?);

		let config = Arc::new(EmbedConfig::default());

		Ok(Self { embedder, config })
	}

	pub async fn query_embeddings(
		&self,
		query: &str,
		crate_name: &str,
		version: &str,
		limit: u64,
	) -> Result<Vec<(f32, String)>> {
		log::info!("querying for: {query}");

		let data_store = DataStore::try_new(crate_name, version).await?;
		let q_vec = self.embed_query(query).await?;

		let results = data_store.query_with_content(q_vec, limit).await?;

		if results.is_empty() {
			log::info!("no results found for query: {query}");
			return Ok(vec![]);
		}

		log::info!("found {} results for query: {}", results.len(), query);
		Ok(results)
	}

	pub async fn embed_query(&self, query: &str) -> Result<Vec<f32>> {
		self.embedder.embed_query(query).await
	}

	pub async fn embed_crate(&self, repo_path: &str, version: &str) -> Result<()> {
		log::info!("cloning repository for {repo_path}");

		// repo_path is in format "owner/repo", convert to github url
		let github_url = format!("https://github.com/{}", repo_path);

		// clone the repository
		let repo_glob = RepoGlob::clone(&github_url)?;

		// find all rust and markdown files
		let files = repo_glob.find_files(&["**/*.rs", "**/*.md"])?;
		log::info!("found {} files to embed", files.len());

		// extract crate name from repo path (e.g., "tokio-rs/tokio" -> "tokio")
		let crate_name = repo_path.split('/').next_back().unwrap_or(repo_path);
		let data_store = DataStore::try_new(crate_name, version).await?;
		data_store.reset().await?;

		// setup tokenizer and splitters
		let tokenizer = cl100k_base()?;
		let rust_splitter = CodeSplitter::new(
			tree_sitter_rust::LANGUAGE,
			ChunkConfig::new(self.config.chunk_size..self.config.chunk_size + 2500)
				.with_sizer(tokenizer.clone()),
		)?;
		let md_splitter = MarkdownSplitter::new(
			ChunkConfig::new(self.config.chunk_size..self.config.chunk_size + 2500)
				.with_sizer(tokenizer),
		);

		const CONCURRENT_FILES: usize = 10;

		let results = stream::iter(files)
			.map(|file_path| {
				let repo_glob = &repo_glob;
				let rust_splitter = &rust_splitter;
				let md_splitter = &md_splitter;
				let embedder = self.embedder.clone();

				async move {
					log::info!("processing file: {file_path:?}");

					let content = repo_glob
						.read_file_content(&file_path)
						.with_context(|| format!("failed to read file: {file_path:?}"))?;

					let file_name =
						file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

					// choose splitter based on file extension
					let chunks: Vec<&str> = if file_name.ends_with(".rs") {
						rust_splitter.chunks(&content).collect()
					} else if file_name.ends_with(".md") {
						md_splitter.chunks(&content).collect()
					} else {
						vec![&content]
					};

					// prepare chunks with metadata
					let file_path_str = file_path.display().to_string();
					let total_chunks = chunks.len();
					let chunks_with_metadata: Vec<String> = chunks
						.into_iter()
						.enumerate()
						.map(|(i, chunk)| {
							format!(
								"File: {}\nChunk: {}/{}\n\n{}",
								file_path_str,
								i + 1,
								total_chunks,
								chunk
							)
						})
						.collect();

					// embed chunks
					if !chunks_with_metadata.is_empty() {
						let embeddings = embedder
							.embed_texts(chunks_with_metadata.clone())
							.await
							.with_context(|| {
								format!("failed to embed chunks from: {file_path:?}")
							})?;

						Ok::<Vec<(String, Vec<f32>)>, anyhow::Error>(
							chunks_with_metadata.into_iter().zip(embeddings).collect(),
						)
					} else {
						Ok(vec![])
					}
				}
			})
			.buffer_unordered(CONCURRENT_FILES)
			.collect::<Vec<_>>()
			.await;

		// store all embeddings
		for result in results {
			let chunk_embeddings = result?;

			for (content, embedding) in chunk_embeddings {
				let row_id = data_store
					.add_embedding_with_content(&content, embedding)
					.await?;

				log::trace!("added embedding with id: {row_id}");
			}
		}

		log::info!("finished embedding all files");
		Ok(())
	}

	pub fn print_results(results: &[(f32, String)]) {
		for (i, (score, content)) in results.iter().enumerate() {
			println!("\n--- Result {} (score: {:.4}) ---", i + 1, score);
			println!("{content}");
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_print_results_empty() {
		let results = vec![];
		QueryService::print_results(&results);
	}

	#[test]
	fn test_print_results_with_data() {
		let results = vec![
			(0.95, "Test content 1".to_string()),
			(0.85, "Test content 2".to_string()),
		];
		QueryService::print_results(&results);
	}
}
