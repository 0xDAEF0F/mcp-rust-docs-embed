use anyhow::{Result, bail};
use url::Url;

/// Creates consistent collection names across server restarts to ensure
/// embeddings can be reliably retrieved for any repository
pub fn gen_table_name_for_repo(repo_url: &str) -> Result<String> {
   let repo_name = extract_repo_name_from_url(repo_url)?;
   Ok(format!("repo_{}", repo_name.replace(['-', '/'], "_")))
}

/// Converts repository URLs into filesystem-safe identifiers for storage
/// and display purposes
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

/// Normalizes various repository input formats into canonical GitHub URLs,
/// supporting both shorthand and full URL inputs for user convenience
pub fn parse_repository_input(input: &str) -> Result<String> {
   // Check if it's already a valid URL
   if let Ok(url) = Url::parse(input) {
      // If it's a GitHub URL, extract just the owner/repo part
      if url.host_str() == Some("github.com") {
         let path = url.path().trim_matches('/');
         let parts: Vec<&str> = path.split('/').collect();
         if parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            return Ok(format!("https://github.com/{}/{}", parts[0], parts[1]));
         }
      }
      // For non-GitHub URLs or invalid GitHub paths, return as-is
      return Ok(input.to_string());
   }

   // Otherwise, try to parse as owner/repo format
   let parts: Vec<&str> = input.split('/').collect();
   if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
      Ok(format!("https://github.com/{input}"))
   } else {
      bail!("Invalid repository format. Expected 'owner/repo' or a full repository URL")
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

   #[test]
   fn test_parse_repository_input() -> Result<()> {
      // Test full URLs
      assert_eq!(
         parse_repository_input("https://github.com/rust-lang/rust")?,
         "https://github.com/rust-lang/rust"
      );

      // Test GitHub URLs with extra path segments
      assert_eq!(
         parse_repository_input(
            "https://github.com/0xDAEF0F/da-crawler/blob/master/utils/job-ai-analysis.schema.ts"
         )?,
         "https://github.com/0xDAEF0F/da-crawler"
      );
      assert_eq!(
         parse_repository_input("https://github.com/rust-lang/rust/tree/master/src/libstd")?,
         "https://github.com/rust-lang/rust"
      );
      assert_eq!(
         parse_repository_input("https://github.com/tokio-rs/tokio/pull/1234")?,
         "https://github.com/tokio-rs/tokio"
      );

      // Test non-GitHub URLs (returned as-is)
      assert_eq!(
         parse_repository_input("https://gitlab.com/owner/repo")?,
         "https://gitlab.com/owner/repo"
      );

      // Test owner/repo format
      assert_eq!(
         parse_repository_input("rust-lang/rust")?,
         "https://github.com/rust-lang/rust"
      );
      assert_eq!(
         parse_repository_input("tokio-rs/tokio")?,
         "https://github.com/tokio-rs/tokio"
      );

      // Test invalid formats
      assert!(parse_repository_input("invalid").is_err());
      assert!(parse_repository_input("owner/repo/extra").is_err());
      assert!(parse_repository_input("/repo").is_err());
      assert!(parse_repository_input("owner/").is_err());

      Ok(())
   }
}
