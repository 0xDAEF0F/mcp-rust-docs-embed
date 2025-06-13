use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Walks through a directory and returns all paths of files with .md extension
pub fn find_md_files<P: AsRef<Path>>(dir: P) -> Result<Vec<PathBuf>> {
	let mut md_files = Vec::new();

	for entry in WalkDir::new(dir) {
		let entry = entry?;
		let path = entry.path();

		if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
			md_files.push(path.to_path_buf());
		}
	}

	Ok(md_files)
}

/// Generate deterministically the table/collection name in "sqlite" and "qdrant" for a
/// given crate name and version
pub fn gen_table_name(crate_name: &str, version: &str) -> String {
	format!(
		"{}_v{}",
		crate_name.replace('-', "_"),
		version.replace('.', "_")
	)
}

/// Resolves the latest version of a Rust crate from crates.io
pub async fn resolve_latest_crate_version(crate_name: &str) -> Result<String> {
	let url = format!("https://crates.io/api/v1/crates/{}", crate_name);
	let client = reqwest::Client::new();

	let response = client
		.get(&url)
		.header("User-Agent", "embed-anything-rs")
		.send()
		.await?;

	if !response.status().is_success() {
		anyhow::bail!("Failed to fetch crate info: {}", response.status());
	}

	let json: serde_json::Value = response.json().await?;

	let version = json["crate"]["max_stable_version"]
		.as_str()
		.or_else(|| json["crate"]["max_version"].as_str())
		.ok_or_else(|| {
			anyhow::anyhow!("Could not find version for crate: {}", crate_name)
		})?;

	Ok(version.to_string())
}

#[cfg(test)]
mod tests {
	use super::*;
	use iter_tools::Itertools;
	use std::fs;
	use tempfile::TempDir;

	#[test]
	fn test_find_md_files_in_temp_dir() -> Result<()> {
		let temp_dir = TempDir::new()?;
		let temp_path = temp_dir.path();

		let md_files_to_create = vec!["index.md", "readme.md", "nested/doc.md"];

		for file_path in &md_files_to_create {
			let full_path = temp_path.join(file_path);

			// Create parent directory if needed
			if let Some(parent) = full_path.parent() {
				fs::create_dir_all(parent)?;
			}

			// Create the .md file with some content
			fs::write(&full_path, "# Test markdown file")?;
		}

		// Create some non-.md files that should be ignored
		fs::write(temp_path.join("test.txt"), "text file")?;
		fs::write(temp_path.join("config.json"), "{}")?;

		// Test the function
		let found_files = find_md_files(temp_path)?;

		// Should find exactly 3 .md files
		assert_eq!(found_files.len(), 3, "should find exactly 3 .md files");

		// All found files should have .md extension
		for file in &found_files {
			assert_eq!(
				file.extension().and_then(|s| s.to_str()),
				Some("md"),
				"file {file:?} should have .md extension",
			);
		}

		// Check that specific files are found
		let found_file_names = found_files
			.iter()
			.flat_map(|p| {
				anyhow::Ok(p.strip_prefix(temp_path)?.to_string_lossy().to_string())
			})
			.collect_vec();

		for expected_file in &md_files_to_create {
			assert!(
				found_file_names.iter().any(|f| f == expected_file),
				"Expected file {expected_file} not found in results: \
				 {found_file_names:?}"
			);
		}

		Ok(())
	}

	#[test]
	fn test_gen_table_name() {
		assert_eq!(gen_table_name("my-crate", "1.0.0"), "my_crate_v1_0_0");
	}

	#[tokio::test]
	async fn test_resolve_latest_crate_version() -> Result<()> {
		// test with anyhow crate
		let version = resolve_latest_crate_version("anyhow").await?;

		// verify it's a valid version format
		let parts: Vec<&str> = version.split('.').collect();
		assert_eq!(
			parts.len(),
			3,
			"Version should have 3 parts (major.minor.patch)"
		);

		// verify each part is a number
		for part in parts {
			part.parse::<u32>()
				.expect("Each version part should be a valid number");
		}

		// verify it matches known version format (e.g., 1.0.98)
		assert!(
			version.starts_with("1."),
			"anyhow version should start with 1."
		);

		Ok(())
	}

	#[tokio::test]
	async fn test_resolve_latest_crate_version_nonexistent() {
		// test with non-existent crate
		let result =
			resolve_latest_crate_version("this-crate-definitely-does-not-exist-12345")
				.await;
		assert!(result.is_err(), "Should fail for non-existent crate");
	}
}
