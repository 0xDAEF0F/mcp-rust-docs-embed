use crate::{
	docs_builder::build_crate_docs,
	json_types::JsonDocs,
	my_types::{DocItem, create_doc_items_with_source},
};
use anyhow::Result;
use std::fs;
use tracing::info;

/// Generates JSON documentation for a given crate in a temporary directory,
/// then loads and parses the JSON documents.
/// Returns the DocItems with embedded source code and resolved version.
pub fn load_documents(
	crate_name: &str,
	crate_version_req: &str,
	features: &[String],
) -> Result<(Vec<DocItem>, String)> {
	// Generate documentation
	let (temp_dir, json_path) =
		build_crate_docs(crate_name, crate_version_req, features)?;

	info!("Using documentation path: {}", json_path.display());

	// Read the JSON file
	let json_content = fs::read_to_string(&json_path)?;

	// Parse JSON to get documentation items
	let rustdoc: JsonDocs = serde_json::from_str(&json_content)?;

	// Create DocItems with source code
	let doc_items = create_doc_items_with_source(&rustdoc, temp_dir.path())?;

	info!("Parsed {} documentation items", doc_items.len());

	// Extract version from Cargo.lock in the temp directory
	let resolved_version = extract_version_from_temp_dir(temp_dir.path(), crate_name)?;

	Ok((doc_items, resolved_version))
}

fn extract_version_from_temp_dir(
	temp_dir_path: &std::path::Path,
	crate_name: &str,
) -> Result<String> {
	let cargo_lock = temp_dir_path.join("Cargo.lock");
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

	anyhow::bail!("could not extract version from generated documentation")
}
