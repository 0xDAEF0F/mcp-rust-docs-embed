# embed-anything-rs

Rust crate documentation embedding and semantic search service.

## Quick Start

```bash
# Generate and embed docs
cargo run --example cli -- gen-docs <crate_name> --version <version>
cargo run --example cli -- embed <crate_name> --version <version>

# Query
cargo run --example cli -- query "your search query" --crate <crate_name>

# MCP Server
cargo run  # Starts at http://127.0.0.1:8000/sse
```

## API

Primary interfaces subject to change:
- `DocumentationService::generate_docs()` - Cargo doc generation
- `QueryService::embed_crate()` - Vector embedding pipeline  
- `QueryService::query_embeddings()` - Semantic search

## Dependencies

- Qdrant for vector storage
- embed_anything for embeddings
- MCP server for integration

## Environment

Configure via `.env`:
```
QDRANT_URL=http://localhost:6334
QDRANT_API_KEY=your_key
```