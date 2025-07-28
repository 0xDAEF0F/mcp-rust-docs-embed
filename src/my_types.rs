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
   pub span: FileRange,
}

impl fmt::Display for DocItem {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      if let Some(doc_string) = &self.doc_string {
         writeln!(f, "{doc_string}")?;
         writeln!(f)?;
      }
      writeln!(f, "```rust")?;
      write!(f, "{}", self.source_code)?;
      write!(f, "\n```")
   }
}

/// Transforms rustdoc JSON output into structured items with source code,
/// filtering out internal items and preserving only public API elements
pub fn create_doc_items_with_source(docs: &JsonDocs, temp_dir: &Path) -> Result<Vec<DocItem>> {
   let mut doc_items = Vec::new();

   for item in docs.index.values() {
      // Filter criteria
      if item.crate_id != 0
         || item.span.is_none()
         || matches!(
            item.item_type(),
            Some("struct_field") | Some("variant") | Some("module")
         )
      {
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
      let source_content = fs::read_to_string(&source_path)
         .with_context(|| format!("Failed to read source file: {}", source_path.display()))?;

      // Split into lines for easy access
      let lines: Vec<&str> = source_content.lines().collect();

      // Extract the relevant code using the line range
      // Line numbers in the JSON are 1-based
      let mut start_line = (span.begin.0 as usize).saturating_sub(1);
      let end_line = (span.end.0 as usize).min(lines.len());

      // For struct/enum/constant/function items, include any preceding attributes
      if matches!(
         item_type,
         ItemType::Struct | ItemType::Enum | ItemType::Constant | ItemType::Function
      ) {
         start_line = find_start_line_with_attributes(&lines, start_line);
      }

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

      // Skip derive attribute Impl items (they'll be bundled with their target items)
      if item_type == ItemType::Impl && source_code.trim_start().starts_with("#[") {
         continue;
      }

      // Skip false "function" items that are actually just derive attributes
      // These are a rustdoc JSON bug where derive attributes get classified as
      // functions
      if item_type == ItemType::Function {
         let trimmed_code = source_code.trim();
         // If it starts with attributes and doesn't contain "fn ", it's not a real
         // function
         if trimmed_code.starts_with("#[") && !trimmed_code.contains("fn ") {
            continue;
         }
      }

      doc_items.push(DocItem {
         name: item.name.clone(),
         doc_string: item.docs.clone(),
         r#type: item_type,
         source_code,
         filename: span.filename.clone(),
         span: FileRange {
            start: span.begin,
            end: span.end,
         },
      });
   }

   // Second pass: filter out functions that are within impl blocks
   filter_impl_functions(doc_items)
}

/// Filters out functions that are within impl blocks by comparing spans
fn filter_impl_functions(doc_items: Vec<DocItem>) -> Result<Vec<DocItem>> {
   // collect all impl block spans grouped by filename
   let impl_spans: std::collections::HashMap<String, Vec<FileRange>> = doc_items
      .iter()
      .filter(|item| item.r#type == ItemType::Impl)
      .fold(std::collections::HashMap::new(), |mut acc, item| {
         acc.entry(item.filename.clone())
            .or_default()
            .push(item.span.clone());
         acc
      });

   // filter out functions that are within any impl block span
   let filtered_items = doc_items
      .into_iter()
      .filter(|item| {
         if item.r#type != ItemType::Function {
            return true;
         }

         // check if this function's span is within any impl block span in the same
         // file
         if let Some(impl_ranges) = impl_spans.get(&item.filename) {
            for impl_range in impl_ranges {
               if is_span_within(impl_range, &item.span) {
                  return false;
               }
            }
         }

         true
      })
      .collect();

   Ok(filtered_items)
}

/// checks if the inner span is completely within the outer span
fn is_span_within(outer: &FileRange, inner: &FileRange) -> bool {
   // check if inner span is completely within outer span
   (outer.start.0 < inner.start.0
      || (outer.start.0 == inner.start.0 && outer.start.1 <= inner.start.1))
      && (outer.end.0 > inner.end.0 || (outer.end.0 == inner.end.0 && outer.end.1 >= inner.end.1))
}

/// Finds the start line that includes any preceding attributes for an item
/// Returns the adjusted start line index (0-based) that includes all attributes
fn find_start_line_with_attributes(lines: &[&str], item_start_line: usize) -> usize {
   let mut current_line = item_start_line;

   // Look backwards for attributes and empty lines
   while current_line > 0 {
      let prev_line_idx = current_line - 1;
      let prev_line = lines[prev_line_idx].trim();

      if prev_line.starts_with("#[") || prev_line.is_empty() {
         // Include attributes and empty lines
         current_line = prev_line_idx;
      } else {
         // Hit a non-empty, non-attribute line, stop looking
         break;
      }
   }

   current_line
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn test_is_span_within() {
      // Test case where inner is completely within outer
      let outer = FileRange {
         start: (10, 0),
         end: (20, 0),
      };
      let inner = FileRange {
         start: (12, 0),
         end: (18, 0),
      };
      assert!(is_span_within(&outer, &inner));

      // Test case where inner starts at same line but different column
      let outer = FileRange {
         start: (10, 5),
         end: (20, 0),
      };
      let inner = FileRange {
         start: (10, 10),
         end: (18, 0),
      };
      assert!(is_span_within(&outer, &inner));

      // Test case where inner is not within outer (starts before)
      let outer = FileRange {
         start: (10, 0),
         end: (20, 0),
      };
      let inner = FileRange {
         start: (5, 0),
         end: (15, 0),
      };
      assert!(!is_span_within(&outer, &inner));

      // Test case where inner is not within outer (ends after)
      let outer = FileRange {
         start: (10, 0),
         end: (20, 0),
      };
      let inner = FileRange {
         start: (15, 0),
         end: (25, 0),
      };
      assert!(!is_span_within(&outer, &inner));

      // Test case where spans are identical
      let outer = FileRange {
         start: (10, 0),
         end: (20, 0),
      };
      let inner = FileRange {
         start: (10, 0),
         end: (20, 0),
      };
      assert!(is_span_within(&outer, &inner)); // identical spans should be considered within
   }

   #[test]
   fn test_find_start_line_with_attributes() {
      // Test case 1: No attributes
      let lines = vec!["fn foo() {}", "    42", "}"];
      assert_eq!(find_start_line_with_attributes(&lines, 0), 0);

      // Test case 2: Single attribute
      let lines = vec!["#[derive(Debug)]", "struct Foo {", "    x: i32,", "}"];
      assert_eq!(find_start_line_with_attributes(&lines, 1), 0);

      // Test case 3: Multiple attributes
      let lines = vec![
         "#[derive(Debug)]",
         "#[serde(rename_all = \"camelCase\")]",
         "struct Foo {",
         "    x: i32,",
         "}",
      ];
      assert_eq!(find_start_line_with_attributes(&lines, 2), 0);

      // Test case 4: Attributes with empty lines
      let lines = vec![
         "#[derive(Debug)]",
         "#[serde(rename_all = \"camelCase\")]",
         "",
         "struct Foo {",
         "    x: i32,",
         "}",
      ];
      assert_eq!(find_start_line_with_attributes(&lines, 3), 0);

      // Test case 5: Mixed content - should stop at non-attribute
      let lines = vec![
         "use std::fmt;",
         "#[derive(Debug)]",
         "struct Foo {",
         "    x: i32,",
         "}",
      ];
      assert_eq!(find_start_line_with_attributes(&lines, 2), 1);

      // Test case 6: Edge case - first line (no preceding lines)
      let lines = vec!["struct Foo {", "    x: i32,", "}"];
      assert_eq!(find_start_line_with_attributes(&lines, 0), 0);
   }
}
