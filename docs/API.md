# Directory Indexer API Reference

## CLI Commands

All commands support the `-v, --verbose` flag for detailed logging.

### `index`

Index directories for semantic search.

```bash
npx directory-indexer index <paths...> [options]
```

**Arguments:**
- `<paths...>` - One or more directory paths to index

**Options:**
- `-v, --verbose` - Enable verbose logging

**Examples:**
```bash
npx directory-indexer index /home/user/docs
npx directory-indexer index ./projects/api-docs ./work/reports
npx directory-indexer index --verbose /opt/documentation
```

### `search`

Search indexed content semantically.

```bash
npx directory-indexer search <query> [options]
```

**Arguments:**
- `<query>` - Search query text

**Options:**
- `-l, --limit <number>` - Maximum results (default: 10)
- `-v, --verbose` - Enable verbose logging

**Examples:**
```bash
npx directory-indexer search "database timeout errors"
npx directory-indexer search "authentication" --limit 5
```

### `similar`

Find files similar to a given file.

```bash
npx directory-indexer similar <file> [options]
```

**Arguments:**
- `<file>` - Path to reference file

**Options:**
- `-l, --limit <number>` - Maximum results (default: 10)
- `-v, --verbose` - Enable verbose logging

**Examples:**
```bash
npx directory-indexer similar ./docs/api-guide.md
npx directory-indexer similar /home/user/incident.md --limit 5
```

### `get`

Get file content with optional chunk selection.

```bash
npx directory-indexer get <file> [options]
```

**Arguments:**
- `<file>` - Path to file

**Options:**
- `-c, --chunks <range>` - Chunk range (e.g., "2-5")
- `-v, --verbose` - Enable verbose logging

**Examples:**
```bash
npx directory-indexer get ./docs/auth-guide.md
npx directory-indexer get ./deployment.md --chunks 2-4
```

### `serve`

Start MCP server for AI assistant integration.

```bash
npx directory-indexer serve [options]
```

**Options:**
- `-v, --verbose` - Enable verbose logging

**Examples:**
```bash
npx directory-indexer serve
npx directory-indexer serve --verbose
```

### `status`

Show indexing status and statistics.

```bash
npx directory-indexer status [options]
```

**Options:**
- `-v, --verbose` - Show detailed error information

**Examples:**
```bash
npx directory-indexer status
npx directory-indexer status --verbose
```

## MCP Tools

Available when running as MCP server (`npx directory-indexer serve`).

### `index`

Index directories for semantic search.

**Input Schema:**
```json
{
  "directory_path": "/home/user/docs,/opt/projects"
}
```

**Response:**
```
Indexed 145 files, skipped 12 files, 0 errors
```

### `search`

Search indexed content semantically.

**Input Schema:**
```json
{
  "query": "database timeout errors",
  "limit": 10
}
```

**Response:**
```json
[
  {
    "filePath": "/work/incidents/redis-timeout.md",
    "score": 0.89,
    "content": "Redis connection pool exhausted during peak traffic..."
  }
]
```

### `similar_files`

Find files similar to a given file.

**Input Schema:**
```json
{
  "file_path": "/work/incidents/database-outage.md",
  "limit": 10
}
```

**Response:**
```json
[
  {
    "filePath": "/work/incidents/redis-timeout.md",
    "score": 0.91
  }
]
```

### `get_content`

Get file content with optional chunk selection.

**Input Schema:**
```json
{
  "file_path": "/home/user/docs/api-guide.md",
  "chunks": "2-5"
}
```

**Response:**
```
# API Authentication Guide

This document covers authentication patterns...
```

### `server_info`

Get server information and status.

**Input Schema:**
```json
{}
```

**Response:**
```json
{
  "name": "directory-indexer",
  "version": "0.0.10",
  "status": {
    "directoriesIndexed": 3,
    "filesIndexed": 1247,
    "chunksIndexed": 3891,
    "databaseSize": "15.2 MB",
    "lastIndexed": "2025-01-15T10:30:00Z",
    "errors": []
  }
}
```

## Error Handling

All commands use exit code 1 for errors. Error messages are written to stderr.

**Common Error Types:**
- Configuration errors (missing services)
- File access errors (permissions, not found)
- Network errors (Qdrant/Ollama unavailable)
- Processing errors (unsupported file types)

**Example Error Output:**
```
Error indexing directories: Failed to connect to Qdrant at http://localhost:6333
Check that Qdrant is running: docker run -p 6333:6333 qdrant/qdrant
```

## Configuration

See [Environment Variables](./design.md#environment-variables) for complete configuration options.

## Supported File Types

- **Text**: `.md`, `.txt`
- **Code**: `.rs`, `.py`, `.js`, `.ts`, `.go`, `.java`, etc.
- **Data**: `.json`, `.yaml`, `.csv`, `.toml`
- **Config**: `.env`, `.conf`, `.ini`