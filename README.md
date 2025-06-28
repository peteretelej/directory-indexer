# Directory Indexer

**Turn your directories into an AI-powered knowledge base.**

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

## Requirements

- **Qdrant**: Vector database for semantic search

```bash
docker run -d --name qdrant \
    -p 127.0.0.1:6333:6333 \
    -v qdrant_storage:/qdrant/storage \
    qdrant/qdrant
```

**Embedding Provider** (choose one):

- **Ollama** (recommended): Self-hosted embeddings

```bash
# Install Ollama natively for GPU support (recommended)
curl -fsSL https://ollama.ai/install.sh | sh
# Visit https://ollama.ai for Windows installer
ollama pull nomic-embed-text

# OR use Docker (CPU-only, slower)
docker run -d --name ollama \
    -p 127.0.0.1:11434:11434 \
    -v ollama_data:/root/.ollama \
    ollama/ollama

docker exec ollama ollama pull nomic-embed-text
```

- **OpenAI**: Requires API key

## Configuration

Directory Indexer uses environment variables for configuration. Set these if your services run on different ports or require API keys:

```bash
# Service endpoints (defaults shown)
export QDRANT_ENDPOINT="http://localhost:6333"
export OLLAMA_ENDPOINT="http://localhost:11434"

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
        "OLLAMA_ENDPOINT": "http://localhost:11434"
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
