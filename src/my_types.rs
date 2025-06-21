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

		// Skip functions that are part of impl blocks by checking for self parameter
		if item_type == ItemType::Function && is_impl_function(&source_code) {
			continue;
		}

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

/// Checks if a function is part of an impl block by looking for self parameter
fn is_impl_function(source_code: &str) -> bool {
	// Find the opening parenthesis after 'fn' keyword
	// This handles generics that might appear between fn name and parameters
	if let Some(fn_pos) = source_code.find("fn ") {
		// Find the opening parenthesis for parameters
		if let Some(paren_pos) = source_code[fn_pos..].find('(') {
			let after_paren = &source_code[fn_pos + paren_pos + 1..];
			// Get everything up to the first comma or closing paren
			let first_param = if let Some(comma_pos) = after_paren.find(',') {
				&after_paren[..comma_pos]
			} else if let Some(paren_pos) = after_paren.find(')') {
				&after_paren[..paren_pos]
			} else {
				return false;
			};

			// Remove whitespace and check if it contains "self" as a whole word
			let normalized = first_param.trim();

			// Simple cases
			if normalized == "self" || normalized == "&self" || normalized == "&mut self"
			{
				return true;
			}

			// Handle lifetimes and spacing variations
			// Split by whitespace and filter empty strings
			let parts: Vec<&str> = normalized.split_whitespace().collect();

			// Check various patterns
			match parts.as_slice() {
				["self"] => true,
				["&", "self"] => true,
				["&", "mut", "self"] => true,
				["&mut", "self"] => true,
				// Handle lifetime cases
				[s, "self"] if s.starts_with("&'") => true,
				["&", l, "self"] if l.starts_with("'") => true,
				["&", l, "mut", "self"] if l.starts_with("'") => true,
				// Handle self with type annotation
				["self", ":", ..] => true,
				["&self", ":", ..] => true,
				["&", "self", ":", ..] => true,
				["&mut", "self", ":", ..] => true,
				["&", "mut", "self", ":", ..] => true,
				[s, "self", ":", ..] if s.starts_with("&'") => true,
				["&", l, "self", ":", ..] if l.starts_with("'") => true,
				["&", l, "mut", "self", ":", ..] if l.starts_with("'") => true,
				_ => false,
			}
		} else {
			false
		}
	} else {
		false
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_is_impl_function() {
		// Test regular impl functions
		assert!(is_impl_function("fn foo(&self) -> i32 { 42 }"));
		assert!(is_impl_function("fn foo(&mut self) -> i32 { 42 }"));
		assert!(is_impl_function("fn foo(self) -> i32 { 42 }"));
		assert!(is_impl_function("fn foo(&'a self) -> i32 { 42 }"));
		assert!(is_impl_function("fn foo(&'_ self) -> i32 { 42 }"));
		assert!(is_impl_function("fn foo( &self ) -> i32 { 42 }"));
		assert!(is_impl_function("fn foo( & mut self ) -> i32 { 42 }"));

		// Test with generics
		assert!(is_impl_function("fn foo<T>(&self) -> T { todo!() }"));
		assert!(is_impl_function("fn foo<T, U>(&mut self) -> T { todo!() }"));
		assert!(is_impl_function(
			"fn foo<'a, T: Clone>(&'a self) -> &'a T { todo!() }"
		));

		// Test standalone functions
		assert!(!is_impl_function("fn foo() -> i32 { 42 }"));
		assert!(!is_impl_function("fn foo(x: i32) -> i32 { x }"));
		assert!(!is_impl_function("fn foo(x: &Self) -> i32 { 42 }"));
		assert!(!is_impl_function("fn foo(selfish: i32) -> i32 { selfish }"));
		assert!(!is_impl_function("fn foo<T>(x: T) -> T { x }"));
		assert!(!is_impl_function(
			"fn foo<T>(x: &T, y: &T) -> bool { x == y }"
		));
	}
}
