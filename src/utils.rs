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

	#[test]
	fn test_find_md_files_in_tap_md() {
		let result = find_md_files("tap-md");

		assert!(result.is_ok(), "find_md_files should succeed");

		let md_files = result.unwrap();

		// should find at least one .md file (index.md)
		assert!(!md_files.is_empty(), "should find at least one .md file");

		// all returned files should have .md extension
		for file in &md_files {
			assert_eq!(
				file.extension().and_then(|s| s.to_str()),
				Some("md"),
				"file {file:?} should have .md extension",
			);
		}

		// print found files for verification
		println!("Found {} .md files:", md_files.len());
		for file in &md_files {
			println!("  {file:?}");
		}
	}
}
