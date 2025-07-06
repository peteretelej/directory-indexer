# Directory Indexer

**Turn your directories into an AI-powered knowledge base.**

[![npm](https://img.shields.io/npm/v/directory-indexer)](https://npmjs.com/package/directory-indexer)
[![codecov](https://codecov.io/gh/peteretelej/directory-indexer/graph/badge.svg?token=j6aBBpfqkN)](https://codecov.io/gh/peteretelej/directory-indexer)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/peteretelej/directory-indexer/workflows/CI/badge.svg)](https://github.com/peteretelej/directory-indexer/actions)

Self-hosted semantic search for local files. Enable AI assistants to search your documents using vector embeddings and MCP integration.

## Quick Start

**Prerequisites:**

- **[Docker](https://docs.docker.com/get-docker/)** - For running Qdrant and Ollama _(skip if you already have them running natively)_
- **[Node.js 18+](https://nodejs.org/en/download/)** - Required for running directory-indexer

_Note: For native Qdrant and Ollama installation without Docker, see [Setup section](#setup)._

**1. Start Qdrant vector database** _(skip if already running)_

```bash
docker run -d --name qdrant -p 127.0.0.1:6333:6333 -v qdrant_storage:/qdrant/storage qdrant/qdrant
```

**2. Start Ollama embedding service** _(skip if already running)_

_Note: Docker Ollama won't use GPU acceleration. For better performance, consider [native installation](#2-embedding-provider)._

```bash
docker run -d --name ollama -p 127.0.0.1:11434:11434 -v ollama:/root/.ollama ollama/ollama

# Pull the embedding model
docker exec ollama ollama pull nomic-embed-text
```

**3. Index your directories**

```bash
npx directory-indexer index ~/Documents ~/Projects
```

**4. Configure AI assistant** _(Claude Desktop, Cursor, Cline, Roo Code, Zed etc.)_

Add to your MCP configuration:

```json
{
  "mcpServers": {
    "directory-indexer": {
      "command": "npx",
      "args": ["directory-indexer", "serve"]
    }
  }
}
```

Your AI assistant will automatically start the MCP server and can now search your indexed files.

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
# Check Qdrant Health
curl http://localhost:6333/healthz

# View collections
curl http://localhost:6333/collections

# Check Ollama
curl http://localhost:11434/api/tags
```

If either fails, directory-indexer will show a helpful error with setup guidance.

## Usage

### MCP Integration

Configure with AI assistants (Claude Desktop, Cline, etc.) using npx:

```json
{
  "mcpServers": {
    "directory-indexer": {
      "command": "npx",
      "args": ["directory-indexer", "serve"]
    }
  }
}
```

Index your directories:

```bash
# Index your directories first
npx directory-indexer index /home/user/projects/docs /home/user/work/reports
```

**How it works:**

1. **MCP server starts automatically** - When your AI assistant connects, it launches the MCP server in the background
2. **Indexing runs independently** - You can index files before, during, or after MCP setup
3. **Search immediately available** - Your AI assistant can search files as soon as they're indexed

**Key point:** You don't need to wait for indexing to complete before using the MCP server. Index files as needed, and your AI assistant will immediately have access to search them.

### Using with AI Assistant

Once configured, your AI assistant can search your indexed documents semantically:

**Search by concept:**

- _"Find API authentication examples"_
- _"Show me error handling patterns"_
- _"Find configuration for Redis"_

**Find similar content:**

- _"Show me incidents similar to this outage report"_ _(when you have an incident file open)_
- _"Find documentation like this API guide"_ _(when viewing an API doc)_
- _"What files are similar to my deployment script?"_

**Troubleshoot issues:**

- _"Find troubleshooting guides on SQL deadlocks"_
- _"Show me solutions for timeout errors"_
- _"Find debugging tips for performance issues"_

### Custom Configuration

Configure with custom endpoints and data directory:

```json
{
  "mcpServers": {
    "directory-indexer": {
      "command": "npx",
      "args": ["directory-indexer", "serve"],
      "env": {
        "DIRECTORY_INDEXER_DATA_DIR": "/opt/ai-knowledge-base",
        "QDRANT_ENDPOINT": "http://localhost:6333",
        "OLLAMA_ENDPOINT": "http://localhost:11434"
      }
    }
  }
}
```

### CLI Usage

For advanced users who prefer command-line usage, see [CLI Documentation](./docs/design.md#cli-usage).

## Configuration

Environment variables (all optional):

```bash
# Data directory (default: ~/.directory-indexer)
export DIRECTORY_INDEXER_DATA_DIR="/opt/ai-knowledge-base"

# Service endpoints (defaults shown)
export QDRANT_ENDPOINT="http://localhost:6333"
export OLLAMA_ENDPOINT="http://localhost:11434"

# Optional API keys
export OPENAI_API_KEY="your-key-here"
export QDRANT_API_KEY="your-key-here"
```

For all configuration options, see [Environment Variables](./docs/design.md#environment-variables).

## Supported Files

- **Text**: `.md`, `.txt`
- **Code**: `.rs`, `.py`, `.js`, `.ts`, `.go`, `.java`, etc.
- **Data**: `.json`, `.yaml`, `.csv`, `.toml`
- **Config**: `.env`, `.conf`, `.ini`

## Documentation

- **[API Reference](docs/API.md)**: CLI commands and MCP tools
- **[Flow Diagrams](docs/flows.md)**: System architecture and process flows
- **[Contributing](docs/CONTRIBUTING.md)**: Development setup and guidelines
- **[Design](docs/design.md)**: Architecture and technical decisions

## License

MIT
