# Directory Indexer Design

## Overview

Rust-based semantic search system for local directories. Generates vector embeddings of file content and provides search via CLI and MCP server.

**Dependencies**: Keep minimal - only essential crates for core functionality.

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
directory-indexer index <paths...>

# Search content
directory-indexer search <query> [--path PATH] [--limit N]

# Find similar files  
directory-indexer similar <file> [--limit N]

# Get file content
directory-indexer get <file> [--chunks RANGE]

# Start MCP server
directory-indexer serve

# Show status
directory-indexer status [--format json|text]
```

### MCP Tools

**index(directory_paths: string[])** - Index directories  
**search(query: string, directory_path?: string, limit?: number)** - Semantic search  
**similar_files(file_path: string, limit?: number)** - Find similar files  
**get_content(file_path: string, chunks?: string)** - Retrieve file content  
**server_info()** - Get server status and configuration

## Configuration

Location: `~/.directory-indexer/config.json`

```json
{
  "storage": {
    "sqlite_path": "~/.directory-indexer/data.db",
    "qdrant": {
      "endpoint": "http://localhost:6333",
      "collection": "directory-indexer"
    }
  },
  "embedding": {
    "provider": "ollama",
    "model": "nomic-embed-text", 
    "endpoint": "http://localhost:11434"
  },
  "indexing": {
    "chunk_size": 512,
    "overlap": 50,
    "max_file_size": 10485760,
    "ignore_patterns": [".git", "node_modules", "target"],
    "concurrency": 4
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
# Start dev services (Qdrant on 6335, Ollama on 11435)
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