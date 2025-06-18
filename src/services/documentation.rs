use crate::doc_loader;
use anyhow::{Context, Result};
use htmd::{
	HtmlToMarkdown,
	options::{HeadingStyle, Options},
};
use thin_logger::log;

pub fn generate_md_docs(
	crate_name: &str,
	version: &str,
	features: &[String],
) -> Result<()> {
	log::info!(
		"Generating documentation for crate: {crate_name} (version: {version}) with \
		 features: [{}]",
		features.join(", ")
	);

	// Generate docs and get both the documents and resolved version
	let (documents, resolved_version) =
		doc_loader::load_documents(crate_name, version, features)
			.context("Failed to load documents")?;

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

		let markdown_content = converter
			.convert(&doc.html_content)
			.context("Failed to convert HTML to markdown")?;

		std::fs::write(&file_path, &markdown_content)?;
		log::info!("Saved documentation to: {file_path}");
	}

	log::info!("Documentation generation complete");
	Ok(())
}
