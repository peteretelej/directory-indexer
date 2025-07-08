# Directory Indexer - Node.js Design

## Overview

Pure Node.js/TypeScript implementation for AI-powered semantic search of local files. Eliminates binary installation issues while maintaining full feature parity with the Rust version.

Design documents:

- [Design](design.md) - Design overview and architecture
- [API](API.md) - CLI commands and MCP tools
- [Design Flows](flows.md) - Flow diagrams

Design documents should be kept up-to-date with implementation changes.

## Architecture

**Core Services**

- SQLite (metadata) + Qdrant (vectors) + Ollama/OpenAI (embeddings)
- CLI commands + MCP server
- Simple functions, no classes

**Coding Rules**

- No comments (self-explanatory code)
- Prefer strict types, no any
- No duplication (eliminate redundant functions)
- Simple logic (straightforward over complex)
- Clear naming (functions explain purpose)
- Minimal abstraction (only when needed)
- No mocks (integration tests with real services)
- Real testing (use `./scripts/start-dev-services.sh`)

**Key Benefits**

- No binary downloads (pure npm package)
- Cross-platform compatibility
- Standard npm install process

## Project Structure

```
src/
├── cli.ts              # Main CLI entry point with commander.js
├── config.ts           # Configuration loading and validation
├── storage.ts          # SQLite + Qdrant operations
├── embedding.ts        # Ollama/OpenAI providers
├── indexing.ts         # File scanning and processing
├── search.ts           # Search and similarity
├── mcp.ts              # MCP server implementation
└── utils.ts            # Path/file utilities

tests/
├── integration.test.ts # Main test (covers most paths)
├── unit.test.ts        # Fast unit tests
├── fixtures/           # Test markdown/code files
└── test_data/          # Real file corpus for testing
```

## Implementation Phases

### Phase 1: Core Foundation

**Files**: `config.ts`, `utils.ts`, `storage.ts`

- **Configuration System** - Environment variables with defaults
- **Path Utilities** - Cross-platform path handling
- **SQLite Setup** - Database schema and basic operations
- **Qdrant Client** - HTTP REST API calls

**Data Types**:

```typescript
type Config = {
  storage: {
    sqlitePath: string;
    qdrantEndpoint: string;
    qdrantCollection: string;
  };
  embedding: {
    provider: "ollama" | "openai" | "mock";
    model: string;
    endpoint: string;
  };
  indexing: {
    chunkSize: number;
    maxFileSize: number;
    ignorePatterns: string[];
  };
  dataDir: string;
  verbose: boolean;
};

type FileRecord = {
  path: string;
  size: number;
  modifiedTime: Date;
  hash: string;
  parentDirs: string[];
  chunks: ChunkInfo[];
  errors?: string[];
};
```

### Phase 2: Embedding & Indexing

**Files**: `embedding.ts`, `indexing.ts`

- **Embedding Providers** - Ollama, OpenAI, Mock implementations
- **File Scanner** - Directory traversal with ignore patterns
- **Text Chunker** - Sliding window with overlap
- **Indexing Engine** - Orchestrate scan → chunk → embed → store

**Core Functions**:

```typescript
export async function generateEmbedding(
  text: string,
  config: Config
): Promise<number[]>;
export async function scanDirectory(
  path: string,
  options: ScanOptions
): Promise<FileInfo[]>;
export async function chunkText(
  content: string,
  chunkSize: number,
  overlap: number
): Promise<Chunk[]>;
export async function indexDirectories(
  paths: string[],
  config: Config
): Promise<IndexResult>;
```

### Phase 3: Search & Retrieval

**Files**: `search.ts`

- **Semantic Search** - Query embedding + vector search + metadata enrichment
- **Similar Files** - File-to-file similarity matching
- **Content Retrieval** - Get file content with chunk selection
- **Result Ranking** - Score-based sorting and filtering

**Core Functions**:

```typescript
export async function searchContent(
  query: string,
  options: SearchOptions
): Promise<SearchResult[]>;
export async function findSimilarFiles(
  filePath: string,
  limit: number
): Promise<SimilarFile[]>;
export async function getFileContent(
  filePath: string,
  chunks?: string
): Promise<string>;
export async function getChunkContent(
  filePath: string,
  chunkId: string
): Promise<string>;
```

### Phase 4: CLI Interface

**Files**: `cli.ts`

- **Command Parsing** - Commander.js setup with subcommands
- **CLI Commands** - index, search, similar, get, status, serve
- **Progress Display** - User feedback and error handling
- **Cross-platform Support** - Windows/Unix path handling

**Command Structure**:

```bash
directory-indexer index <paths...>
directory-indexer search <query> [--limit 10]
directory-indexer similar <file> [--limit 10]
directory-indexer get <file> [--chunks 2-5]
directory-indexer serve
directory-indexer status
```

### Phase 5: MCP Server

**Files**: `mcp.ts`

- **MCP Protocol** - JSON-RPC 2.0 over stdio
- **Tool Definitions** - index, search, similar_files, get_content, get_chunk, server_info
- **Error Handling** - Proper MCP error responses
- **Integration** - Reuse CLI logic for tool implementations

**MCP Tools**:

```typescript
const tools = {
  index: (args: { directory_path: string }) =>
    indexDirectories(args.directory_path.split(",")),
  search: (args: { query: string; limit?: number }) =>
    searchContent(args.query, { limit }),
  similar_files: (args: { file_path: string; limit?: number }) =>
    findSimilarFiles(args.file_path, args.limit),
  get_content: (args: { file_path: string; chunks?: string }) =>
    getFileContent(args.file_path, args.chunks),
  get_chunk: (args: { file_path: string; chunk_id: string }) =>
    getChunkContent(args.file_path, args.chunk_id),
  server_info: () => getServerInfo(),
};
```

## Data Flow

**Indexing**: `scanDirectory()` → `chunkText()` → `generateEmbedding()` → `storeInSQLite()` + `storeInQdrant()`

**Search**: `generateEmbedding(query)` → `vectorSearch()` → `groupByFile()` → `calculateAverageScores()` → `rankResults()`

**Storage**: SQLite as source of truth, Qdrant synced for vectors

## Storage Schema

**SQLite Tables** (matches Rust implementation):

```sql
CREATE TABLE directories (id INTEGER PRIMARY KEY, path TEXT UNIQUE, status TEXT, indexed_at INTEGER);
CREATE TABLE files (id INTEGER PRIMARY KEY, path TEXT UNIQUE, size INTEGER, modified_time INTEGER,
                   hash TEXT, parent_dirs TEXT, chunks_json TEXT, errors_json TEXT);
```

**Qdrant Points**:

```typescript
{ id: uuid, vector: number[], payload: { filePath: string, chunkId: string, parentDirectories: string[] } }
```

## Configuration

Environment variables with sensible defaults:

```bash
QDRANT_ENDPOINT=http://localhost:6333
OLLAMA_ENDPOINT=http://localhost:11434
DIRECTORY_INDEXER_DATA_DIR=~/.directory-indexer
DIRECTORY_INDEXER_QDRANT_COLLECTION=directory-indexer
```

**Hierarchy**: Defaults → Environment → CLI args

## Environment Variables

### Core Configuration

- `DIRECTORY_INDEXER_DATA_DIR` - Data directory (default: `~/.directory-indexer`)
- `QDRANT_ENDPOINT` - Qdrant vector database endpoint (default: `http://localhost:6333`)
- `OLLAMA_ENDPOINT` - Ollama embedding provider endpoint (default: `http://localhost:11434`)
- `DIRECTORY_INDEXER_QDRANT_COLLECTION` - Qdrant collection name (default: `directory-indexer`)

### Embedding Provider

- `EMBEDDING_PROVIDER` - Provider type: `ollama`, `openai`, `mock` (default: `ollama`)
- `EMBEDDING_MODEL` - Model name (default: `nomic-embed-text`)
- `OPENAI_API_KEY` - OpenAI API key (required for OpenAI provider)

### Processing Options

- `CHUNK_SIZE` - Text chunk size in characters (default: `512`)
- `CHUNK_OVERLAP` - Chunk overlap in characters (default: `50`)
- `MAX_FILE_SIZE` - Maximum file size in bytes (default: `10485760` = 10MB)
- `VERBOSE` - Enable verbose logging: `true`/`false` (default: `false`)

### API Keys (Optional)

- `QDRANT_API_KEY` - Qdrant API key for authenticated instances
- `OLLAMA_API_KEY` - Ollama API key for hosted instances

## CLI Usage

For users who prefer command-line interface over MCP integration:

### Installation

```bash
npm install -g directory-indexer
```

### Environment Setup

```bash
# Optional: Custom data directory
export DIRECTORY_INDEXER_DATA_DIR="/opt/ai-knowledge-base"

# Optional: Custom service endpoints
export QDRANT_ENDPOINT="http://localhost:6333"
export OLLAMA_ENDPOINT="http://localhost:11434"

# Optional: OpenAI provider
export EMBEDDING_PROVIDER="openai"
export OPENAI_API_KEY="your-api-key"
```

### Basic Usage

```bash
# Index directories
directory-indexer index /home/user/projects/docs /home/user/work/reports

# Search semantically
directory-indexer search "database timeout errors" --limit 10

# Find similar files
directory-indexer similar /path/to/file.md --limit 5

# Get file content
directory-indexer get /path/to/file.md

# Show indexing status
directory-indexer status

# Start MCP server
directory-indexer serve
```

### Advanced Usage

```bash
# Verbose output
directory-indexer --verbose search "authentication examples"

# Multiple directories
directory-indexer index "/home/user/docs" "/opt/projects" "/var/log/app"

# Custom limits
directory-indexer search "error handling" --limit 20
directory-indexer similar file.py --limit 10
```

## Error Handling

**Simple Error Types**:

```typescript
export class AppError extends Error {
  constructor(message: string, public code: string, public cause?: Error) {
    super(message);
  }
}
export class ConfigError extends AppError {
  constructor(msg: string, cause?: Error) {
    super(msg, "CONFIG_ERROR", cause);
  }
}
export class StorageError extends AppError {
  constructor(msg: string, cause?: Error) {
    super(msg, "STORAGE_ERROR", cause);
  }
}
export class EmbeddingError extends AppError {
  constructor(msg: string, cause?: Error) {
    super(msg, "EMBEDDING_ERROR", cause);
  }
}
```

**Recovery Strategy**: Continue on partial failures, log errors in SQLite

## Testing Strategy

**Integration-First Testing**:

1. **Real Integration Tests** - Use actual Qdrant + Ollama via `./scripts/start-dev-services.sh`
   - Index `tests/test_data/` with real embeddings
   - Search with actual vector similarity
   - Verify SQLite + Qdrant consistency
   - Test all embedding providers (mock, ollama)
2. **Pure Function Tests** - Only for algorithmic functions (text chunking, path utils)

**Development Services**: `./scripts/start-dev-services.sh` provides Docker containers
**Auto-Skip**: Tests skip gracefully if services unavailable (CI compatibility)

## Dependencies

**Production** (minimal):

- `commander` - CLI parsing
- `better-sqlite3` - SQLite
- `@modelcontextprotocol/sdk` - MCP protocol
- `glob` - File patterns
- `zod` - Type validation

**Development**:

- `vitest` - Testing
- `tsup` - Building
- `eslint` + `typescript` - Code quality

## Migration from Rust

**Compatibility**:

- Same MCP tools and functionality
- Same SQLite schema and Qdrant collection format
- Same configuration environment variables
- Same file type support and chunking strategy

**Key Differences**:

- Pure JavaScript (no binaries)
- Function-based (no classes)
- Single integration test (vs multiple test files)
- Simplified project structure (8 files vs 30+ modules)
- npx installation (no global install required)
