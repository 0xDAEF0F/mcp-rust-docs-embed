use crate::doc_generator::DocGenerator;
use anyhow::Result;
use scraper::{Html, Selector};
use std::{collections::HashMap, fs, path::PathBuf};
use walkdir::WalkDir;

// Simple struct to hold document content, maybe add path later if needed
#[derive(Debug, Clone)]
pub struct Document {
	pub path: String,
	pub content: String,
	pub html_content: String,
}

/// Generates documentation for a given crate in a temporary directory,
/// then loads and parses the HTML documents.
/// Extracts text content from the main content area of rustdoc generated HTML.
/// Returns both the documents and the resolved version.
pub fn load_documents_with_version(
	crate_name: &str,
	crate_version_req: &str,
	features: Option<&Vec<String>>,
) -> Result<(Vec<Document>, String)> {

	// Generate documentation using the new DocGenerator
	let features_vec = features.cloned();
	let doc_generator = DocGenerator::new(crate_name, crate_version_req, features_vec)?;
	let docs_path = doc_generator.generate_docs()?;

	eprintln!("Using documentation path: {}", docs_path.display());

	// Extract version from the generated docs path or index.html
	let resolved_version = extract_version_from_docs(&docs_path, crate_name)?;

	// Define the CSS selector for the main content area in rustdoc HTML
	// This might need adjustment based on the exact rustdoc version/theme
	let content_selector = Selector::parse("section#main-content.content")
		.map_err(|e| anyhow::anyhow!("Failed to parse CSS selector: {}", e))?;

	// ... rest of the document loading logic ...
	let documents_result = load_documents_internal(&docs_path, &content_selector)?;
	
	Ok((documents_result, resolved_version))
}

fn extract_version_from_docs(docs_path: &PathBuf, crate_name: &str) -> Result<String> {
	// Try to find the temp directory containing the Cargo.lock
	let mut temp_dir = docs_path.clone();
	
	// Go up from docs_path (e.g., .../doc/bon) to find the temp directory root
	while temp_dir.parent().is_some() {
		temp_dir = temp_dir.parent().unwrap().to_path_buf();
		
		let cargo_lock = temp_dir.join("Cargo.lock");
		if cargo_lock.exists() {
			let content = std::fs::read_to_string(&cargo_lock)?;
			
			// Look for the crate in Cargo.lock
			// Format is like:
			// [[package]]
			// name = "bon"
			// version = "3.6.3"
			if let Some(package_start) = content.find(&format!("name = \"{}\"", crate_name)) {
				// Look for the version line after the name
				if let Some(version_start) = content[package_start..].find("version = \"") {
					let version_pos = package_start + version_start + "version = \"".len();
					if let Some(version_end) = content[version_pos..].find('"') {
						let version = &content[version_pos..version_pos + version_end];
						return Ok(version.to_string());
					}
				}
			}
		}
	}
	
	// Fallback: try to extract from directory name if docs_path contains version info
	if let Some(dir_name) = docs_path.file_name().and_then(|n| n.to_str()) {
		// If the directory name looks like a version (contains dots and numbers)
		if dir_name.contains('.') && dir_name.chars().any(|c| c.is_ascii_digit()) {
			return Ok(dir_name.to_string());
		}
	}
	
	Err(anyhow::anyhow!("Could not extract version from generated documentation"))
}

fn load_documents_internal(docs_path: &PathBuf, content_selector: &Selector) -> Result<Vec<Document>> {
	let mut documents = Vec::new();

	// --- Collect all HTML file paths first ---
	let all_html_paths: Vec<PathBuf> = WalkDir::new(docs_path)
		.into_iter()
		.filter_map(Result::ok) // Ignore errors during iteration
		.filter(|e| {
			!e.file_type().is_dir()
				&& e.path().extension().is_some_and(|ext| ext == "html")
		})
		.map(|e| e.into_path()) // Get the PathBuf
		.collect();

	eprintln!(
		"[DEBUG] Found {} total HTML files initially.",
		all_html_paths.len()
	);

	// --- Group files by basename ---
	let mut basename_groups: HashMap<String, Vec<PathBuf>> = HashMap::new();
	for path in all_html_paths {
		if let Some(filename_osstr) = path.file_name() {
			if let Some(filename_str) = filename_osstr.to_str() {
				basename_groups
					.entry(filename_str.to_string())
					.or_default()
					.push(path);
			} else {
				eprintln!(
					"[WARN] Skipping file with non-UTF8 name: {}",
					path.display()
				);
			}
		} else {
			eprintln!("[WARN] Skipping file with no name: {}", path.display());
		}
	}

	// --- Initialize paths_to_process and explicitly add the root index.html if it exists
	// ---
	let mut paths_to_process: Vec<PathBuf> = Vec::new();
	let root_index_path = docs_path.join("index.html");
	if root_index_path.is_file() {
		paths_to_process.push(root_index_path);
	}

	// --- Filter based on duplicates and size ---
	// NOTE: Initialization of paths_to_process moved before this loop
	for (basename, mut paths) in basename_groups {
		// Always ignore index.html at this stage (except the root one added earlier)
		if basename == "index.html" {
			continue;
		}

		// Also ignore files within source code view directories
		// Check the first path (they should share the problematic component if any)
		if paths
			.first()
			.is_some_and(|p| p.components().any(|comp| comp.as_os_str() == "src"))
		{
			continue;
		}

		if paths.len() == 1 {
			// Single file with this basename (and not index.html), keep it
			paths_to_process.push(paths.remove(0));
		} else {
			// Multiple files with the same basename (duplicates)
			// Find the largest one by file size
			// Explicit type annotation needed for the error type in try_fold
			let largest_path_result: Result<Option<(PathBuf, u64)>, std::io::Error> =
				paths
					.into_iter()
					.try_fold(None::<(PathBuf, u64)>, |largest, current| {
						let current_meta = fs::metadata(&current)?;
						let current_size = current_meta.len();
						match largest {
							None => Ok(Some((current, current_size))),
							Some((largest_path_so_far, largest_size_so_far)) => {
								if current_size > largest_size_so_far {
									Ok(Some((current, current_size)))
								} else {
									Ok(Some((largest_path_so_far, largest_size_so_far)))
								}
							}
						}
					});

			match largest_path_result {
				Ok(Some((p, _size))) => {
					// eprintln!("[DEBUG] Duplicate basename '{}': Keeping largest file
					// {}", basename, p.display());
					paths_to_process.push(p);
				}
				Ok(None) => {
					// This case should ideally not happen if the input `paths` was not
					// empty, but handle it defensively.
					eprintln!(
						"[WARN] No files found for basename '{basename}' during size \
						 comparison."
					);
				}
				Err(e) => {
					eprintln!(
						"[WARN] Error getting metadata for basename '{basename}', \
						 skipping: {e}"
					);
					// Decide if you want to skip the whole group or handle differently
				}
			}
		}
	}

	eprintln!(
		"[DEBUG] Filtered down to {} files to process.",
		paths_to_process.len()
	);

	// --- Process the filtered list of files ---
	for path in paths_to_process {
		// Calculate path relative to the docs_path root
		let relative_path = match path.strip_prefix(&docs_path) {
			Ok(p) => p.to_path_buf(),
			Err(e) => {
				eprintln!(
					"[WARN] Failed to strip prefix {} from {}: {}",
					docs_path.display(),
					path.display(),
					e
				);
				continue; // Skip if path manipulation fails
			}
		};
		let path_str = relative_path.to_string_lossy().to_string();

		let html_content = match fs::read_to_string(&path) {
			// Read from the absolute path
			Ok(content) => content,
			Err(e) => {
				eprintln!("[WARN] Failed to read file {}: {}", path.display(), e);
				continue; // Skip this file if reading fails
			}
		};

		let document = Html::parse_document(&html_content);

		if let Some(main_content_element) = document.select(&content_selector).next() {
			let text_content: String = main_content_element
				.text()
				.map(|s| s.trim())
				.filter(|s| !s.is_empty())
				.collect::<Vec<&str>>()
				.join("\n");

			if !text_content.is_empty() {
				documents.push(Document {
					path: path_str,
					content: text_content,
					html_content: html_content.clone(),
				});
			} else {
				eprintln!(
					"[DEBUG] No text content found in main section for: {}",
					path.display()
				);
			}
		} else {
			eprintln!(
				"[DEBUG] 'main-content' selector not found for: {}",
				path.display()
			);
		}
	}

	Ok(documents)
}

/// Generates documentation for a given crate in a temporary directory,
/// then loads and parses the HTML documents.
/// Extracts text content from the main content area of rustdoc generated HTML.
pub fn load_documents(
	crate_name: &str,
	crate_version_req: &str,
	features: Option<&Vec<String>>,
) -> Result<Vec<Document>> {
	let (documents, _version) = load_documents_with_version(crate_name, crate_version_req, features)?;
	Ok(documents)
}
