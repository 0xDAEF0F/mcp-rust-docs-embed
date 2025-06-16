use anyhow::{Context, Result};
use cargo::{
	core::{Workspace, resolver::features::CliFeatures},
	ops::{self, CompileOptions, DocOptions, Packages},
	util::context::GlobalContext,
};
use std::{
	fs::{self, File, create_dir_all},
	io::Write,
	path::{Path, PathBuf},
};
use tempfile::TempDir;

pub struct DocGenerator {
	temp_dir: TempDir,
	crate_name: String,
	crate_version: String,
	features: Option<Vec<String>>,
}

impl DocGenerator {
	pub fn new(
		crate_name: &str,
		crate_version: &str,
		features: Option<Vec<String>>,
	) -> Result<Self> {
		let temp_dir =
			tempfile::tempdir().context("Failed to create temporary directory")?;

		Ok(DocGenerator {
			temp_dir,
			crate_name: crate_name.to_string(),
			crate_version: crate_version.to_string(),
			features,
		})
	}

	pub fn generate_docs(&self) -> Result<PathBuf> {
		self.create_temp_project()?;
		self.run_cargo_doc()?;
		self.find_docs_path()
	}

	pub fn temp_dir_path(&self) -> &Path {
		self.temp_dir.path()
	}

	fn create_temp_project(&self) -> Result<()> {
		let temp_dir_path = self.temp_dir.path();
		let temp_manifest_path = temp_dir_path.join("Cargo.toml");

		let features_string = self
			.features
			.as_ref()
			.filter(|f| !f.is_empty())
			.map(|f| {
				let feature_list = f
					.iter()
					.map(|feat| format!("\"{feat}\""))
					.collect::<Vec<_>>()
					.join(", ");
				format!(", features = [{feature_list}]")
			})
			.unwrap_or_default();

		let cargo_toml_content = format!(
			r#"[package]
name = "temp-doc-crate"
version = "0.1.0"
edition = "2024"

[lib]

[dependencies]
{} = {{ version = "{}"{} }}
"#,
			self.crate_name, self.crate_version, features_string
		);

		let src_path = temp_dir_path.join("src");
		create_dir_all(&src_path)?;
		File::create(src_path.join("lib.rs"))?;

		let mut temp_manifest_file = File::create(&temp_manifest_path)?;
		temp_manifest_file.write_all(cargo_toml_content.as_bytes())?;

		Ok(())
	}

	fn run_cargo_doc(&self) -> Result<()> {
		let temp_manifest_path = self.temp_dir.path().join("Cargo.toml");

		let mut config = GlobalContext::default()?;
		config.configure(
			0,     // verbose
			true,  // quiet
			None,  // color
			false, // frozen
			false, // locked
			false, // offline
			&None, // target_dir
			&[],   // unstable_flags
			&[],   // cli_config
		)?;

		let mut ws = Workspace::new(&temp_manifest_path, &config)?;
		ws.set_target_dir(cargo::util::Filesystem::new(
			self.temp_dir.path().to_path_buf(),
		));

		let mut compile_opts = CompileOptions::new(
			&config,
			cargo::core::compiler::CompileMode::Doc {
				deps: false,
				json: false,
			},
		)?;

		compile_opts.cli_features = CliFeatures::new_all(false);
		compile_opts.spec = Packages::Packages(vec![self.crate_name.clone()]);

		let doc_opts = DocOptions {
			compile_opts,
			open_result: false,
			output_format: ops::OutputFormat::Html,
		};

		ops::doc(&ws, &doc_opts)?;
		Ok(())
	}

	fn find_docs_path(&self) -> Result<PathBuf> {
		let base_doc_path = self.temp_dir.path().join("doc");

		// Convert crate name to directory name (replace - with _)
		let crate_dir_name = self.crate_name.replace('-', "_");

		let target_docs_path = base_doc_path.join(&crate_dir_name);

		Ok(target_docs_path)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::fs;

	#[test]
	fn test_doc_generator_creation() {
		let generator = DocGenerator::new("serde", "1.0", None);
		assert!(generator.is_ok());

		let generator = generator.unwrap();
		assert_eq!(generator.crate_name, "serde");
		assert_eq!(generator.crate_version, "1.0");
		assert!(generator.features.is_none());
	}

	#[test]
	fn test_doc_generator_with_features() {
		let features = vec!["derive".to_string(), "std".to_string()];
		let generator = DocGenerator::new("serde", "1.0", Some(features.clone()));
		assert!(generator.is_ok());

		let generator = generator.unwrap();
		assert_eq!(generator.features, Some(features));
	}

	#[test]
	fn test_temp_project_creation() {
		let generator = DocGenerator::new("serde", "1.0", None).unwrap();
		let result = generator.create_temp_project();
		assert!(result.is_ok());

		let temp_path = generator.temp_dir_path();
		assert!(temp_path.join("Cargo.toml").exists());
		assert!(temp_path.join("src").exists());
		assert!(temp_path.join("src/lib.rs").exists());

		let cargo_toml_content =
			fs::read_to_string(temp_path.join("Cargo.toml")).unwrap();
		assert!(cargo_toml_content.contains("serde = { version = \"1.0\" }"));
	}

	#[test]
	fn test_temp_project_with_features() {
		let features = vec!["derive".to_string()];
		let generator = DocGenerator::new("serde", "1.0", Some(features)).unwrap();
		let result = generator.create_temp_project();
		assert!(result.is_ok());

		let cargo_toml_content =
			fs::read_to_string(generator.temp_dir_path().join("Cargo.toml")).unwrap();
		assert!(
			cargo_toml_content
				.contains("serde = { version = \"1.0\", features = [\"derive\"] }")
		);
	}
}
