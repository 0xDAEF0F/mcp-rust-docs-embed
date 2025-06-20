use anyhow::{Context, Result};
use cargo::{
	core::{Workspace, resolver::features::CliFeatures},
	ops::{self, CompileOptions, DocOptions, Packages},
	util::context::GlobalContext,
};
use std::{
	env,
	fs::{File, create_dir_all},
	io::Write,
	path::PathBuf,
};
use tempfile::TempDir;

/// Generates JSON documentation for a given crate in a temporary directory.
/// Returns the `TempDir` and the `PathBuf` to the JSON documentation file.
pub fn build_crate_docs(
	crate_name: &str,
	crate_version: &str,
	features: &[String],
) -> Result<(TempDir, PathBuf)> {
	let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;

	create_temp_project(&temp_dir, crate_name, crate_version, features)?;
	run_cargo_doc(&temp_dir, crate_name)?;

	let json_doc_path = temp_dir
		.path()
		.join("doc")
		.join(format!("{}.json", crate_name.replace('-', "_")));

	Ok((temp_dir, json_doc_path))
}

fn create_temp_project(
	temp_dir: &TempDir,
	crate_name: &str,
	crate_version: &str,
	features: &[String],
) -> Result<()> {
	let temp_dir_path = temp_dir.path();
	let temp_manifest_path = temp_dir_path.join("Cargo.toml");

	let features_string = if features.is_empty() {
		String::new()
	} else {
		let feature_list = features
			.iter()
			.map(|feat| format!("\"{feat}\""))
			.collect::<Vec<_>>()
			.join(", ");
		format!(", features = [{feature_list}]")
	};

	let cargo_toml_content = format!(
		r#"[package]
name = "temp-doc-crate"
version = "0.1.0"
edition = "2024"

[lib]

[dependencies]
{} = {{ version = "{}"{} }}
"#,
		crate_name, crate_version, features_string
	);

	let src_path = temp_dir_path.join("src");
	create_dir_all(&src_path)?;
	File::create(src_path.join("lib.rs"))?;

	let mut temp_manifest_file = File::create(&temp_manifest_path)?;
	temp_manifest_file.write_all(cargo_toml_content.as_bytes())?;

	Ok(())
}

fn run_cargo_doc(temp_dir: &TempDir, crate_name: &str) -> Result<()> {
	let temp_manifest_path = temp_dir.path().join("Cargo.toml");

	// Set RUSTDOCFLAGS to generate JSON output
	unsafe {
		env::set_var("RUSTDOCFLAGS", "-Z unstable-options --output-format json");
	}

	let mut config = GlobalContext::default()?;
	config.configure(
		0,                                 // verbose
		true,                              // quiet
		None,                              // color
		false,                             // frozen
		false,                             // locked
		false,                             // offline
		&None,                             // target_dir
		&["unstable-options".to_string()], // unstable_flags
		&[],                               // cli_config
	)?;

	let mut workspace = Workspace::new(&temp_manifest_path, &config)?;
	workspace.set_target_dir(cargo::util::Filesystem::new(temp_dir.path().to_path_buf()));

	let mut compile_opts = CompileOptions::new(
		&config,
		cargo::core::compiler::CompileMode::Doc {
			deps: false,
			json: false,
		},
	)?;
	compile_opts.cli_features = CliFeatures::new_all(false);
	compile_opts.spec = Packages::Packages(vec![crate_name.to_string()]);

	let doc_opts = DocOptions {
		compile_opts,
		open_result: false,
		output_format: ops::OutputFormat::Html,
	};

	ops::doc(&workspace, &doc_opts)?;
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::fs;

	#[test]
	fn test_build_crate_docs() {
		let temp_dir = build_crate_docs("serde", "1.0", &[]);

		assert!(temp_dir.is_ok());

		let (temp_dir, _) = temp_dir.unwrap();

		let temp_path = temp_dir.path();
		assert!(temp_path.join("Cargo.toml").exists());
		assert!(temp_path.join("src").exists());
		assert!(temp_path.join("src/lib.rs").exists());

		let cargo_toml_content =
			fs::read_to_string(temp_path.join("Cargo.toml")).unwrap();
		assert!(cargo_toml_content.contains("serde = { version = \"1.0\" }"));
	}

	#[test]
	fn test_build_crate_docs_with_features() {
		let features = vec!["derive".to_string(), "std".to_string()];
		let result = build_crate_docs("serde", "1.0", &features);
		assert!(result.is_ok());

		let (temp_dir, _) = result.unwrap();
		let cargo_toml_content =
			fs::read_to_string(temp_dir.path().join("Cargo.toml")).unwrap();
		assert!(
			cargo_toml_content.contains(
				"serde = { version = \"1.0\", features = [\"derive\", \"std\"] }"
			)
		);
	}

	#[test]
	fn test_build_crate_docs_returns_correct_path() {
		let result = build_crate_docs("serde", "1.0", &[]);
		assert!(result.is_ok());

		let (temp_dir, docs_path) = result.unwrap();
		assert_eq!(docs_path, temp_dir.path().join("doc").join("serde.json"));
	}

	#[test]
	fn test_docs_path_construction() {
		// Test that the path is constructed correctly without actually building docs
		let temp_dir = tempfile::tempdir().unwrap();
		let crate_name = "test-crate";
		let expected_path = temp_dir.path().join("doc").join("test_crate.json");

		// The actual construction logic from build_crate_docs
		let constructed_path = temp_dir
			.path()
			.join("doc")
			.join(format!("{}.json", crate_name.replace('-', "_")));

		assert_eq!(constructed_path, expected_path);
	}
}
