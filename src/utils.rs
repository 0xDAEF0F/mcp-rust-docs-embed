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
}
