use anyhow::{Result, bail};
use url::Url;

/// Generate deterministically the collection name in Qdrant for a
/// given repository URL
pub fn gen_table_name_for_repo(repo_url: &str) -> Result<String> {
	let repo_name = extract_repo_name_from_url(repo_url)?;
	Ok(format!("repo_{}", repo_name.replace(['-', '/'], "_")))
}

/// Extract a safe repository name from a URL
/// e.g., "https://github.com/owner/repo" -> "owner_repo"
pub fn extract_repo_name_from_url(repo_url: &str) -> Result<String> {
	let url = Url::parse(repo_url)?;

	// Get the path and remove leading/trailing slashes
	let path = url.path().trim_matches('/');

	// For GitHub URLs, extract owner/repo
	let parts: Vec<&str> = path.split('/').collect();
	if parts.len() >= 2 {
		// Take the first two parts (owner/repo)
		Ok(format!("{}_{}", parts[0], parts[1]))
	} else {
		bail!("Invalid repository URL format")
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_gen_table_name_for_repo() -> Result<()> {
		assert_eq!(
			gen_table_name_for_repo("https://github.com/rust-lang/rust")?,
			"repo_rust_lang_rust"
		);
		assert_eq!(
			gen_table_name_for_repo("https://github.com/tokio-rs/tokio")?,
			"repo_tokio_rs_tokio"
		);
		Ok(())
	}

	#[test]
	fn test_extract_repo_name_from_url() -> Result<()> {
		assert_eq!(
			extract_repo_name_from_url("https://github.com/rust-lang/rust")?,
			"rust-lang_rust"
		);
		assert_eq!(
			extract_repo_name_from_url("https://github.com/tokio-rs/tokio")?,
			"tokio-rs_tokio"
		);
		Ok(())
	}
}
