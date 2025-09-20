use text_splitter::{ChunkConfig, MarkdownSplitter};

const MARKDOWN_CONTENT: &str = r#"# Main Documentation Title

This is an introduction paragraph that explains the overall purpose of this documentation.
It contains multiple sentences to provide context.

## Getting Started

Before you begin, make sure you have the following prerequisites:
- Rust 1.70 or later
- Cargo installed
- Basic understanding of async programming

### Installation

You can install this package using cargo:

```bash
cargo add mcp-rust-docs-embed
```

Or add it manually to your `Cargo.toml`:

```toml
[dependencies]
mcp-rust-docs-embed = "0.1.0"
```

### Configuration

Configure the system by setting environment variables:

```bash
export API_KEY="your-api-key"
export QDRANT_URL="http://localhost:6333"
```

## Core Features

This section describes the main features of the library.

### Feature 1: Document Processing

The document processor can handle multiple file formats:
- Markdown files (.md)
- Rust source files (.rs)
- Plain text files (.txt)

Here's an example of how to use it:

```rust
use mcp_rust_docs_embed::processor;

fn main() {
    let doc = processor::load_document("example.md");
    let chunks = processor::extract_chunks(doc);
    println!("Found {} chunks", chunks.len());
}
```

### Feature 2: Vector Embeddings

Generate embeddings for your documents using OpenAI or other providers.

The embedding process involves:
1. Chunking the document into semantic sections
2. Converting each chunk to embeddings
3. Storing in a vector database

#### Embedding Models

Supported models include:
- OpenAI text-embedding-3-small
- OpenAI text-embedding-3-large
- Local models via Ollama

#### Usage Example

```rust
async fn create_embeddings(text: &str) -> Result<Vec<f32>> {
    let client = OpenAIClient::new();
    let embedding = client.embed(text).await?;
    Ok(embedding)
}
```

## Advanced Topics

### Performance Optimization

For large documents, consider these optimizations:

1. **Batch Processing**: Process multiple documents in parallel
2. **Caching**: Cache embeddings to avoid recomputation
3. **Chunking Strategy**: Adjust chunk size based on your needs

### Custom Chunking

You can implement custom chunking strategies:

```rust
impl ChunkStrategy for CustomChunker {
    fn chunk(&self, content: &str) -> Vec<Chunk> {
        // Custom implementation
    }
}
```

## API Reference

### `extract_markdown_chunks`

Extracts semantic chunks from markdown content.

**Parameters:**
- `source`: The markdown content as a string

**Returns:**
- `Result<Vec<Chunk>>`: Vector of extracted chunks

### `Chunk` Structure

```rust
pub struct Chunk {
    pub kind: ChunkKind,
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
}
```

## Troubleshooting

### Common Issues

#### Issue 1: Token Limit Exceeded

If you encounter token limit errors, try:
- Reducing the MAX_TOKENS constant
- Splitting large documents before processing

#### Issue 2: Poor Chunk Boundaries

To improve chunk boundaries:
- Ensure markdown headers are properly formatted
- Use consistent indentation
- Avoid extremely long paragraphs

## Contributing

We welcome contributions! Please see our contributing guidelines.

### Development Setup

1. Clone the repository
2. Install dependencies: `cargo build`
3. Run tests: `cargo test`

### Code Style

Follow Rust standard conventions:
- Use `rustfmt` for formatting
- Run `clippy` for linting
- Write unit tests for new features

## License

This project is licensed under the MIT License."#;

#[test]
fn test_extract_markdown_chunks() {
   let splitter = MarkdownSplitter::new(ChunkConfig::new(1000..1500).with_trim(false));

   let chunks = splitter.chunks(MARKDOWN_CONTENT);

   let chunk_vec: Vec<_> = chunks.collect();

   println!("Generated {} chunks", chunk_vec.len());
   println!("\n--- Chunk Details ---");

   for (i, chunk) in chunk_vec.iter().enumerate() {
      let char_count = chunk.len();

      println!("\nChunk {}:", i + 1);
      println!("  Character count: {}", char_count);
      println!("  First 100 chars: {}...", &chunk[..chunk.len().min(100)]);

      // Verify chunk size is within our target range (with some tolerance for boundaries)
      assert!(
         char_count <= 1600,
         "Chunk {} exceeds maximum size: {} chars",
         i + 1,
         char_count
      );
   }

   // Verify we got a reasonable number of chunks
   assert!(!chunk_vec.is_empty(), "No chunks generated");

   // Verify content integrity - all chunks concatenated should equal original
   let reconstructed: String = chunk_vec.join("");
   assert_eq!(
      reconstructed.len(),
      MARKDOWN_CONTENT.len(),
      "Content length mismatch after chunking"
   );
}
