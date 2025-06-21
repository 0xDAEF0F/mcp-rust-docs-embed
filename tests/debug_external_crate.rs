use anyhow::Result;
use mcp_rust_docs_embed::doc_loader;
use std::collections::HashMap;

#[tokio::test]
async fn debug_external_crate() -> Result<()> {
	let crate_name = "log";
	let version = "*";
	let features = vec![];

	println!(
		"\n=== Loading documents for {} v{} ===\n",
		crate_name, version
	);

	let (doc_items, resolved_version) =
		doc_loader::load_documents(crate_name, version, &features)?;

	println!("Resolved to version: {}", resolved_version);
	println!("Total doc items loaded: {}\n", doc_items.len());

	let mut type_counts = HashMap::new();
	for item in &doc_items {
		*type_counts.entry(format!("{:?}", item.r#type)).or_insert(0) += 1;
	}

	println!("=== Item type distribution ===");
	for (item_type, count) in &type_counts {
		println!("{}: {}", item_type, count);
	}

	println!("\n=== Items ===");
	for item in &doc_items {
		println!("~~~~~~{:?}~~~~~~~\n", item.r#type);
		println!("{}\n", item.source_code);
	}

	Ok(())
}
