# Directory Indexer

**Turn your directories into an AI-powered knowledge base.**

[![npm](https://img.shields.io/npm/v/directory-indexer)](https://npmjs.com/package/directory-indexer)
[![Crates.io](https://img.shields.io/crates/v/directory-indexer)](https://crates.io/crates/directory-indexer)
[![codecov](https://codecov.io/gh/peteretelej/directory-indexer/graph/badge.svg?token=j6aBBpfqkN)](https://codecov.io/gh/peteretelej/directory-indexer)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/peteretelej/directory-indexer/workflows/CI/badge.svg)](https://github.com/peteretelej/directory-indexer/actions)

Self-hosted semantic search for local files. Enable AI assistants to search your documents using vector embeddings and MCP integration.

## Setup

Directory Indexer runs locally on your machine or server. It uses an embedding provider (such as Ollama) to create vector embeddings of your files and stores them in a Qdrant vector database for fast semantic search. Both services can run remotely if needed.

Setup requires two services:

### 1. Qdrant Vector Database

Choose one option:

**Docker (recommended for most users):**

```bash
docker run -d --name qdrant \
    -p 127.0.0.1:6333:6333 \
    -v qdrant_storage:/qdrant/storage \
    qdrant/qdrant
```

- This option requires [Docker](https://docs.docker.com/get-docker/)
- Runs Qdrant on docker container, uses a named volume `qdrant_storage` for persistent storage.

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

- You can also [run Ollama via Docker](https://ollama.com/blog/ollama-is-now-available-as-an-official-docker-image)
  - GPU support may require additional configuration

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

## Installation

```bash
npm install -g directory-indexer
```

## Usage

### MCP Integration

Configure with Claude Desktop:

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

Start the MCP server:

```bash
directory-indexer serve
```

Your AI assistant (Claude, Cline, Copilot, etc.) can now search your indexed documents semantically. Ask: _"Find API authentication examples"_, _"Show me incidents similar to this"_, or _"Find troubleshooting guides on SQL deadlocks"_.

### CLI Commands

```bash
# Index your directories
# Linux/macOS
directory-indexer index /home/user/projects/api-docs /mnt/work/incident-reports
# Windows
directory-indexer index "C:\work\documentation" "D:\projects\my-app\docs"

# Search semantically
directory-indexer search "database timeout errors"

# Find similar files
# Linux/macOS
directory-indexer similar /mnt/work/incident-reports/redis-outage.md
# Windows
directory-indexer similar "C:\work\incidents\redis-outage.md"

# Get file content
directory-indexer get /home/user/projects/api-docs/auth-guide.md

# Show status
directory-indexer status
```

## Configuration

Directory Indexer uses environment variables for configuration. Set these if your services run on different ports or require API keys:

```bash
# Service endpoints (defaults shown)
export QDRANT_ENDPOINT="http://localhost:6333"
export OLLAMA_ENDPOINT="http://localhost:11434"

# Optional data directory (default: ~/.directory-indexer)
# Linux/macOS
export DIRECTORY_INDEXER_DATA_DIR="/opt/directory-indexer-data"
# Windows
set DIRECTORY_INDEXER_DATA_DIR=D:\data\directory-indexer

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
        "DIRECTORY_INDEXER_DATA_DIR": "/opt/directory-indexer-data"
      }
    }
  }
}
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

## Usage Examples

Once indexed, try these queries with your AI assistant:

**Search by concept:**
- _"Find API authentication examples"_
- _"Show me error handling patterns"_
- _"Find configuration for Redis"_

**Find similar content:**
- _"Show me incidents similar to this outage report"_ *(when you have an incident file open)*
- _"Find documentation like this API guide"_ *(when viewing an API doc)*
- _"What files are similar to my deployment script?"_

**Troubleshoot issues:**
- _"Find troubleshooting guides on SQL deadlocks"_
- _"Show me solutions for timeout errors"_
- _"Find debugging tips for performance issues"_

## License

MIT
