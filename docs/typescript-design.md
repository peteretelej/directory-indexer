# TypeScript Rewrite Design Document

## Overview

This document outlines the design for rewriting directory-indexer from Rust to TypeScript/Node.js to eliminate binary installation issues on Windows while maintaining full functionality.

**Note**: The original Rust implementation has been moved to `rust-reference/` for reference during migration and will be removed once the TypeScript version is complete and validated.

## Goals

1. **Eliminate binary downloads** - Pure Node.js solution, no platform-specific binaries
2. **Maintain feature parity** - All CLI commands and MCP server functionality
3. **Improve Windows compatibility** - No corporate firewall or antivirus issues
4. **Preserve performance** - Bottleneck is Qdrant/Ollama, not application logic
5. **Simplify deployment** - Standard npm install process

## Architecture Overview

### Core Principles
- **Async-first**: Leverage Node.js event loop for concurrent operations
- **Minimal dependencies**: Keep dependency tree small and secure
- **Cross-platform**: Path handling, file operations work on all platforms
- **Type safety**: Full TypeScript with strict mode
- **Error handling**: Comprehensive error types with proper context

### Project Structure

```
src/
├── cli/                    # CLI command implementations
│   ├── index.ts           # Main CLI entry point
│   ├── commands/          # Command implementations
│   │   ├── index.ts       # Index command
│   │   ├── search.ts      # Search command
│   │   ├── similar.ts     # Similar files command
│   │   ├── get.ts         # Get content command
│   │   ├── serve.ts       # MCP server command
│   │   └── status.ts      # Status command
│   └── args.ts            # CLI argument parsing
├── config/                # Configuration management
│   ├── index.ts           # Configuration loader
│   ├── types.ts           # Configuration types
│   └── defaults.ts        # Default values
├── storage/               # Storage layer
│   ├── sqlite.ts          # SQLite operations
│   ├── qdrant.ts          # Qdrant vector store
│   └── types.ts           # Storage types
├── embedding/             # Embedding providers
│   ├── index.ts           # Provider factory
│   ├── ollama.ts          # Ollama provider
│   ├── openai.ts          # OpenAI provider
│   ├── mock.ts            # Mock provider for testing
│   └── types.ts           # Provider interfaces
├── indexing/              # Indexing engine
│   ├── engine.ts          # Main indexing orchestrator
│   ├── scanner.ts         # File scanning and filtering
│   ├── chunker.ts         # Text chunking logic
│   └── types.ts           # Indexing types
├── search/                # Search engine
│   ├── engine.ts          # Search orchestrator
│   ├── ranker.ts          # Result ranking logic
│   └── types.ts           # Search types
├── mcp/                   # MCP server
│   ├── server.ts          # MCP server implementation
│   ├── tools.ts           # MCP tool definitions
│   └── types.ts           # MCP types
├── utils/                 # Utility functions
│   ├── paths.ts           # Path normalization
│   ├── files.ts           # File operations
│   ├── text.ts            # Text processing
│   └── validation.ts      # Input validation
├── errors/                # Error handling
│   ├── index.ts           # Error types and utilities
│   └── types.ts           # Error type definitions
└── types/                 # Global types
    ├── index.ts           # Main type exports
    └── common.ts          # Common type definitions
```

## Key Components

### 1. CLI Layer (`src/cli/`)

**Main Entry Point** (`cli/index.ts`):
- Uses `commander.js` for argument parsing
- Global options: `--verbose`, `--config`
- Command delegation to specific handlers

**Command Structure**:
```typescript
interface CLICommand {
  name: string;
  description: string;
  options: CommandOption[];
  handler: (args: any, config: Config) => Promise<void>;
}
```

### 2. Configuration Management (`src/config/`)

**Configuration Hierarchy**:
1. Default values
2. Environment variables
3. Config file (JSON/YAML)
4. Command-line arguments

**Configuration Interface**:
```typescript
interface Config {
  storage: {
    sqlitePath: string;
    qdrantEndpoint: string;
    qdrantCollection: string;
    qdrantApiKey?: string;
  };
  embedding: {
    provider: 'ollama' | 'openai' | 'mock';
    model: string;
    endpoint: string;
    apiKey?: string;
    dimensions: number;
  };
  indexing: {
    chunkSize: number;
    chunkOverlap: number;
    maxFileSize: number;
    ignorePatterns: string[];
    batchSize: number;
  };
  dataDir: string;
  verbose: boolean;
}
```

### 3. Storage Layer (`src/storage/`)

**SQLite Store** (`storage/sqlite.ts`):
- Uses `better-sqlite3` for synchronous operations
- Database schema matches Rust implementation
- Prepared statements for performance
- Transaction support for batch operations

**Qdrant Store** (`storage/qdrant.ts`):
- HTTP client using `node-fetch` or built-in `fetch`
- REST API interactions, no SDK dependency
- Point management and collection operations
- Error handling for network failures

### 4. Embedding Providers (`src/embedding/`)

**Provider Interface**:
```typescript
interface EmbeddingProvider {
  name: string;
  dimensions: number;
  generateEmbedding(text: string): Promise<number[]>;
  generateEmbeddings(texts: string[]): Promise<number[][]>;
  healthCheck(): Promise<boolean>;
}
```

**Ollama Provider**:
- HTTP requests to local Ollama server
- Model management and health checks
- Batch processing support

**OpenAI Provider**:
- OpenAI API integration
- Rate limiting and error handling
- Cost optimization for batch requests

### 5. Indexing Engine (`src/indexing/`)

**Main Engine** (`indexing/engine.ts`):
- Orchestrates file scanning, chunking, embedding, storage
- Concurrent processing with configurable limits
- Progress tracking and statistics
- Error recovery and retry logic

**File Scanner** (`indexing/scanner.ts`):
- Recursive directory traversal
- File filtering by patterns and size
- Content extraction with encoding detection
- Path normalization for cross-platform support

**Text Chunker** (`indexing/chunker.ts`):
- Sliding window chunking with overlap
- Preserves word boundaries
- Handles multiple file formats
- Metadata preservation

### 6. Search Engine (`src/search/`)

**Search Engine** (`search/engine.ts`):
- Query embedding generation
- Vector similarity search
- Result filtering and ranking
- Metadata enrichment

**Result Ranking** (`search/ranker.ts`):
- Score-based ranking algorithms
- Directory filtering
- Similarity thresholds
- Result deduplication

### 7. MCP Server (`src/mcp/`)

**Server Implementation** (`mcp/server.ts`):
- JSON-RPC 2.0 over stdio
- Tool registration and execution
- Error handling and validation
- Protocol compliance

**Tool Definitions**:
```typescript
interface MCPTool {
  name: string;
  description: string;
  inputSchema: JSONSchema;
  handler: (args: any) => Promise<any>;
}
```

## Dependencies

### Production Dependencies

**Core Runtime**:
- `commander` - CLI argument parsing
- `better-sqlite3` - SQLite database
- `node-fetch` or native `fetch` - HTTP requests
- `zod` - Runtime type validation

**File Operations**:
- `glob` - File pattern matching
- `mime-types` - File type detection
- `fast-glob` - Fast file system traversal

**Utilities**:
- `path` (built-in) - Path manipulation
- `fs/promises` (built-in) - File system operations
- `crypto` (built-in) - Hashing

**MCP**:
- `@modelcontextprotocol/sdk` - MCP protocol implementation

### Development Dependencies

**TypeScript**:
- `typescript` - TypeScript compiler
- `@types/node` - Node.js type definitions
- `@types/better-sqlite3` - SQLite type definitions

**Testing**:
- `vitest` - Fast test runner
- `@types/jest` - Type definitions for testing

**Build Tools**:
- `tsup` - TypeScript bundler
- `rimraf` - Cross-platform rm -rf

## Migration Strategy

### Phase 1: Core Infrastructure
1. Set up TypeScript project structure
2. Implement configuration management
3. Create storage layer (SQLite + Qdrant)
4. Add embedding providers

### Phase 2: Indexing Engine
1. Implement file scanner
2. Add text chunker
3. Create indexing engine
4. Add batch processing

### Phase 3: Search Engine
1. Implement search functionality
2. Add result ranking
3. Create similar files feature
4. Add content retrieval

### Phase 4: CLI Interface
1. Implement CLI commands
2. Add argument parsing
3. Error handling and validation
4. Progress reporting

### Phase 5: MCP Server
1. Implement MCP protocol
2. Add tool definitions
3. Server lifecycle management
4. Integration testing

### Phase 6: Testing & Polish
1. Comprehensive test suite
2. Performance optimization
3. Documentation updates
4. Binary compatibility testing

## Performance Considerations

### Bottleneck Analysis
- **Embedding generation**: 90% of execution time (external service)
- **Vector search**: 5% of execution time (Qdrant)
- **File I/O**: 4% of execution time
- **Application logic**: 1% of execution time

### Optimization Strategies
1. **Concurrent processing**: Parallel file scanning and embedding generation
2. **Batch operations**: Group database operations and API calls
3. **Memory management**: Stream large files, avoid loading everything into memory
4. **Caching**: Cache embeddings and frequently accessed data
5. **Connection pooling**: Reuse HTTP connections for external services

## Error Handling

### Error Types
```typescript
abstract class AppError extends Error {
  abstract code: string;
  abstract statusCode: number;
}

class ConfigError extends AppError { /* ... */ }
class StorageError extends AppError { /* ... */ }
class EmbeddingError extends AppError { /* ... */ }
class FileSystemError extends AppError { /* ... */ }
class NetworkError extends AppError { /* ... */ }
class ValidationError extends AppError { /* ... */ }
```

### Error Recovery
- Retry logic for transient failures
- Graceful degradation for non-critical errors
- User-friendly error messages
- Detailed logging for debugging

## Testing Strategy

### Test Structure
```
tests/
├── unit/                  # Fast tests, no external dependencies
│   ├── config.test.ts    # Configuration loading
│   ├── chunker.test.ts   # Text chunking logic
│   ├── paths.test.ts     # Path utilities
│   └── embedding/        # Mock embedding providers
├── integration/          # Real services required
│   ├── indexing.test.ts  # Full indexing workflow
│   ├── search.test.ts    # Search functionality
│   └── cli.test.ts       # CLI commands
├── mcp/                  # MCP protocol tests
│   └── server.test.ts    # JSON-RPC server
├── fixtures/             # Test data
│   ├── docs/            # Markdown files
│   ├── code/            # Source code files
│   └── configs/         # Configuration files
└── helpers/              # Test utilities
    ├── services.ts       # Service health checks
    ├── cleanup.ts        # Test cleanup
    └── mocks.ts          # Mock implementations
```

### Testing Framework
- **Vitest** - Fast test runner with TypeScript support
- **Supertest** - HTTP testing (if needed)
- **tmp** - Temporary directories
- **Mock providers** - Deterministic fake embeddings

### Test Data Management
- **Persistent test collection** - Shared across integration tests
- **Lazy initialization** - Check if data exists before indexing
- **Real file corpus** - Consistent test files matching production use

### Service Mocking
- **Mock embedding provider** - Deterministic outputs, configurable dimensions
- **Health checks** - Skip tests if services unavailable
- **Timeout handling** - Graduated timeouts (30s unit, 300s integration)

## Security Considerations

### Input Validation
- Path traversal prevention
- File size limits
- Content type validation
- SQL injection prevention

### External Services
- API key management
- TLS/SSL verification
- Rate limiting
- Timeout handling

### File System Access
- Sandboxed file operations
- Permission checks
- Symlink handling
- Access logging

## Deployment and Distribution

### npm Package
- Pure TypeScript/JavaScript
- No binary dependencies
- Cross-platform compatibility
- Minimal installation footprint

### Build Process
- TypeScript compilation
- Bundle optimization
- Tree shaking
- Source maps for debugging

### Publishing
- Automated CI/CD pipeline
- Version management
- Release notes
- Backwards compatibility

## Future Enhancements

### Potential Improvements
1. **Plugin system**: Custom embedding providers
2. **Web UI**: Browser-based interface
3. **Real-time indexing**: File system watching
4. **Distributed indexing**: Multi-node support
5. **Advanced search**: Filters, facets, boolean queries

### Monitoring and Observability
- Metrics collection
- Health checks
- Performance monitoring
- Error tracking

## Conclusion

The TypeScript rewrite will eliminate Windows installation issues while maintaining full functionality. The modular architecture allows for incremental migration and future enhancements. Performance will be adequate since the bottleneck is external services, not application logic.

The pure Node.js approach aligns with the success of uvx-based Python tools and will provide a better user experience on Windows platforms.