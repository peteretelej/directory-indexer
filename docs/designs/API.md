# Directory Indexer API Reference

## CLI Commands

### `index`

Index directories for semantic search.

```bash
directory-indexer index <paths...> [options]
```

**Arguments:**

- `<paths...>` - One or more directory paths to index

**Global Options:**

- `-v, --verbose` - Enable verbose logging
- `-c, --config <FILE>` - Custom config file path

**Examples:**

```bash
directory-indexer index ~/Documents
directory-indexer index ~/work/docs ~/personal/notes
directory-indexer -v index ~/Documents
```

---

### `search`

Search indexed content semantically.

```bash
directory-indexer search <query> [options]
```

**Arguments:**

- `<query>` - Search query text

**Options:**

- `-p, --path <PATH>` - Scope search to specific directory
- `-l, --limit <LIMIT>` - Maximum results to return (default: 10)

**Examples:**

```bash
directory-indexer search "Redis connection timeout"
directory-indexer search "error handling" --path ~/work/docs
directory-indexer search "authentication" --limit 5
```

---

### `similar`

Find files similar to a given file.

> **⚠️ Status**: Placeholder implementation - warns "not yet implemented"

```bash
directory-indexer similar <file> [options]
```

**Arguments:**

- `<file>` - Path to reference file

**Options:**

- `-l, --limit <LIMIT>` - Maximum similar files to return (default: 10)

**Examples:**

```bash
directory-indexer similar ~/work/incidents/database-outage.md
directory-indexer similar ~/docs/api-guide.md --limit 5
```

---

### `get`

Retrieve file content with optional chunk selection.

> **⚠️ Status**: Placeholder implementation - warns "not yet implemented"

```bash
directory-indexer get <file> [options]
```

**Arguments:**

- `<file>` - Path to file

**Options:**

- `-c, --chunks <RANGE>` - Chunk range (e.g., "2-5", "3")

**Examples:**

```bash
directory-indexer get ~/work/docs/api-guide.md
directory-indexer get ~/work/docs/deployment.md --chunks 2-4
```

---

### `serve`

Start MCP (Model Context Protocol) server.

```bash
directory-indexer serve [options]
```

**Examples:**

```bash
directory-indexer serve
directory-indexer -v serve
```

---

### `status`

Show indexing status and statistics.

```bash
directory-indexer status [options]
```

**Options:**

- `-f, --format <FORMAT>` - Output format: `text` (default) or `json`

**Examples:**

```bash
directory-indexer status
directory-indexer status --format json
```

## MCP Tools

When running as MCP server (`directory-indexer serve`), these tools are available to AI assistants:

### `index`

Index directories for semantic search.

**Input Schema:**

```json
{
  "directory_paths": ["~/Documents", "~/work/docs"]
}
```

**Response:**

```json
{
  "success": true,
  "message": "Successfully indexed 2 directories",
  "directories_processed": 2,
  "files_processed": 1234,
  "chunks_created": 3456
}
```

---

### `search`

Search indexed content semantically.

**Input Schema:**

```json
{
  "query": "Redis connection timeout",
  "directory_path": "~/work/docs", // optional
  "limit": 10 // optional, default: 10
}
```

**Response:**

```json
{
  "query": "Redis connection timeout",
  "total_results": 8,
  "results": [
    {
      "file_path": "/work/incidents/redis-timeout-2024.md",
      "score": 0.89,
      "snippet": "Redis connection pool exhausted during peak traffic...",
      "chunk_id": 2
    }
  ]
}
```

---

### `similar_files`

Find files similar to a given file.

> **⚠️ Status**: Placeholder implementation - returns "not yet implemented" message

**Input Schema:**

```json
{
  "file_path": "~/work/incidents/database-outage.md",
  "limit": 10 // optional, default: 10
}
```

**Response:**

```json
{
  "reference_file": "/work/incidents/database-outage.md",
  "similar_files": [
    {
      "file_path": "/work/incidents/redis-timeout-2024.md",
      "score": 0.91,
      "snippet": "Similar incident with timeout issues..."
    }
  ]
}
```

---

### `get_content`

Retrieve file content with optional chunk selection.

> **⚠️ Status**: Placeholder implementation - returns "not yet implemented" message

**Input Schema:**

```json
{
  "file_path": "~/work/docs/api-guide.md",
  "chunks": "2-5" // optional
}
```

**Response:**

```json
{
  "file_path": "/work/docs/api-guide.md",
  "content": "# API Authentication Guide\\n\\nThis document...",
  "chunks_requested": "2-5",
  "total_chunks": 8
}
```

---

### `server_info`

Get server information and statistics.

**Input Schema:**

```json
{}
```

**Response:**

```json
{
  "name": "directory-indexer",
  "version": "0.1.0",
  "description": "AI-powered directory indexing with semantic search",
  "stats": {
    "indexed_directories": 3,
    "indexed_files": 2268,
    "total_chunks": 5432,
    "database_size_mb": 45.2
  },
  "config": {
    "embedding_provider": "ollama",
    "embedding_model": "nomic-embed-text",
    "chunk_size": 512,
    "overlap": 50
  }
}
```

## Configuration

### Config File Location

`~/.directory-indexer/config.json`

### Config Schema

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
    "endpoint": "http://localhost:11434",
    "api_key": null
  },
  "indexing": {
    "chunk_size": 512,
    "overlap": 50,
    "max_file_size": 10485760,
    "ignore_patterns": [".git", "node_modules", "target"],
    "concurrency": 4
  },
  "monitoring": {
    "file_watching": false,
    "batch_size": 100
  }
}
```

### Configuration Options

#### Storage

- `sqlite_path` - Path to SQLite database file
- `qdrant.endpoint` - Qdrant server URL
- `qdrant.collection` - Collection name for embeddings

#### Embedding

- `provider` - Embedding provider: `ollama`, `openai`
- `model` - Model name (e.g., `nomic-embed-text`, `text-embedding-ada-002`)
- `endpoint` - Provider API endpoint
- `api_key` - API key (for remote providers)

#### Indexing

- `chunk_size` - Text chunk size in tokens
- `overlap` - Overlap between chunks in tokens
- `max_file_size` - Maximum file size to process (bytes)
- `ignore_patterns` - File/directory patterns to ignore
- `concurrency` - Number of files to process concurrently

#### Monitoring

- `file_watching` - Enable file system monitoring (future feature)
- `batch_size` - Batch size for database operations

## Supported File Types

### Text Files

- `.md`, `.txt`, `.rst`, `.org`

### Code Files

- `.rs`, `.py`, `.js`, `.ts`, `.go`, `.java`, `.cpp`, `.c`, `.h`

### Data Files

- `.json`, `.yaml`, `.yml`, `.toml`, `.csv`

### Config Files

- `.env`, `.conf`, `.ini`, `.cfg`

### Web Files

- `.html`, `.xml`

## Error Handling

### Common Error Types

- **Config Error** - Invalid configuration file
- **Storage Error** - Database or vector store connection issues
- **Embedding Error** - Embedding provider API failures
- **File Processing Error** - File reading or parsing issues
- **Network Error** - Connection timeouts or network issues

### Error Response Format

```json
{
  "error": {
    "type": "EmbeddingError",
    "message": "Failed to connect to Ollama at http://localhost:11434",
    "details": "Connection refused"
  }
}
```

## Exit Codes

- `0` - Success
- `1` - General error
- `2` - Configuration error
- `3` - Storage error
- `4` - Network error
- `5` - File processing error
