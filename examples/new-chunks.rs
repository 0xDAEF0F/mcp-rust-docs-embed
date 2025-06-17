use anyhow::Result;
use repo_glob::RepoGlob;
use text_splitter::{ChunkConfig, CodeSplitter};
use tiktoken_rs::cl100k_base;

#[tokio::main]
async fn main() -> Result<()> {
	// clone a repository
	let repo_glob = RepoGlob::clone("dtolnay/anyhow")?;

	// list rust files from the repository
	let rust_files = repo_glob.find_files(&["**/*.rs", "**/*.md"])?;

	// setup tokenizer and splitter
	let tokenizer = cl100k_base()?;
	let splitter = CodeSplitter::new(
		tree_sitter_rust::LANGUAGE,
		ChunkConfig::new(3000..7500).with_sizer(tokenizer),
	)?;

	// process first few rust files as example
	for (i, file_path) in rust_files.iter().enumerate() {
		println!("\nProcessing file {}: {:?}", i + 1, file_path.file_name());

		let content = repo_glob.read_file_content(file_path)?;

		let chunks: Vec<&str> = splitter.chunks(&content).collect();

		println!("  Found {} chunks", chunks.len());

		// if let Some(first_chunk) = chunks.first() {
		// 	let preview = if first_chunk.len() > 100 {
		// 		&first_chunk[..100]
		// 	} else {
		// 		first_chunk
		// 	};
		// 	println!("  First chunk preview: {}...", preview.trim());
		// }
	}

	Ok(())
}
