use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CrateResponse {
	#[serde(rename = "crate")]
	crate_info: CrateInfo,
	versions: Vec<VersionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CrateInfo {
	name: String,
	max_version: String,
	max_stable_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VersionInfo {
	num: String,
	features: Option<HashMap<String, Vec<String>>>,
}

/// fetches the feature flags for a crate from crates.io
/// if version is None, fetches features for the latest version
pub async fn get_crate_features(
	crate_name: &str,
	version: Option<&str>,
) -> Result<Vec<String>> {
	let version = match version {
		Some(v) => v.to_string(),
		None => crate::utils::resolve_latest_crate_version(crate_name).await?,
	};

	let url = format!("https://crates.io/api/v1/crates/{}/{}", crate_name, version);
	let client = reqwest::Client::new();

	let response = client
		.get(&url)
		.header("User-Agent", "embed-anything-rs")
		.send()
		.await?;

	if !response.status().is_success() {
		anyhow::bail!(
			"failed to fetch crate version info: {} for {}/{}",
			response.status(),
			crate_name,
			version
		);
	}

	let version_response: VersionResponse = response.json().await?;

	let features = version_response.version.features.unwrap_or_default();

	let mut feature_names: Vec<String> = features.keys().cloned().collect();
	feature_names.sort();

	Ok(feature_names)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VersionResponse {
	version: VersionDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VersionDetail {
	num: String,
	features: Option<HashMap<String, Vec<String>>>,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_get_crate_features_specific_version() -> Result<()> {
		// test with serde 1.0.0 which we know has features
		let features = get_crate_features("serde", Some("1.0.0")).await?;

		// serde should have some features
		assert!(!features.is_empty(), "serde should have features");

		// check for known features
		assert!(
			features.contains(&"derive".to_string()),
			"serde should have 'derive' feature"
		);
		assert!(
			features.contains(&"std".to_string()),
			"serde should have 'std' feature"
		);

		// verify features are sorted
		let mut sorted_features = features.clone();
		sorted_features.sort();
		assert_eq!(features, sorted_features, "features should be sorted");

		Ok(())
	}

	#[tokio::test]
	async fn test_get_crate_features_latest() -> Result<()> {
		// test with anyhow which has minimal features
		let features = get_crate_features("anyhow", None).await?;

		// anyhow has features like std, default
		assert!(!features.is_empty(), "anyhow should have some features");

		Ok(())
	}

	#[tokio::test]
	async fn test_get_crate_features_no_features() -> Result<()> {
		// test with a simple crate that might have no features
		// using once_cell version 1.0.0 as an example
		let result = get_crate_features("once_cell", Some("1.0.0")).await;

		match result {
			Ok(features) => {
				// it's ok if it has features or not
				println!("once_cell 1.0.0 features: {:?}", features);
			}
			Err(e) => {
				// also ok if this specific version doesn't exist
				println!("Error (expected for old version): {}", e);
			}
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_get_crate_features_nonexistent_crate() {
		let result =
			get_crate_features("this-crate-does-not-exist-12345", Some("1.0.0")).await;
		assert!(result.is_err(), "should fail for non-existent crate");
	}

	#[tokio::test]
	async fn test_get_crate_features_nonexistent_version() {
		let result = get_crate_features("serde", Some("999.999.999")).await;
		assert!(result.is_err(), "should fail for non-existent version");
	}

	#[tokio::test]
	async fn test_tokio_features() -> Result<()> {
		// test with tokio which has many features
		let features = get_crate_features("tokio", Some("1.0.0")).await?;

		// println!("features: {features:#?}");

		// tokio should have many features
		assert!(features.len() > 5, "tokio should have many features");

		// check for some common tokio features
		let expected_features = ["full", "net", "rt", "time", "io-util"];
		for feature in expected_features {
			assert!(
				features.contains(&feature.to_string()),
				"tokio should have '{}' feature",
				feature
			);
		}

		Ok(())
	}
}

