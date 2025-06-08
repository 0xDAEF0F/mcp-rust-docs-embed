use crate::{config::AppConfig, doc_loader};
use anyhow::Result;
use htmd::{
	HtmlToMarkdown,
	options::{HeadingStyle, Options},
};
use thin_logger::log;

pub struct DocumentationService {
	config: AppConfig,
}

impl DocumentationService {
	pub fn new(config: AppConfig) -> Self {
		Self { config }
	}

	pub async fn generate_docs(
		&self,
		crate_name: &str,
		version: Option<&str>,
		features: &[String],
	) -> Result<()> {
		// Use "*" for cargo (latest) as default if no version specified or "latest" is
		// specified
		let version_req = match version {
			Some("latest") | None => "*",
			Some(v) => v,
		};

		log::info!(
			"Generating documentation for crate: {crate_name} (version: {version_req})"
		);

		let features_vec = features.to_vec();
		let features_option = if features_vec.is_empty() {
			None
		} else {
			Some(&features_vec)
		};

		// Generate docs and get both the documents and resolved version
		let (documents, resolved_version) = if version_req == "*" {
			doc_loader::load_documents_with_version(
				crate_name,
				version_req,
				features_option,
			)
			.map_err(|e| anyhow::anyhow!("Failed to load documents: {}", e))?
		} else {
			let docs =
				doc_loader::load_documents(crate_name, version_req, features_option)
					.map_err(|e| anyhow::anyhow!("Failed to load documents: {}", e))?;
			(docs, version_req.to_string())
		};

		log::info!("Loaded {} documents", documents.len());

		log::info!("Resolved version: {resolved_version}");

		let converter = HtmlToMarkdown::builder()
			.skip_tags(vec!["script", "style", "meta", "head"])
			.options(Options {
				heading_style: HeadingStyle::Atx,
				..Default::default()
			})
			.build();

		let docs_dir = format!("docs/{crate_name}/{resolved_version}");
		std::fs::create_dir_all(&docs_dir)?;

		for doc in documents {
			let safe_path = doc.path.replace(['/', '\\'], "_");
			let file_path = format!("{docs_dir}/{safe_path}.md");

			let markdown_content = converter.convert(&doc.html_content).map_err(|e| {
				anyhow::anyhow!("Failed to convert HTML to markdown: {}", e)
			})?;

			std::fs::write(&file_path, &markdown_content)?;
			log::info!("Saved documentation to: {file_path}");
		}

		log::info!("Documentation generation complete");
		Ok(())
	}
}
