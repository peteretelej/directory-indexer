# Directory Indexer Design

## Overview

Self-hosted semantic search for local files. Cross-platform Rust application that generates vector embeddings of file content and provides search via CLI and MCP server integration.

**Dependencies**: Minimal essential crates for core functionality, no heavy frameworks.

## Related Documentation

- [System Flow Diagrams](designs/flows.md) - Visual flow charts for all major operations
- [API Reference](designs/API.md) - Complete CLI and MCP API documentation

## Architecture

- **CLI**: Single binary with subcommands
- **Storage**: SQLite (metadata) + Qdrant (vectors via REST API)  
- **Embeddings**: Ollama/OpenAI via HTTP
- **MCP Server**: Model Context Protocol for AI assistant integration

## Storage Design

### SQLite (Source of Truth)
```sql
-- Directories table
CREATE TABLE directories (
    id INTEGER PRIMARY KEY,
    path TEXT UNIQUE NOT NULL,
    status TEXT DEFAULT 'pending',
    indexed_at INTEGER DEFAULT 0
);

-- Files table  
CREATE TABLE files (
    id INTEGER PRIMARY KEY,
    path TEXT UNIQUE NOT NULL,
    size INTEGER NOT NULL,
    modified_time INTEGER NOT NULL,
    hash TEXT NOT NULL,
    parent_dirs TEXT NOT NULL, -- JSON array
    chunks_json TEXT,          -- JSON array of chunks
    errors_json TEXT           -- JSON array of errors
);
```

### Qdrant Vector Store
- **Collections**: Single `directory-indexer` collection
- **Points**: UUID IDs with minimal payload (file_path, chunk_id, parent_directories)
- **API**: Direct REST calls, no SDK dependency

## API Reference

### CLI Commands

```bash
# Index directories
# Linux/macOS
directory-indexer index /home/user/projects/docs /mnt/work/runbooks
# Windows
directory-indexer index "C:\work\documentation" "D:\projects\api-docs"

# Search content
directory-indexer search "database timeout" [--path /mnt/work/docs] [--limit 10]

# Find similar files  
directory-indexer similar /home/user/incidents/outage.md [--limit 10]

# Get file content
directory-indexer get /home/user/docs/api-guide.md [--chunks 2-5]

# Start MCP server
directory-indexer serve

# Show status
directory-indexer status [--format json|text]
```

### MCP Tools

**index(directory_path: string)** - Index directories (comma-separated paths)
**search(query: string, directory_path?: string, limit?: number)** - Semantic search  
**similar_files(file_path: string, limit?: number)** - Find similar files  
**get_content(file_path: string, chunks?: string)** - Retrieve file content  
**server_info()** - Get server status and configuration

## Configuration

Directory Indexer uses environment variables for configuration, following 12-factor app principles. Default values are used when environment variables are not set.

### Environment Variables

```bash
# Service Endpoints
QDRANT_ENDPOINT="http://localhost:6333"     # Qdrant vector database
OLLAMA_ENDPOINT="http://localhost:11434"    # Ollama embedding service

# Database Path  
DIRECTORY_INDEXER_DATA_DIR="/opt/directory-indexer-data" # Data directory (contains data.db)

# Optional API Keys
QDRANT_API_KEY="your-api-key"               # For Qdrant Cloud or secured instances
OLLAMA_API_KEY="your-api-key"               # For hosted Ollama services
```

### Default Configuration Values

```rust
// Storage
sqlite_path: ~/.directory-indexer/data.db
qdrant.collection: "directory-indexer"

// Embedding
provider: "ollama"
model: "nomic-embed-text"

// Indexing  
chunk_size: 512
overlap: 50
max_file_size: 10485760 (10MB)
ignore_patterns: [".git", "node_modules", "target"]
concurrency: 4
```

### MCP Client Configuration

For AI assistants like Claude Desktop, set environment variables in the MCP server configuration:

```json
{
  "mcpServers": {
    "directory-indexer": {
      "command": "directory-indexer",
      "args": ["serve"],
      "env": {
        "QDRANT_ENDPOINT": "http://localhost:6333",
        "OLLAMA_ENDPOINT": "http://localhost:11434",
        "DIRECTORY_INDEXER_DATA_DIR": "/opt/directory-indexer-data"
      }
    }
  }
}
```

## Indexing Process

1. Walk directory tree, filter by ignore patterns
2. Extract file metadata (size, mtime, hash)
3. Compare with SQLite to detect changes
4. Read and chunk file content
5. Generate embeddings via HTTP API
6. Store metadata in SQLite, vectors in Qdrant
7. Update directory status

## Search Process

1. Generate query embedding
2. Vector search in Qdrant via REST API
3. Fetch file metadata from SQLite
4. Rank by similarity score
5. Return results with content previews

## Development

### Quick Start
```bash
# Start dev services (Qdrant on 6333, Ollama on 11434)
./scripts/start-dev-services.sh

# Run tests
cargo test --test connectivity_tests

# Stop services
./scripts/stop-dev-services.sh
```

### File Types Supported
- Text: .md, .txt, .rst
- Code: .rs, .py, .js, .ts, .go, .java, .cpp, .c
- Data: .json, .yaml, .toml, .csv
- Config: .env, .conf, .ini
- Web: .html, .xml

## Error Handling

- **Partial failures**: Continue processing, log errors in SQLite
- **Resource failures**: Fail fast with clear error messages  
- **File errors**: Skip unreadable files, continue with others

## Performance

- **Concurrency**: Configurable file processing (default: 4)
- **Memory**: Stream file processing, batch database operations
- **Updates**: Only reprocess changed files (hash comparison)

## Implementation Status

### ✅ Fully Implemented
- **CLI Commands**: `index`, `search`, `similar`, `get`, `serve`, `status`
- **MCP Server**: JSON-RPC 2.0 protocol with stdio communication
- **MCP Tools**: `index`, `search`, `similar_files`, `get_content`, `server_info`
- **Storage**: SQLite metadata store + Qdrant vector store integration
- **Embeddings**: Ollama and OpenAI provider support
- **File Processing**: Multi-format support with chunking and error handling

### Key Features
- **Graceful Fallback**: Commands work with both indexed and unindexed files
- **Chunk Support**: Stored chunks for indexed files, line-based chunking for unindexed files
- **Semantic Search**: Vector similarity search with metadata enrichment
- **Error Recovery**: Partial failure handling with detailed error logging