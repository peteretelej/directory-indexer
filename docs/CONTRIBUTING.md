# Contributing to Directory Indexer

## Quick Start

```bash
# 1. Clone and setup
git clone https://github.com/peteretelej/directory-indexer.git
cd directory-indexer && npm install

# 2. Build and test
npm run build && npm test

# 3. Try local CLI
npm run cli status
npm run cli index ./tests/test_data
npm run cli search "database"
```

## Development Scenarios

### Scenario 1: Quick Code Changes

For small changes and testing:

```bash
# Make your changes, then:
npm run build
npm run cli <command>
```

### Scenario 2: Live Development

For active development with automatic rebuilds:

```bash
# Terminal 1: Watch mode (rebuilds on file changes)
npm run dev

# Terminal 2: Test commands (rerun after changes)
npm run cli status
npm run cli index ./tests/test_data
```

### Scenario 3: Integration Testing

For testing with real services (Qdrant + Ollama):

```bash
# Start services
./scripts/start-dev-services.sh

# Run integration tests
npm test

# Run specific commands with services
export QDRANT_ENDPOINT=http://localhost:6333
export OLLAMA_ENDPOINT=http://localhost:11434
npm run cli index ./tests/test_data
npm run cli search "authentication"

# Stop services when done
./scripts/stop-dev-services.sh
```

### Scenario 4: Clean Environment Testing

For testing CLI as end-users would experience it:

```bash
# Setup clean Docker environment
./scripts/docker-debug/setup-debug-container.sh

# Test commands in container
docker exec test-directory-indexer directory-indexer status
docker exec test-directory-indexer directory-indexer index /test-data

# Cleanup
docker stop test-directory-indexer && docker rm test-directory-indexer
```

## Project Structure

```
src/                    # TypeScript source code
├── cli.ts             # Main CLI entry point
├── config.ts          # Configuration management
├── storage.ts         # SQLite + Qdrant operations
├── embedding.ts       # Ollama/OpenAI providers
├── indexing.ts        # File processing
├── search.ts          # Search functionality
└── mcp.ts             # MCP server

bin/
└── directory-indexer.js # CLI wrapper (calls dist/cli.js)

dist/                  # Built JavaScript (npm run build)
tests/                 # Test files
scripts/               # Development helper scripts
```

## Testing

### Unit Tests (No Services Required)

```bash
npm run test:unit        # Fast tests, no external dependencies
npm run test:unit -- --watch  # Watch mode
```

### Integration Tests (Requires Services)

```bash
# Start services first
./scripts/start-dev-services.sh

# Run integration tests
npm run test:integration

# Or run all tests with coverage
npm test
```

### Test Structure

- **Unit tests**: Mock embedding provider, no external services
- **Integration tests**: Real Qdrant + Ollama services
- **Test data**: `tests/test_data/` contains structured files for testing

## Code Quality

```bash
npm run typecheck       # TypeScript validation
npm run lint            # ESLint checks
npm run lint -- --fix  # Auto-fix issues
```

## Development Commands

All CLI commands require building first:

```bash
npm run build           # Required before CLI usage
npm run cli status      # Show indexing status
npm run cli index <dir> # Index directory
npm run cli search <query> # Search content
npm run cli similar <file> # Find similar files
npm run cli get <file>  # Get file content
npm run cli serve       # Start MCP server
```

Direct node execution:
```bash
node bin/directory-indexer.js <command>
```

## Environment Setup

### Required Services

- **Qdrant**: Vector database on `localhost:6333`
- **Ollama**: Embedding provider on `localhost:11434` with `nomic-embed-text` model

### Using Development Scripts

```bash
./scripts/start-dev-services.sh  # Start Qdrant + Ollama via Docker
./scripts/stop-dev-services.sh   # Stop services
```

### Manual Service Setup

**Qdrant:**
```bash
docker run -d --name qdrant -p 127.0.0.1:6333:6333 -v qdrant_storage:/qdrant/storage qdrant/qdrant
```

**Ollama:**
```bash
# Install from https://ollama.ai or use Docker:
docker run -d --name ollama -p 127.0.0.1:11434:11434 -v ollama:/root/.ollama ollama/ollama
docker exec ollama ollama pull nomic-embed-text
```

### Environment Variables

```bash
export QDRANT_ENDPOINT="http://localhost:6333"
export OLLAMA_ENDPOINT="http://localhost:11434"
export DIRECTORY_INDEXER_DATA_DIR="/tmp/directory-indexer-dev"
```

## CI/CD

### Automated Testing

- **Unit tests**: Run on Ubuntu, Windows, macOS with Node 18+20
- **Integration tests**: Run conditionally with real services
- **Code quality**: TypeScript validation + ESLint

### Triggering Integration Tests

Integration tests run automatically on `main` branch, or manually with:
- Commit message containing `[integration]`
- PR title containing `[integration]`

### Release Process

```bash
git tag v1.0.0 && git push origin v1.0.0
# Automatically: builds, tests, publishes to npm
```

## Debugging

### Common Issues

```bash
# Build not found
npm run build

# Service connection errors
./scripts/start-dev-services.sh
curl http://localhost:6333/healthz
curl http://localhost:11434/api/tags

# Clean rebuild
rm -rf node_modules dist && npm install && npm run build
```

### Debug Logging

```bash
npm run cli -- status --verbose
npm run cli -- search "query" --verbose
```

## Code Style

- **TypeScript**: Strict mode, explicit types
- **Functions**: Prefer functions over classes
- **Async**: Use async/await over promises
- **Exports**: Named exports, avoid defaults
- **Errors**: Typed errors with context

## Pull Request Process

1. **Fork and branch** from `main`
2. **Make changes** and ensure `npm run build` works
3. **Add tests** for new functionality
4. **Run quality checks**: `npm run typecheck && npm run lint`
5. **Test locally**: `npm test` (with services if needed)
6. **Clear commit messages** describing changes
7. **Submit PR** with description of changes

## Getting Help

- **Issues**: Report bugs and feature requests
- **Discussions**: Ask questions about development
- **Documentation**: Check `docs/` folder for detailed guides