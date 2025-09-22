use super::types::{Chunk, ChunkKind};
use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use std::{collections::HashSet, ops::RangeInclusive};
use tiktoken_rs::{CoreBPE, cl100k_base};
use tracing::trace;
use tree_sitter::Node;

/// Tree-sitter nodes to ignore
const NODES_TO_IGNORE: [&str; 2] = ["import_statement", "import_alias"];

/// Maximum token limit for chunks
const MAX_TOKENS: usize = 8192;

/// Lazy-initialized BPE tokenizer to avoid repeated initialization
static BPE: Lazy<CoreBPE> = Lazy::new(|| cl100k_base().expect("Failed to initialize tiktoken BPE"));

/// Parses TypeScript source code into semantic chunks preserving documentation context
/// and respecting token limits for effective embedding generation
pub fn extract_typescript_chunks(source: &str) -> Result<Vec<Chunk>> {
   let start = std::time::Instant::now();
   trace!(
      "Starting chunk extraction for {} chars of source",
      source.len()
   );

   let mut parser = tree_sitter::Parser::new();
   let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
   parser.set_language(&language)?;

   let tree = parser
      .parse(source, None)
      .context("Failed to parse TypeScript source")?;
   let root_node = tree.root_node();

   let mut chunks = Vec::new();
   let mut cursor = root_node.walk();
   let mut processed_lines = HashSet::new();

   for child in root_node.children(&mut cursor) {
      if NODES_TO_IGNORE.contains(&child.kind()) {
         continue;
      }

      // Skip if this node has already been processed as part of another chunk
      if processed_lines.contains(&child.start_position().row) {
         continue;
      }

      // Special handling for decorators - they should be processed with their next sibling
      if child.kind() == "decorator" {
         if let Some(next) = child.next_sibling()
            && !processed_lines.contains(&next.start_position().row)
            && let Some(chunk) = process_decorated_node(&child, &next, source, &mut processed_lines)
         {
            chunks.push(chunk);
         }
         continue;
      }

      // Debug logging
      trace!(
         "Processing node: kind={}, line={}",
         child.kind(),
         child.start_position().row
      );

      if let Some(chunk) = process_node(&child, source, &mut processed_lines) {
         chunks.push(chunk);
      }
   }

   let elapsed = start.elapsed();
   trace!(
      "Chunk extraction completed in {:?} - produced {} chunks",
      elapsed,
      chunks.len()
   );

   Ok(chunks)
}

fn process_node(node: &Node, source: &str, processed_lines: &mut HashSet<usize>) -> Option<Chunk> {
   // Handle decorators specially - they're processed with their decorated nodes
   if node.kind() == "decorator" {
      return None;
   }

   let mut start_line = node.start_position().row;
   let end_line = node.end_position().row;

   // Find the earliest adjacent comment/decorator before this node
   if let Some(prev_sibling) = node.prev_sibling()
      && is_adjacent_decoration(&prev_sibling, node)
   {
      start_line = find_first_decoration(&prev_sibling);
   }

   // Determine chunk kind and handle special cases
   let kind = match node.kind() {
      "class_declaration" => ChunkKind::Class,
      "interface_declaration" => ChunkKind::Interface,
      "type_alias_declaration" => {
         // Only process exported type aliases
         if !is_exported_node(node, source) {
            return None;
         }
         ChunkKind::TypeAlias
      }
      "enum_declaration" => ChunkKind::Enum,
      "function_declaration" | "arrow_function" | "method_definition" => ChunkKind::Function,
      "lexical_declaration" => {
         // Only process exported const/let declarations
         if !is_const_or_export(node, source) {
            return None;
         }
         ChunkKind::Const
      }
      "export_statement" => {
         // First, check for comments before this export
         let mut start_line = node.start_position().row;
         if let Some(prev) = node.prev_sibling()
            && prev.kind() == "comment"
            && prev.end_position().row + 1 >= node.start_position().row
         {
            start_line = find_first_decoration(&prev);
         }

         // Check if this export contains decorators
         let mut has_decorators = false;
         let mut has_lexical = false;
         let mut cursor = node.walk();

         for child in node.children(&mut cursor) {
            if child.kind() == "decorator" {
               has_decorators = true;
            }
            if child.kind() == "lexical_declaration" {
               has_lexical = true;
            }
         }

         if has_decorators {
            // Process the entire export statement including decorators
            return process_decorated_export(node, source, processed_lines);
         }

         if has_lexical {
            // Process as a const/let export with any preceding comments
            mark_lines_processed(start_line..=node.end_position().row, processed_lines);
            let content = extract_lines(source, start_line..=node.end_position().row);
            let content = trim_to_token_limit(&content).unwrap_or_default();
            return Some(Chunk {
               kind: ChunkKind::Const,
               start_line: start_line + 1,
               end_line: node.end_position().row + 1,
               content,
            });
         }

         // For other exports, try to find the actual declaration
         let mut cursor = node.walk();
         for child in node.children(&mut cursor) {
            match child.kind() {
               "class_declaration"
               | "interface_declaration"
               | "function_declaration"
               | "type_alias_declaration"
               | "enum_declaration" => {
                  // Process with comments included
                  let kind = match child.kind() {
                     "class_declaration" => ChunkKind::Class,
                     "interface_declaration" => ChunkKind::Interface,
                     "function_declaration" => ChunkKind::Function,
                     "type_alias_declaration" => ChunkKind::TypeAlias,
                     "enum_declaration" => ChunkKind::Enum,
                     _ => continue,
                  };

                  mark_lines_processed(start_line..=node.end_position().row, processed_lines);
                  let content = extract_lines(source, start_line..=node.end_position().row);
                  let content = trim_to_token_limit(&content).unwrap_or_default();

                  return Some(Chunk {
                     kind,
                     start_line: start_line + 1,
                     end_line: node.end_position().row + 1,
                     content,
                  });
               }
               _ => {}
            }
         }

         return None;
      }
      "decorated_definition" => {
         return process_decorated_definition(node, source, processed_lines);
      }
      "comment" => {
         return handle_comment(node, source, start_line, processed_lines);
      }
      _ => return None,
   };

   // Mark lines as processed and extract content
   mark_lines_processed(start_line..=end_line, processed_lines);
   let content = extract_lines(source, start_line..=end_line);
   let content = trim_to_token_limit(&content).unwrap_or_default();

   Some(Chunk {
      kind,
      start_line: start_line + 1,
      end_line: end_line + 1,
      content,
   })
}

fn is_const_or_export(node: &Node, source: &str) -> bool {
   // Check if this is an exported const or let declaration
   let text = node.utf8_text(source.as_bytes()).unwrap_or("");
   text.starts_with("export const") || text.starts_with("export let")
}

fn is_exported_node(node: &Node, source: &str) -> bool {
   // Check if node is exported or has export parent
   if let Some(parent) = node.parent()
      && parent.kind() == "export_statement"
   {
      return true;
   }
   let text = node.utf8_text(source.as_bytes()).unwrap_or("");
   text.starts_with("export ")
}

fn is_adjacent_decoration(previous_sibling: &Node, next_sibling: &Node) -> bool {
   matches!(previous_sibling.kind(), "comment" | "decorator")
      && previous_sibling.end_position().row + 1 >= next_sibling.start_position().row
}

fn find_first_decoration(node: &Node) -> usize {
   let mut start_line = node.start_position().row;
   let mut current = *node;

   while let Some(prev) = current.prev_sibling() {
      if is_adjacent_decoration(&prev, &current) {
         start_line = prev.start_position().row;
         current = prev;
      } else {
         break;
      }
   }

   start_line
}

fn handle_comment(
   node: &Node,
   source: &str,
   start_line: usize,
   processed_lines: &mut HashSet<usize>,
) -> Option<Chunk> {
   // Check if this comment precedes an item declaration
   if is_comment_before_item(node) {
      return None;
   }

   // Collect all consecutive standalone comments
   let end_line = find_last_consecutive_comment(node);

   mark_lines_processed(start_line..=end_line, processed_lines);
   let content = extract_lines(source, start_line..=end_line);
   let content = trim_to_token_limit(&content).unwrap_or_default();

   Some(Chunk {
      kind: ChunkKind::Comment,
      start_line: start_line + 1,
      end_line: end_line + 1,
      content,
   })
}

fn is_comment_before_item(node: &Node) -> bool {
   let mut check_node = *node;

   // Look ahead through comments and decorators to find an item
   while let Some(next) = check_node.next_sibling() {
      match next.kind() {
         "class_declaration"
         | "interface_declaration"
         | "type_alias_declaration"
         | "enum_declaration"
         | "function_declaration"
         | "lexical_declaration"
         | "export_statement"
         | "decorated_definition" => {
            // Found an item - check if adjacent
            return check_node.end_position().row + 1 >= next.start_position().row;
         }
         "comment" | "decorator"
            if next.start_position().row <= check_node.end_position().row + 1 =>
         {
            // Continue through adjacent decorations
            check_node = next;
         }
         _ => break,
      }
   }

   false
}

fn find_last_consecutive_comment(node: &Node) -> usize {
   let mut end_line = node.end_position().row;
   let mut current = *node;

   while let Some(next) = current.next_sibling() {
      if next.kind() == "comment" && next.start_position().row <= current.end_position().row + 1 {
         end_line = next.end_position().row;
         current = next;
      } else {
         break;
      }
   }

   end_line
}

fn mark_lines_processed(range: RangeInclusive<usize>, processed_lines: &mut HashSet<usize>) {
   range.for_each(|line| {
      processed_lines.insert(line);
   });
}

fn extract_lines(source: &str, range: RangeInclusive<usize>) -> String {
   source
      .lines()
      .skip(*range.start())
      .take(range.end() - range.start() + 1)
      .collect::<Vec<_>>()
      .join("\n")
}

fn process_decorated_node(
   first_decorator: &Node,
   decorated_node: &Node,
   source: &str,
   processed_lines: &mut HashSet<usize>,
) -> Option<Chunk> {
   // Find all decorators before the decorated node
   let mut start_line = first_decorator.start_position().row;
   let end_line = decorated_node.end_position().row;

   // Check for comments before the first decorator
   if let Some(prev) = first_decorator.prev_sibling()
      && prev.kind() == "comment"
      && prev.end_position().row + 1 >= first_decorator.start_position().row
   {
      start_line = find_first_decoration(&prev);
   }

   // Handle the decorated node based on its type
   let actual_node = if decorated_node.kind() == "export_statement" {
      // For exported decorated classes, get the actual class
      if let Some(declaration) = decorated_node.child_by_field_name("declaration") {
         declaration
      } else {
         *decorated_node
      }
   } else {
      *decorated_node
   };

   let kind = match actual_node.kind() {
      "class_declaration" => ChunkKind::Class,
      "function_declaration" => ChunkKind::Function,
      "interface_declaration" => ChunkKind::Interface,
      _ => return None,
   };

   mark_lines_processed(start_line..=end_line, processed_lines);
   let content = extract_lines(source, start_line..=end_line);
   let content = trim_to_token_limit(&content).unwrap_or_default();

   Some(Chunk {
      kind,
      start_line: start_line + 1,
      end_line: end_line + 1,
      content,
   })
}

fn process_decorated_export(
   node: &Node,
   source: &str,
   processed_lines: &mut HashSet<usize>,
) -> Option<Chunk> {
   let mut start_line = node.start_position().row;
   let end_line = node.end_position().row;

   // Check for comments before the export
   if let Some(prev) = node.prev_sibling()
      && prev.kind() == "comment"
      && prev.end_position().row + 1 >= node.start_position().row
   {
      start_line = find_first_decoration(&prev);
   }

   // Find the actual declaration within the export
   let mut cursor = node.walk();
   let mut actual_kind = None;

   for child in node.children(&mut cursor) {
      match child.kind() {
         "class_declaration" => actual_kind = Some(ChunkKind::Class),
         "interface_declaration" => actual_kind = Some(ChunkKind::Interface),
         "function_declaration" => actual_kind = Some(ChunkKind::Function),
         _ => {}
      }
   }

   if let Some(kind) = actual_kind {
      mark_lines_processed(start_line..=end_line, processed_lines);
      let content = extract_lines(source, start_line..=end_line);
      let content = trim_to_token_limit(&content).unwrap_or_default();

      return Some(Chunk {
         kind,
         start_line: start_line + 1,
         end_line: end_line + 1,
         content,
      });
   }

   None
}

fn process_decorated_definition(
   node: &Node,
   source: &str,
   processed_lines: &mut HashSet<usize>,
) -> Option<Chunk> {
   let mut start_line = node.start_position().row;
   let end_line = node.end_position().row;

   // Check for comments before the decorators
   if let Some(prev_sibling) = node.prev_sibling()
      && prev_sibling.kind() == "comment"
      && is_adjacent_decoration(&prev_sibling, node)
   {
      start_line = find_first_decoration(&prev_sibling);
   }

   // Find the actual declaration within the decorated definition
   let mut cursor = node.walk();
   let mut actual_kind = None;

   for child in node.children(&mut cursor) {
      match child.kind() {
         "class_declaration" => actual_kind = Some(ChunkKind::Class),
         "interface_declaration" => actual_kind = Some(ChunkKind::Interface),
         "function_declaration" => actual_kind = Some(ChunkKind::Function),
         "type_alias_declaration" => actual_kind = Some(ChunkKind::TypeAlias),
         "enum_declaration" => actual_kind = Some(ChunkKind::Enum),
         "lexical_declaration" if is_const_or_export(&child, source) => {
            actual_kind = Some(ChunkKind::Const)
         }
         _ => {}
      }
   }

   if let Some(kind) = actual_kind {
      mark_lines_processed(start_line..=end_line, processed_lines);
      let content = extract_lines(source, start_line..=end_line);
      let content = trim_to_token_limit(&content).unwrap_or_default();

      return Some(Chunk {
         kind,
         start_line: start_line + 1,
         end_line: end_line + 1,
         content,
      });
   }

   None
}

fn trim_to_token_limit(content: &str) -> Result<String> {
   let start = std::time::Instant::now();
   let tokens = BPE.encode_with_special_tokens(content);
   let encode_time = start.elapsed();

   trace!(
      "Token encoding took {:?} for {} chars -> {} tokens",
      encode_time,
      content.len(),
      tokens.len()
   );

   if tokens.len() <= MAX_TOKENS {
      return Ok(content.to_string());
   }

   // Trim to MAX_TOKENS
   let trimmed_tokens = &tokens[..MAX_TOKENS];
   let decode_start = std::time::Instant::now();
   let trimmed_content = BPE.decode(trimmed_tokens.to_vec())?;
   let decode_time = decode_start.elapsed();

   trace!(
      "Token decoding took {:?} for {} tokens -> {} chars",
      decode_time,
      trimmed_tokens.len(),
      trimmed_content.len()
   );

   Ok(trimmed_content)
}
