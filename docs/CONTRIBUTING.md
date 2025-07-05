# Contributing to Directory Indexer

## Development Environment Setup

### Prerequisites

- **Node.js**: Version 18+ (latest LTS recommended)
- **npm**: Version 9+ (comes with Node.js)
- **Qdrant**: Local vector database instance
- **Embedding Provider**: Ollama (recommended) or OpenAI API

### Quick Setup

```bash
# 1. Clone and install
git clone https://github.com/peteretelej/directory-indexer.git
cd directory-indexer
npm install

# 2. Start services
docker run -d --name qdrant \
  -p 127.0.0.1:6333:6333 \
  -v qdrant_storage:/qdrant/storage \
  qdrant/qdrant

# 3. Install Ollama and pull model
# Visit https://ollama.ai for installation
ollama pull nomic-embed-text

# 4. Build and test
npm run build
npm test
```

## Project Structure

```
src/
├── cli.ts              # Main CLI entry point
├── config.ts           # Configuration loading
├── storage.ts          # SQLite + Qdrant operations
├── embedding.ts        # Ollama/OpenAI providers
├── indexing.ts         # File scanning and processing
├── search.ts           # Search and similarity
├── mcp.ts              # MCP server implementation
└── utils.ts            # Path/file utilities

tests/
├── integration.test.ts # Main test (covers most paths)
├── unit.test.ts        # Fast unit tests
└── fixtures/           # Test data
```

## Development Workflow

### Building

```bash
# Development build with watch
npm run dev

# Production build
npm run build

# Clean build
npm run clean && npm run build
```

### Testing

```bash
# All tests
npm test

# Unit tests only (fast)
npm run test:unit

# Integration tests (requires services)
npm run test:integration

# Watch mode
npm run test:watch
```

### Code Quality

```bash
# Type checking
npm run typecheck

# Linting
npm run lint

# Auto-fix linting issues
npm run lint -- --fix
```

### Running Commands

```bash
# Index directories
npm run build && node dist/cli.js index ./test_data

# Search content
node dist/cli.js search "database timeout"

# Start MCP server
node dist/cli.js serve

# Show status
node dist/cli.js status
```

## Configuration

### Environment Variables

```bash
# Service endpoints (defaults shown)
export QDRANT_ENDPOINT="http://localhost:6333"
export OLLAMA_ENDPOINT="http://localhost:11434"

# Data directory (default: ~/.directory-indexer)
export DIRECTORY_INDEXER_DATA_DIR="/opt/directory-indexer-dev"

# Collection name (default: directory-indexer)
export DIRECTORY_INDEXER_QDRANT_COLLECTION="my-test-collection"

# Optional API keys
export QDRANT_API_KEY="your-key"
export OLLAMA_API_KEY="your-key"
export OPENAI_API_KEY="your-key"
```

### MCP Development

Test with Claude Desktop configuration:

```json
{
  "mcpServers": {
    "directory-indexer": {
      "command": "node",
      "args": ["/path/to/directory-indexer/dist/cli.js", "serve"],
      "env": {
        "QDRANT_ENDPOINT": "http://localhost:6333",
        "OLLAMA_ENDPOINT": "http://localhost:11434"
      }
    }
  }
}
```

## Testing Strategy

### Integration Tests

The main integration test covers the complete workflow:

1. **Index test fixtures** - Verifies file scanning and embedding
2. **Search functionality** - Tests semantic search
3. **Similar files** - Tests similarity matching
4. **Content retrieval** - Tests file content access
5. **Storage validation** - Verifies SQLite + Qdrant consistency

### Service Dependencies

Tests automatically skip if services are unavailable:

```bash
# Check service health
curl http://localhost:6333/healthz  # Qdrant
curl http://localhost:11434/api/tags  # Ollama
```

### Mock Testing

Unit tests use mock embedding providers for deterministic results without external service dependencies.

## Code Style

### TypeScript Guidelines

- **No classes** - Use functions and plain objects
- **Async/await** - Prefer over Promises/callbacks
- **Explicit types** - Avoid `any`, use strict TypeScript
- **Simple exports** - Named exports, avoid default exports
- **Error handling** - Use proper Error types with context

### Function Style

```typescript
// Good: Simple function with clear types
export async function indexFiles(paths: string[]): Promise<IndexResult> {
  // Implementation
}

// Avoid: Classes with complex inheritance
export class FileIndexer extends BaseIndexer {
  // Complex implementation
}
```

## Debugging

### Enable Debug Logging

```bash
# Set debug level
export DEBUG="directory-indexer:*"
npm run build && node dist/cli.js index ./test_data

# Verbose output
node dist/cli.js --verbose search "query"
```

### Common Issues

```bash
# SQLite permission issues
rm -rf ~/.directory-indexer && mkdir -p ~/.directory-indexer

# Service connection issues
docker restart qdrant
ollama list  # Check if model is available

# Build issues
rm -rf node_modules dist && npm install && npm run build
```

## CI/CD

### Automated Testing

- **Unit tests** - Run on all platforms (Ubuntu, Windows, macOS)
- **Integration tests** - Conditional on main branch or `[integration]` flag
- **Type checking** - Strict TypeScript validation
- **Linting** - ESLint with TypeScript rules

### Release Process

Releases are automated via GitHub Actions:

```bash
# Create release
git tag v1.0.0
git push origin v1.0.0

# Automated actions:
# - Run all tests
# - Build production bundle
# - Publish to npm
# - Create GitHub release
```

## Architecture Notes

### Storage Design

- **SQLite** - Source of truth for file metadata
- **Qdrant** - Vector storage for embeddings
- **Consistency** - SQLite drives Qdrant state

### Embedding Strategy

- **Local-first** - Ollama for development/production
- **Fallback** - OpenAI API for cloud deployments
- **Mock provider** - Deterministic testing

### Error Handling

- **Typed errors** - Custom error classes with context
- **Graceful degradation** - Continue on partial failures
- **User feedback** - Clear error messages with solutions

## Contributing Guidelines

### Pull Requests

1. **Fork and branch** from `main`
2. **Test locally** - Run `npm test` before pushing
3. **Clear commits** - Descriptive commit messages
4. **Update tests** - Add tests for new functionality
5. **Documentation** - Update relevant docs

### Code Review

- **Function-focused** - Prefer small, focused functions
- **Type safety** - Leverage TypeScript fully
- **Error paths** - Handle edge cases gracefully
- **Performance** - Consider async patterns and memory usage

### Community

- **Issues** - Use GitHub Issues for bugs/features
- **Discussions** - Use GitHub Discussions for questions
- **Security** - Report security issues privately

## Troubleshooting

### Build Issues

```bash
# Clear everything and rebuild
rm -rf node_modules dist
npm install
npm run build
```

### Test Failures

```bash
# Ensure services are running
docker ps | grep qdrant
ollama list

# Reset test data
rm -rf ~/.directory-indexer-test
```

### Performance Issues

```bash
# Enable performance monitoring
export NODE_OPTIONS="--enable-source-maps"
npm run build && node --prof dist/cli.js index ./large-directory
```