# Directory Indexer

**Turn your directories into an AI-powered knowledge base for your team.**

Directory Indexer is an MCP server that indexes local directories with semantic search. Give AI assistants the ability to find and read relevant files based on content similarity, not just filenames.

## Problem

You have thousands of files across directories:

- Incident response docs and runbooks
- Code examples and architecture decisions
- Meeting notes and project documentation
- Configuration files and deployment guides

Finding relevant files requires knowing exactly what to search for:

- Filename search needs exact words from the filename
- Text search needs specific terms that appear in the content
- No way to find conceptually related files or similar solutions to problems

## Solution

[directory-indexer](https://github.com/peteretelej/directory-indexer) creates vector embeddings of your files and provides semantic search through MCP tools. AI assistants can find relevant documents based on meaning and context.

**Example workflow:**

1. Index your directories: `directory-indexer index ~/work/docs ~/incidents`
2. Configure with Claude Desktop or any MCP client
3. Ask: _"Find incidents similar to this Redis timeout error"_
4. AI searches embeddings, finds relevant files, reads content, provides context

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   AI Assistant  │◄──►│  MCP Server      │◄──►│  Vector Store   │
│  (Claude, etc)  │    │  (Rust)          │    │  (Qdrant)       │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │
                              ▼
                       ┌──────────────────┐
                       │  File System     │
                       │  Monitor         │
                       └──────────────────┘
```

**Core components:**

- **MCP Server**: Implements Model Context Protocol, provides search tools
- **File Processor**: Converts various formats (PDF, DOCX, etc.) to text (initially unimplemented)
- **Indexing Engine**: Extracts content, creates embeddings, handles updates
- **Vector Storage**: Qdrant for fast similarity search
- **File Monitor**: Watches for changes, incremental updates

## MCP Tools

- `index(directory_path)` - Index all files in directory
- `search(query, directory_path)` - Semantic search across indexed files
- `similar_files(file_path, limit)` - Find files similar to given file
- `get_content(file_path, chunks?)` - Read file with optional chunk selection
- `server_info()` - Show indexed directories and stats

## CLI Commands

- `directory-indexer index <paths...>` - Index directories
- `directory-indexer search <query> [path]` - Search indexed content
- `directory-indexer similar <file> [--limit N]` - Find similar files
- `directory-indexer get <file> [--chunks N-M]` - Get file content
- `directory-indexer serve` - Start MCP server
- `directory-indexer status` - Show indexing status

## Installation

```bash
# Install via npm
npm install -g directory-indexer

# Or run directly
npx directory-indexer@latest
```

**Requirements:**

- Qdrant (local or remote)
- Embedding provider:
  - Local: Ollama (free)
  - Remote: OpenAI, Voyage AI, OpenRouter, or compatible API (requires API key)

## Usage

### 1. Index directories

```bash
directory-indexer index ~/work/incidents ~/docs/runbooks
```

### 2. Search from CLI

```bash
# Semantic search
directory-indexer search "Redis connection timeout" ~/work/incidents

# Find similar files
directory-indexer similar ~/work/incidents/icm-2024-0123.md

# Get file content with chunks
directory-indexer get ~/work/runbooks/deployment.md --chunks 2-5
```

### 3. Use with MCP clients

Add to Cline, Claude Desktop, or Cursor config:

```json
{
  "mcpServers": {
    "directory-indexer": {
      "command": "directory-indexer",
      "args": ["serve"]
    }
  }
}
```

### 4. Interactive with AI

- _"Find documentation about our Redis setup"_
- _"What incidents have we had with database timeouts?"_
- _"Show me examples of error handling in our Go services"_

## Configuration

Default config at `~/.directory-indexer/config.json`:

```json
{
  "embedding": {
    "provider": "ollama",
    "model": "nomic-embed-text",
    "endpoint": "http://localhost:11434"
  },
  "storage": {
    "type": "qdrant",
    "endpoint": "http://localhost:6333",
    "collection": "documents"
  },
  "indexing": {
    "chunk_size": 512,
    "overlap": 50,
    "max_file_size": 10485760,
    "ignore_patterns": [".git", "node_modules", "target"]
  },
  "monitoring": {
    "enabled": true,
    "batch_size": 100
  }
}
```

**API Key handling:**

- Only include `api_key` field when using remote providers (OpenAI, Voyage AI, OpenRouter)
- Omit the field entirely for local Ollama
- Store API keys securely with proper file permissions (`600`)

## Supported File Types

- **Text**: `.md`, `.txt`, `.rst`, `.org`
- **Code**: Most programming languages with syntax awareness
- **Data**: `.json`, `.yaml`, `.csv`, `.toml`
- **Config**: `.env`, `.conf`, `.ini`
- **Web**: `.html`, `.xml`

## Planned Support

- **Documents**: `.pdf`, `.docx`, `.pptx` (requires file conversion preprocessing)
- **Spreadsheets**: `.xlsx`, `.ods`
- **Archives**: Extract and index contents of `.zip`, `.tar.gz`

## Development

**Tech stack:**

- Rust (tokio async runtime)
- Qdrant (vector database)
- Ollama or OpenAI-compatible embedding API
- Model Context Protocol
- Platform-native file watching

**Architecture decisions:**

- Rust for performance and memory safety
- Qdrant for production-ready vector search
- MCP for standardized AI assistant integration
- Local-first with optional remote embedding APIs

## License

MIT
