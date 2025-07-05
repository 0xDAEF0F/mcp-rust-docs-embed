use crate::chunks::{self, Chunk};
use anyhow::{Result, bail};
use std::collections::HashMap;
use tempfile::TempDir;
use tracing::info;
use url::Url;
use walkdir::WalkDir;

/// Process a GitHub repository URL and extract chunks from all Rust files
///
/// # Arguments
/// * `repo_url` - A GitHub URL (e.g., "https://github.com/owner/repo") or "owner/repo"
///   format
///
/// # Returns
/// A HashMap where keys are file paths and values are vectors of chunks
pub async fn process_github_repo(repo_url: &str) -> Result<HashMap<String, Vec<Chunk>>> {
	let repo_url = repo_url.to_string();

	// Run the blocking git clone operation in a separate thread
	let temp_dir = tokio::task::spawn_blocking(move || clone_repo(&repo_url))
		.await
		.map_err(|e| anyhow::anyhow!("Failed to spawn blocking task: {}", e))??;

	let mut file_chunks_map = HashMap::new();

	// Walk through all Rust and Markdown files in the repository
	for entry in WalkDir::new(temp_dir.path())
		.into_iter()
		.filter_map(Result::ok)
		.filter(|e| e.file_type().is_file())
		.filter(|e| {
			let file_extension = e.path().extension().and_then(|s| s.to_str());
			file_extension == Some("rs") || file_extension == Some("md")
		}) {
		let file_path = entry.path();
		let relative_path = file_path
			.strip_prefix(temp_dir.path())
			.unwrap_or(file_path)
			.to_string_lossy()
			.to_string();

		if let Ok(source) = std::fs::read_to_string(file_path) {
			let extension = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");

			// Process chunks in a blocking context to handle sync operations
			let chunks_result = match extension {
				"rs" => tokio::task::spawn_blocking(move || {
					chunks::rust::extract_rust_chunks(&source)
				})
				.await
				.map_err(|e| anyhow::anyhow!("Failed to spawn blocking task: {}", e))?,
				"md" => tokio::task::spawn_blocking(move || {
					chunks::markdown::extract_markdown_chunks(&source)
				})
				.await
				.map_err(|e| anyhow::anyhow!("Failed to spawn blocking task: {}", e))?,
				_ => continue,
			};

			if let Ok(chunks) = chunks_result
				&& !chunks.is_empty()
			{
				file_chunks_map.insert(relative_path, chunks);
			}
		}
	}

	Ok(file_chunks_map)
}

fn clone_repo(repo: &str) -> Result<TempDir> {
	let repo_url = parse_repo_url(repo)?;

	let mut builder = git2::build::RepoBuilder::new();

	let mut fetch_options = git2::FetchOptions::new();
	fetch_options.depth(1);

	builder.fetch_options(fetch_options);

	let temp_dir = TempDir::new()?;

	info!("Cloning repository: {repo_url}");

	builder.clone(repo_url.as_str(), temp_dir.path())?;

	info!("Cloned complete");

	Ok(temp_dir)
}

fn parse_repo_url(repo: &str) -> Result<Url> {
	match Url::parse(repo) {
		Ok(url) => Ok(url),
		_ if repo.split('/').count() == 2 => {
			let url = Url::parse(&format!("https://github.com/{repo}"))?;
			Ok(url)
		}
		_ => bail!("Invalid input: expected URL or owner/repo format"),
	}
}
