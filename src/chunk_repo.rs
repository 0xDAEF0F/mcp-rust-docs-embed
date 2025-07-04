use anyhow::{Result, bail};
use once_cell::sync::Lazy;
use std::{
	collections::{HashMap, HashSet},
	ops::RangeInclusive,
};
use tempfile::TempDir;
use tiktoken_rs::{CoreBPE, cl100k_base};
use tracing::{info, trace};
use tree_sitter::Node;
use url::Url;
use walkdir::WalkDir;

/// tresitter nodes to ignore
const NODES_TO_IGNORE: [&str; 1] = ["use_declaration"];

/// Maximum token limit for chunks
const MAX_TOKENS: usize = 8192;

/// Lazy-initialized BPE tokenizer to avoid repeated initialization
static BPE: Lazy<CoreBPE> =
	Lazy::new(|| cl100k_base().expect("Failed to initialize tiktoken BPE"));

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Chunk {
	pub kind: ChunkKind,
	pub start_line: usize,
	pub end_line: usize,
	pub content: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ChunkKind {
	Struct,
	Enum,
	Function,
	Impl,
	Comment,
}

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

	let mut result = HashMap::new();

	// Walk through all Rust files in the repository
	for entry in WalkDir::new(temp_dir.path())
		.into_iter()
		.filter_map(Result::ok)
		.filter(|e| e.file_type().is_file())
		.filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("rs"))
	{
		let file_path = entry.path();
		let relative_path = file_path
			.strip_prefix(temp_dir.path())
			.unwrap_or(file_path)
			.to_string_lossy()
			.to_string();

		if let Ok(source) = std::fs::read_to_string(file_path) {
			// Process chunks in a blocking context to handle sync operations
			let chunks_result =
				tokio::task::spawn_blocking(move || extract_chunks(&source))
					.await
					.map_err(|e| {
						anyhow::anyhow!("Failed to spawn blocking task: {}", e)
					})?;

			if let Ok(chunks) = chunks_result
				&& !chunks.is_empty()
			{
				result.insert(relative_path, chunks);
			}
		}
	}

	Ok(result)
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

fn extract_chunks(source: &str) -> Result<Vec<Chunk>> {
	let start = std::time::Instant::now();
	trace!(
		"Starting chunk extraction for {} chars of source",
		source.len()
	);

	let mut parser = tree_sitter::Parser::new();
	let language = tree_sitter_rust::LANGUAGE.into();
	parser.set_language(&language)?;

	let tree = parser
		.parse(source, None)
		.ok_or_else(|| anyhow::anyhow!("Failed to parse Rust source"))?;
	let root_node = tree.root_node();

	let mut chunks = Vec::new();
	let mut cursor = root_node.walk();
	let mut processed_lines = HashSet::new();

	for child in root_node.children(&mut cursor) {
		if NODES_TO_IGNORE.contains(&child.kind()) {
			continue;
		}

		// Skip if this node has already been processed as part of another chunk
		if processed_lines.contains(&child.start_position().row) {
			continue;
		}

		if let Some(chunk) = process_node(&child, source, &mut processed_lines) {
			chunks.push(chunk);
		}
	}

	let elapsed = start.elapsed();
	trace!(
		"Chunk extraction completed in {:?} - produced {} chunks",
		elapsed,
		chunks.len()
	);

	Ok(chunks)
}

fn process_node(
	node: &Node,
	source: &str,
	processed_lines: &mut HashSet<usize>,
) -> Option<Chunk> {
	let mut start_line = node.start_position().row;
	let end_line = node.end_position().row;

	// Find the earliest adjacent comment/attribute before this node
	if let Some(prev_sibling) = node.prev_sibling()
		&& is_adjacent_decoration(&prev_sibling, node)
	{
		start_line = find_first_decoration(&prev_sibling);
	}

	// Determine chunk kind and handle special cases
	let kind = match node.kind() {
		"struct_item" => ChunkKind::Struct,
		"enum_item" => ChunkKind::Enum,
		"function_item" => ChunkKind::Function,
		"impl_item" => ChunkKind::Impl,
		"line_comment" => {
			return handle_comment(node, source, start_line, processed_lines);
		}
		_ => return None,
	};

	// Mark lines as processed and extract content
	mark_lines_processed(start_line..=end_line, processed_lines);
	let content = extract_lines(source, start_line..=end_line);
	let content = trim_to_token_limit(&content).unwrap_or_default();

	Some(Chunk {
		kind,
		start_line: start_line + 1,
		end_line: end_line + 1,
		content,
	})
}

fn is_adjacent_decoration(prev: &Node, next: &Node) -> bool {
	matches!(prev.kind(), "line_comment" | "attribute_item")
		&& prev.end_position().row + 1 >= next.start_position().row
}

fn find_first_decoration(node: &Node) -> usize {
	let mut start_line = node.start_position().row;
	let mut current = *node;

	while let Some(prev) = current.prev_sibling() {
		if is_adjacent_decoration(&prev, &current) {
			start_line = prev.start_position().row;
			current = prev;
		} else {
			break;
		}
	}

	start_line
}

fn handle_comment(
	node: &Node,
	source: &str,
	start_line: usize,
	processed_lines: &mut HashSet<usize>,
) -> Option<Chunk> {
	// Check if this comment precedes an item declaration
	if is_comment_before_item(node) {
		return None;
	}

	// Collect all consecutive standalone comments
	let end_line = find_last_consecutive_comment(node);

	mark_lines_processed(start_line..=end_line, processed_lines);
	let content = extract_lines(source, start_line..=end_line);
	let content = trim_to_token_limit(&content).unwrap_or_default();

	Some(Chunk {
		kind: ChunkKind::Comment,
		start_line: start_line + 1,
		end_line: end_line + 1,
		content,
	})
}

fn is_comment_before_item(node: &Node) -> bool {
	let mut check_node = *node;

	// Look ahead through comments and attributes to find an item
	while let Some(next) = check_node.next_sibling() {
		match next.kind() {
			"struct_item" | "enum_item" | "function_item" | "impl_item" => {
				// Found an item - check if adjacent
				return check_node.end_position().row + 1 >= next.start_position().row;
			}
			"line_comment" | "attribute_item"
				if next.start_position().row <= check_node.end_position().row + 1 =>
			{
				// Continue through adjacent decorations
				check_node = next;
			}
			_ => break,
		}
	}

	false
}

fn find_last_consecutive_comment(node: &Node) -> usize {
	let mut end_line = node.end_position().row;
	let mut current = *node;

	while let Some(next) = current.next_sibling() {
		if next.kind() == "line_comment"
			&& next.start_position().row <= current.end_position().row + 1
		{
			end_line = next.end_position().row;
			current = next;
		} else {
			break;
		}
	}

	end_line
}

fn mark_lines_processed(
	range: RangeInclusive<usize>,
	processed_lines: &mut HashSet<usize>,
) {
	range.for_each(|line| {
		processed_lines.insert(line);
	});
}

fn extract_lines(source: &str, range: RangeInclusive<usize>) -> String {
	source
		.lines()
		.skip(*range.start())
		.take(range.end() - range.start() + 1)
		.collect::<Vec<_>>()
		.join("\n")
}

fn trim_to_token_limit(content: &str) -> Result<String> {
	let start = std::time::Instant::now();
	let tokens = BPE.encode_with_special_tokens(content);
	let encode_time = start.elapsed();

	trace!(
		"Token encoding took {:?} for {} chars -> {} tokens",
		encode_time,
		content.len(),
		tokens.len()
	);

	if tokens.len() <= MAX_TOKENS {
		return Ok(content.to_string());
	}

	// Trim to MAX_TOKENS
	let trimmed_tokens = &tokens[..MAX_TOKENS];
	let decode_start = std::time::Instant::now();
	let trimmed_content = BPE.decode(trimmed_tokens.to_vec())?;
	let decode_time = decode_start.elapsed();

	trace!(
		"Token decoding took {:?} for {} tokens -> {} chars",
		decode_time,
		trimmed_tokens.len(),
		trimmed_content.len()
	);

	Ok(trimmed_content)
}
