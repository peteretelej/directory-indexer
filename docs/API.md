# Directory Indexer API Reference

## CLI Commands

### `index`

Index directories for semantic search.

```bash
directory-indexer index <paths...>
```

**Arguments:**
- `<paths...>` - One or more directory paths to index

**Examples:**
```bash
# Index a single directory
directory-indexer index ~/Documents

# Index multiple directories
directory-indexer index ~/work/docs ~/personal/notes ~/projects

# Index with verbose logging
directory-indexer -v index ~/Documents
```

**Output:**
```
✅ Indexing 2 directories
  📁 /home/user/Documents
  📁 /home/user/work/docs
📊 Processed 1,234 files in 5.2s
🧮 Created 3,456 chunks
💾 Stored 3,456 embeddings
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
# Basic search
directory-indexer search "Redis connection timeout"

# Search in specific directory
directory-indexer search "error handling" --path ~/work/docs

# Limit results
directory-indexer search "authentication" --limit 5
```

**Output:**
```
🔍 Found 8 results for "Redis connection timeout"

📄 /work/incidents/redis-timeout-2024.md (score: 0.89)
   Redis connection pool exhausted during peak traffic...

📄 /docs/troubleshooting/redis.md (score: 0.84)
   Common Redis timeout issues and solutions...

📄 /runbooks/redis-maintenance.md (score: 0.78)
   Redis maintenance procedures and monitoring...
```

---

### `similar`

Find files similar to a given file.

```bash
directory-indexer similar <file> [options]
```

**Arguments:**
- `<file>` - Path to reference file

**Options:**
- `-l, --limit <LIMIT>` - Maximum similar files to return (default: 10)

**Examples:**
```bash
# Find similar files
directory-indexer similar ~/work/incidents/database-outage.md

# Limit results
directory-indexer similar ~/docs/api-guide.md --limit 5
```

**Output:**
```
🔗 Files similar to /work/incidents/database-outage.md

📄 /work/incidents/redis-timeout-2024.md (score: 0.91)
📄 /work/incidents/postgres-lock-2023.md (score: 0.87)
📄 /work/runbooks/database-recovery.md (score: 0.82)
```

---

### `get`

Retrieve file content with optional chunk selection.

```bash
directory-indexer get <file> [options]
```

**Arguments:**
- `<file>` - Path to file

**Options:**
- `-c, --chunks <RANGE>` - Chunk range (e.g., "2-5", "1-3")

**Examples:**
```bash
# Get full file content
directory-indexer get ~/work/docs/api-guide.md

# Get specific chunks
directory-indexer get ~/work/docs/deployment.md --chunks 2-4
```

**Output:**
```
📄 /work/docs/api-guide.md

# API Authentication Guide

This document describes the authentication...

[Chunks 1-3 of 8 total chunks]
```

---

### `serve`

Start MCP (Model Context Protocol) server.

```bash
directory-indexer serve
```

**Examples:**
```bash
# Start MCP server
directory-indexer serve

# Start with verbose logging
directory-indexer -v serve
```

**Output:**
```
🚀 Starting MCP server...
🔌 Ready to accept MCP connections
📊 Indexed directories: 3
📄 Indexed files: 1,234
🧮 Total chunks: 3,456

Press Ctrl+C to stop
```

---

### `status`

Show indexing status and statistics.

```bash
directory-indexer status
```

**Examples:**
```bash
directory-indexer status
```

**Output:**
```
📊 Directory Indexer Status

🗃️  Indexed Directories: 3
  📁 /home/user/Documents (1,023 files)
  📁 /home/user/work/docs (456 files)
  📁 /home/user/projects (789 files)

📄 Total Files: 2,268
🧮 Total Chunks: 5,432
💾 Database Size: 45.2 MB
🔗 Vector Store: 5,432 embeddings

⚙️  Configuration:
  🤖 Provider: ollama (nomic-embed-text)
  📐 Chunk Size: 512 tokens
  🔄 Overlap: 50 tokens
```

---

## MCP Tools

When running as an MCP server (`directory-indexer serve`), the following tools are available to AI assistants:

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
  "directory_path": "~/work/docs",  // optional
  "limit": 10                       // optional, default: 10
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

**Input Schema:**
```json
{
  "file_path": "~/work/incidents/database-outage.md",
  "limit": 10  // optional, default: 10
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

**Input Schema:**
```json
{
  "file_path": "~/work/docs/api-guide.md",
  "chunks": "2-5"  // optional
}
```

**Response:**
```json
{
  "file_path": "/work/docs/api-guide.md",
  "content": "# API Authentication Guide\n\nThis document...",
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

---

## Configuration

Configuration file location: `~/.directory-indexer/config.json`

### Example Configuration

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
  },
  "monitoring": {
    "file_watching": false,
    "batch_size": 100
  }
}
```

### Configuration Options

#### Storage
- `sqlite_path`: Path to SQLite database file
- `qdrant.endpoint`: Qdrant server URL
- `qdrant.collection`: Collection name for embeddings

#### Embedding
- `provider`: Embedding provider ("ollama", "openai", "openrouter")
- `model`: Model name (e.g., "nomic-embed-text", "text-embedding-ada-002")
- `endpoint`: Provider API endpoint
- `api_key`: API key (only for remote providers)

#### Indexing
- `chunk_size`: Text chunk size in tokens
- `overlap`: Overlap between chunks in tokens
- `max_file_size`: Maximum file size to process (bytes)
- `ignore_patterns`: File/directory patterns to ignore
- `concurrency`: Number of files to process concurrently

#### Monitoring
- `file_watching`: Enable file system monitoring (future feature)
- `batch_size`: Batch size for database operations

---

## Error Handling

### Common Error Codes

- **Config Error**: Invalid configuration file
- **Storage Error**: Database or vector store connection issues
- **Embedding Error**: Embedding provider API failures
- **File Processing Error**: File reading or parsing issues
- **Network Error**: Connection timeouts or network issues

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

---

## Supported File Types

### Currently Supported
- **Text**: `.md`, `.txt`, `.rst`, `.org`
- **Code**: `.rs`, `.py`, `.js`, `.ts`, `.go`, `.java`, `.cpp`, `.c`, `.h`
- **Data**: `.json`, `.yaml`, `.yml`, `.toml`, `.csv`
- **Config**: `.env`, `.conf`, `.ini`, `.cfg`
- **Web**: `.html`, `.xml`

### Planned Support
- **Documents**: `.pdf`, `.docx`, `.pptx`
- **Spreadsheets**: `.xlsx`, `.ods`
- **Archives**: `.zip`, `.tar.gz` (extract and index contents)