use crate::chunks::{self, Chunk};
use anyhow::{Result, bail};
use std::collections::HashMap;
use tempfile::TempDir;
use tracing::info;
use url::Url;
use walkdir::WalkDir;

/// Processes a GitHub repository by cloning it and extracting semantic chunks from all Rust and
/// Markdown files.
///
/// # Arguments
/// * `repo_url` - The GitHub repository URL (e.g., "https://github.com/owner/repo") or shorthand
///   format ("owner/repo")
///
/// # Returns
/// A `HashMap` where:
/// - Keys are relative file paths within the repository (e.g., "src/main.rs", "docs/README.md")
/// - Values are vectors of `Chunk` structs containing semantic code segments from each file
///
/// Empty files or files that cannot be parsed are excluded from the result.
///
/// # Example
/// ```
/// let chunks = process_github_repo("rust-lang/rust").await?;
/// // chunks["src/main.rs"] contains all extracted chunks from that file
/// ```
pub async fn process_github_repo(repo_url: &str) -> Result<HashMap<String, Vec<Chunk>>> {
   // Clone repository in blocking context
   let temp_dir = tokio::task::spawn_blocking({
      let repo_url = repo_url.to_string();
      move || clone_repo(&repo_url)
   })
   .await??;

   let mut file_chunks_map = HashMap::new();

   // Walk through all Rust and Markdown files
   for entry in WalkDir::new(temp_dir.path())
      .into_iter()
      .filter_map(Result::ok)
      .filter(|e| {
         e.file_type().is_file()
            && e
               .path()
               .extension()
               .and_then(|s| s.to_str())
               .map(|ext| ext == "rs" || ext == "md")
               .unwrap_or(false)
      })
   {
      let file_path = entry.path();
      let relative_path = file_path
         .strip_prefix(temp_dir.path())
         .unwrap_or(file_path)
         .to_string_lossy()
         .to_string();

      if let Ok(source) = std::fs::read_to_string(file_path) {
         // Extract chunks based on file type
         let chunks = match file_path.extension().and_then(|s| s.to_str()) {
            Some("rs") => chunks::rust::extract_rust_chunks(&source)?,
            Some("md") => chunks::markdown::extract_markdown_chunks(&source)?,
            _ => continue,
         };

         if !chunks.is_empty() {
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
