# mcp-rust-docs-embed

An MCP (Model Context Protocol) server that provides semantic search capabilities for Rust crate documentation through vector embeddings.

## Overview

This MCP server enables AI assistants to search and understand Rust crate documentation by:

- Generating JSON-formatted documentation using `cargo doc`
- Creating vector embeddings of documentation content using OpenAI
- Storing embeddings in Qdrant for efficient semantic search
- Providing MCP tools for embedding and querying documentation

## Features

- **Automatic Documentation Generation**: Builds Rust documentation in JSON format using cargo's nightly toolchain
- **Semantic Search**: Query documentation using natural language through vector embeddings
- **Version Management**: Each crate version is stored separately for precise version-specific searches
- **Feature Support**: Validates requested features against available crate features and tracks which features were used for embeddings
- **Async Operations**: Long-running embedding tasks are handled asynchronously with status tracking

## Prerequisites

- Rust nightly toolchain (for JSON documentation output)
- Qdrant vector database instance
- OpenAI API key for embeddings

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/mcp-rust-docs-embed
cd mcp-rust-docs-embed

# Build the project
cargo build --release
```

## Configuration

Create a `.env` file with the following variables:

```env
# Required
QDRANT_URL=http://localhost:6334
OPENAI_API_KEY=your_openai_api_key

# Optional
QDRANT_API_KEY=your_qdrant_api_key
PORT=8080  # Default: 8080
```

## Usage

### Starting the MCP Server

```bash
cargo run
```

The server will start on `http://127.0.0.1:8080/sse` (or your configured port).

### Using with Claude Code or MCP Clients

To use this server with Claude Code or other MCP clients, add it to your MCP configuration:

#### Production Server

```json
{
  "rust-docs": {
    "type": "sse",
    "url": "https://mcp-rust-docs-embed-production.up.railway.app/sse"
  }
}
```

#### Local Development

For local development, replace the URL with your local server:

```json
{
  "rust-docs": {
    "type": "sse",
    "url": "http://127.0.0.1:8080/sse"
  }
}
```

### Available MCP Tools

#### 1. `embed_crate`

Generate and embed documentation for a Rust crate.

Parameters:

- `crate_name` (required): Name of the crate to document
- `version` (optional): Version to embed (defaults to `*`, i.e., latest)
- `features` (optional): List of features to enable (validated against available features)

Example:

```json
{
  "crate_name": "tokio",
  "version": "1.40.0",
  "features": ["full"]
}
```

Note: The server validates requested features against available crate features and prevents re-embedding with different features unless the existing collection is deleted first.

#### 2. `query_embeddings`

Search embedded documentation using natural language.

Parameters:

- `query` (required): Natural language search query
- `crate_name` (required): Crate to search in
- `version` (optional): Version to search (defaults to latest)
- `limit` (optional): Number of results (default: 10)

Example:

```json
{
  "query": "how to create an async tcp server",
  "crate_name": "tokio",
  "limit": 5
}
```

#### 3. `query_embed_status`

Check the status of an ongoing embedding operation.

Parameters:

- `operation_id` (required): ID returned by `embed_crate`

#### 4. `list_embedded_crates`

List all crates and versions that have been embedded, including their features, embedding timestamp, and document count.

#### 5. `query_crate_features`

Query available features for a specific crate and version.

Parameters:

- `crate_name` (required): Name of the crate to query features for
- `version` (optional): Version to check (defaults to `*`, i.e., latest)

Example:

```json
{
  "crate_name": "tokio",
  "version": "1.40.0"
}
```

## Architecture

### Documentation Pipeline

1. **Documentation Generation** (`docs_builder.rs`)

   - Creates a temporary Cargo project
   - Adds target crate as dependency with user-specified features (validated against available features)
   - Validates features to prevent mutually exclusive conflicts
   - Runs `cargo +nightly doc --output-format=json`

2. **JSON Processing** (`doc_loader.rs`, `json_types.rs`)

   - Parses Rust's JSON documentation format
   - Extracts structured information about items, functions, types, etc.

3. **Embedding Generation** (`documentation.rs`)

   - Chunks documentation content appropriately
   - Generates embeddings using OpenAI's text-embedding-3-small model
   - Batches embeddings for efficiency

4. **Storage** (`data_store.rs`)
   - Stores embeddings in Qdrant collections
   - Collection naming: `{crate_name}_v{version}` (normalized)
   - Includes source content for retrieval
   - Stores metadata including features used, embedding timestamp, and document count

### Query System

The `QueryService` (`query.rs`) handles:

- Converting queries to embeddings
- Searching Qdrant for similar documentation
- Returning relevant documentation snippets with similarity scores

## Development

### Key Dependencies

- `rmcp` - Rust MCP SDK
- `qdrant-client` - Vector database client
- `cargo` - Programmatic cargo operations
- `async-openai` - OpenAI API client
- `axum` - Web framework for SSE server
- `tokio` - Async runtime

## Limitations

- Requires Rust nightly for JSON documentation output
- OpenAI API costs for embedding generation
- Storage requirements grow with number of embedded crates
- Feature validation prevents invalid feature combinations
- Cannot re-embed a crate with different features without deleting the existing collection first
