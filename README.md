# Directory Indexer

**Turn your directories into an AI-powered knowledge base.**

[![npm](https://img.shields.io/npm/v/directory-indexer)](https://npmjs.com/package/directory-indexer)
[![Crates.io](https://img.shields.io/crates/v/directory-indexer)](https://crates.io/crates/directory-indexer)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/peteretelej/directory-indexer/workflows/CI/badge.svg)](https://github.com/peteretelej/directory-indexer/actions)

Give AI assistants semantic search across your local files. Find relevant documents based on meaning, not just filenames.

## Quick Start

```bash
# Install via npm
npm install -g directory-indexer

# Index your directories
directory-indexer index ~/Documents ~/work/docs

# Start MCP server for AI assistants
directory-indexer serve
```

**Configure with Claude Desktop:**

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

Now ask Claude: _"Find files similar to my Redis incident reports"_ and it will search your indexed documents semantically.

## Setup

Before using directory-indexer, you need to set up two services:

### 1. Qdrant Vector Database

Choose one option:

**Docker (recommended for most users):**
```bash
docker run -d --name qdrant \
    -p 127.0.0.1:6333:6333 \
    -v qdrant_storage:/qdrant/storage \
    qdrant/qdrant
```

**Alternative:** Install natively from [qdrant.tech](https://qdrant.tech/documentation/guides/installation/)

### 2. Embedding Provider

Choose one option:

**Option A: Ollama (recommended - free, runs locally)**
```bash
# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh  # Linux/macOS
# For Windows: Download from https://ollama.ai

# Pull the embedding model
ollama pull nomic-embed-text
```

**Option B: OpenAI (requires paid API key)**
```bash
export OPENAI_API_KEY="your-api-key-here"
```

### Quick Verification

Test your setup:
```bash
# Check Qdrant
curl http://localhost:6333/collections

# Check Ollama  
curl http://localhost:11434/api/tags
```

If either fails, directory-indexer will show a helpful error with setup guidance.

## Configuration

Directory Indexer uses environment variables for configuration. Set these if your services run on different ports or require API keys:

```bash
# Service endpoints (defaults shown)
export QDRANT_ENDPOINT="http://localhost:6333"
export OLLAMA_ENDPOINT="http://localhost:11434"

# Optional data directory (default: ~/.directory-indexer)
export DIRECTORY_INDEXER_DATA_DIR="/path/to/data"

# Optional Qdrant collection name (default: directory-indexer)
# Note: Setting to "test" enables auto-cleanup for testing
export DIRECTORY_INDEXER_QDRANT_COLLECTION="my-custom-collection"

# Optional API keys
export QDRANT_API_KEY="your-qdrant-key"
export OLLAMA_API_KEY="your-ollama-key"  # if using hosted Ollama
```

**For MCP clients** (like Claude Desktop), configure with environment variables:

```json
{
  "mcpServers": {
    "directory-indexer": {
      "command": "directory-indexer",
      "args": ["serve"],
      "env": {
        "QDRANT_ENDPOINT": "http://localhost:6333",
        "OLLAMA_ENDPOINT": "http://localhost:11434",
        "DIRECTORY_INDEXER_DATA_DIR": "/path/to/data"
      }
    }
  }
}
```

## CLI Usage

```bash
# Index your directories
directory-indexer index ~/Documents ~/work/docs

# Search semantically
directory-indexer search "database timeout errors"

# Find similar files
directory-indexer similar ~/incidents/redis-outage.md

# Get file content
directory-indexer get ~/docs/api-guide.md

# Show status
directory-indexer status
```

## Supported Files

- **Text**: `.md`, `.txt`
- **Code**: `.rs`, `.py`, `.js`, `.ts`, `.go`, `.java`, etc.
- **Data**: `.json`, `.yaml`, `.csv`, `.toml`
- **Config**: `.env`, `.conf`, `.ini`

## Documentation

- **[API Reference](docs/designs/API.md)**: Complete CLI and MCP tool documentation
- **[Contributing](docs/CONTRIBUTING.md)**: Development setup and guidelines
- **[Design](docs/design.md)**: Architecture and technical decisions

## License

MIT
