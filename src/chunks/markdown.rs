use super::types::{Chunk, ChunkKind};
use anyhow::Result;
use text_splitter::{ChunkConfig, MarkdownSplitter};
use tracing::trace;

/// Splits Markdown documents into semantic sections preserving headings and content
/// relationships for optimal documentation search and retrieval
pub fn extract_markdown_chunks(source: &str) -> Result<Vec<Chunk>> {
   let start = std::time::Instant::now();
   trace!(
      "Starting markdown chunk extraction for {} chars of source",
      source.len()
   );

   let splitter = MarkdownSplitter::new(ChunkConfig::new(1000..1500).with_trim(false));
   let mut chunks = Vec::new();

   for (i, chunk_text) in splitter.chunks(source).enumerate() {
      let byte_offset = source.find(chunk_text).unwrap_or(0);
      let start_line = source[..byte_offset].matches('\n').count() + 1;
      let end_line = start_line + chunk_text.matches('\n').count();

      chunks.push(Chunk {
         kind: ChunkKind::MarkdownSection,
         start_line,
         end_line,
         content: chunk_text.to_string(),
      });

      trace!(
         "Created markdown chunk {} with {} chars at lines {}-{}",
         i,
         chunk_text.len(),
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
