use super::types::{Chunk, ChunkKind};
use anyhow::Result;
use once_cell::sync::Lazy;
use text_splitter::{ChunkConfig, MarkdownSplitter};
use tiktoken_rs::{CoreBPE, cl100k_base};
use tracing::trace;

/// Maximum token limit for chunks
const MAX_TOKENS: usize = 8192;

/// Lazy-initialized BPE tokenizer to avoid repeated initialization
static BPE: Lazy<CoreBPE> = Lazy::new(|| cl100k_base().expect("Failed to initialize tiktoken BPE"));

/// Splits Markdown documents into semantic sections preserving headings and content
/// relationships for optimal documentation search and retrieval
pub fn extract_markdown_chunks(source: &str) -> Result<Vec<Chunk>> {
   let start = std::time::Instant::now();
   trace!(
      "Starting markdown chunk extraction for {} chars of source",
      source.len()
   );

   let splitter = MarkdownSplitter::new(ChunkConfig::new(MAX_TOKENS));
   let mut chunks = Vec::new();

   for (i, (byte_offset, chunk_text)) in splitter.chunk_indices(source).enumerate() {
      // Calculate line numbers
      let start_line = source[..byte_offset].matches('\n').count() + 1;
      let end_line = start_line + chunk_text.matches('\n').count();

      chunks.push(Chunk {
         kind: ChunkKind::MarkdownSection,
         start_line,
         end_line,
         content: chunk_text.to_string(),
      });

      trace!(
         "Created markdown chunk {} with {} tokens at lines {}-{}",
         i,
         BPE.encode_with_special_tokens(chunk_text).len(),
         start_line,
         end_line
      );
   }

   let elapsed = start.elapsed();
   trace!(
      "Markdown chunk extraction completed in {:?} - produced {} chunks",
      elapsed,
      chunks.len()
   );

   Ok(chunks)
}

#[cfg(test)]
mod tests {
   use super::*;
   use std::fs;

   #[test]
   fn test_extract_markdown_chunks() {
      let markdown_content = fs::read_to_string("source-code-examples/README.md")
         .expect("Failed to read test markdown file");

      let chunks = extract_markdown_chunks(&markdown_content).expect("Failed to extract chunks");

      // Log the chunks
      println!("\n=== Markdown Chunks ===");
      for (i, chunk) in chunks.iter().enumerate() {
         println!(
            "\nChunk {} (lines {}-{}):",
            i, chunk.start_line, chunk.end_line
         );
         println!(
            "Content preview: {}...",
            chunk.content.chars().take(100).collect::<String>()
         );
         println!("Content length: {} chars", chunk.content.len());
      }
      println!("\nTotal chunks: {}\n", chunks.len());

      // Basic assertions
      assert!(
         !chunks.is_empty(),
         "Should have extracted at least one chunk"
      );
      for chunk in &chunks {
         assert!(chunk.start_line > 0, "Start line should be positive");
         assert!(
            chunk.end_line >= chunk.start_line,
            "End line should be >= start line"
         );
         assert!(
            !chunk.content.is_empty(),
            "Chunk content should not be empty"
         );
      }
   }
}
