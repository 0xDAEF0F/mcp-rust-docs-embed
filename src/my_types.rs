use crate::json_types::JsonDocs;
use anyhow::{Context, Result};
use std::{fmt, fs, path::Path};

#[derive(Debug, Clone, PartialEq)]
pub enum ItemType {
	Struct,
	Enum,
	Function,
	Constant,
	Impl,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileRange {
	pub start: (u32, u32),
	pub end: (u32, u32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocItem {
	pub name: Option<String>,
	pub doc_string: Option<String>,
	pub r#type: ItemType,
	pub source_code: String,
	pub filename: String,
}

impl fmt::Display for DocItem {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if let Some(doc_string) = &self.doc_string {
			writeln!(f, "{}", doc_string)?;
			writeln!(f)?;
		}
		writeln!(f, "```rust")?;
		write!(f, "{}", self.source_code)?;
		write!(f, "\n```")
	}
}

/// Creates DocItems from JsonDocs, reading the source code from the temp directory
pub fn create_doc_items_with_source(
	docs: &JsonDocs,
	temp_dir: &Path,
) -> Result<Vec<DocItem>> {
	let mut doc_items = Vec::new();

	for item in docs.index.values() {
		// Filter criteria
		if item.crate_id != 0
			|| item.span.is_none()
			|| matches!(
				item.item_type(),
				Some("struct_field") | Some("variant") | Some("module")
			) {
			continue;
		}

		// Extract item type
		let item_type = match item.item_type() {
			Some("struct") => ItemType::Struct,
			Some("enum") => ItemType::Enum,
			Some("function") => ItemType::Function,
			Some("constant") => ItemType::Constant,
			Some("impl") => ItemType::Impl,
			_ => continue,
		};

		// Get span information
		let span = match &item.span {
			Some(s) => s,
			None => continue,
		};

		// Build the full path to the source file
		let source_path = temp_dir.join(&span.filename);

		// Read the source file
		let source_content = fs::read_to_string(&source_path).with_context(|| {
			format!("Failed to read source file: {}", source_path.display())
		})?;

		// Split into lines for easy access
		let lines: Vec<&str> = source_content.lines().collect();

		// Extract the relevant code using the line range
		// Line numbers in the JSON are 1-based
		let start_line = (span.begin.0 as usize).saturating_sub(1);
		let end_line = (span.end.0 as usize).min(lines.len());

		if start_line >= lines.len() {
			tracing::warn!(
				"Invalid line range for {:?} in {}: start={}, total lines={}",
				item.name,
				span.filename,
				span.begin.0,
				lines.len()
			);
			continue;
		}

		// Extract the code chunk
		let code_lines = &lines[start_line..end_line];
		let source_code = code_lines.join("\n");

		doc_items.push(DocItem {
			name: item.name.clone(),
			doc_string: item.docs.clone(),
			r#type: item_type,
			source_code,
			filename: span.filename.clone(),
		});
	}

	Ok(doc_items)
}
