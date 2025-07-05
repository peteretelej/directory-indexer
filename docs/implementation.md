# Directory Indexer - Node.js Implementation

## Overview

Implementation tracking for the Node.js/TypeScript port of the Directory Indexer. Focus on simple, clear, and concise code following modern TypeScript best practices.

## Implementation Rules

- **TypeScript**: Strict type checking, minimal `any` types
- **Simplicity**: Functions over classes, clear over clever, avoid complexity
- **Conciseness**: Minimal dependencies, focused functionality, no code duplication
- **Readability**: Self-explanatory code, conventional patterns, no comments
- **Best Practices**: Modern TypeScript patterns, proper error handling
- **Testing**: Unit tests with mocks, integration tests with real services

### Code Guidelines

- **No Comments**: Code should be self-explanatory through clear naming and structure
- **No Duplication**: Eliminate redundant functions and repeated code patterns
- **Simple Logic**: Prefer straightforward implementations over complex optimizations
- **Clear Naming**: Function and variable names should explain their purpose
- **Minimal Abstraction**: Only abstract when there's clear repeated patterns

## Phase 1: Core Foundation

**Status**: ✅ Complete

### Files Implemented

- [x] `docs/implementation.md` - This tracking document
- [x] `src/config.ts` - Configuration system with environment variables and validation
- [x] `src/utils.ts` - Cross-platform path utilities and file operations
- [x] `src/storage.ts` - SQLite setup and Qdrant HTTP client with proper error handling
- [x] `src/embedding.ts` - Stub file for Phase 2 (embedding providers)
- [x] `src/indexing.ts` - Stub file for Phase 2 (file scanning and processing)
- [x] `src/search.ts` - Stub file for Phase 3 (search functionality)

### Phase 1 Test Results

✅ Configuration tests passing - Environment variables loaded correctly
✅ Path utilities tests passing - Cross-platform path handling working  
✅ Storage architecture complete - SQLite and Qdrant client classes implemented
✅ Linting passes - Only expected warnings for Phase 2+ stubs
⚠️ Future phase tests failing as expected (not yet implemented)

### Code Simplifications Applied

- **Removed comments**: Inline explanations replaced with clear code
- **Eliminated duplicates**: Combined `toAbsolutePath` → `normalizePath`, `calculateFileHash` → `calculateHash`
- **Simplified logic**: Streamlined ignore pattern matching and error handling
- **Cleaner interfaces**: Removed unnecessary stub function comments

### Key Data Types

```typescript
type Config = {
  storage: { 
    sqlitePath: string; 
    qdrantEndpoint: string; 
    qdrantCollection: string; 
  };
  embedding: { 
    provider: 'ollama' | 'openai' | 'mock'; 
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

### Configuration Requirements

Environment variables with sensible defaults:
- `QDRANT_ENDPOINT=http://localhost:6333`
- `OLLAMA_ENDPOINT=http://localhost:11434`
- `DIRECTORY_INDEXER_DATA_DIR=~/.directory-indexer`
- `DIRECTORY_INDEXER_QDRANT_COLLECTION=directory-indexer`

### SQLite Schema

```sql
CREATE TABLE directories (
  id INTEGER PRIMARY KEY,
  path TEXT UNIQUE NOT NULL,
  status TEXT DEFAULT 'pending',
  indexed_at INTEGER DEFAULT 0
);

CREATE TABLE files (
  id INTEGER PRIMARY KEY,
  path TEXT UNIQUE NOT NULL,
  size INTEGER NOT NULL,
  modified_time INTEGER NOT NULL,
  hash TEXT NOT NULL,
  parent_dirs TEXT NOT NULL,
  chunks_json TEXT,
  errors_json TEXT
);
```

### Qdrant Integration

- HTTP REST API calls (no SDK dependency)
- Collection: `directory-indexer`
- Point format: `{ id: uuid, vector: number[], payload: { filePath, chunkId, parentDirectories } }`

## Next Phases

- **Phase 2**: Embedding & Indexing (`embedding.ts`, `indexing.ts`)
- **Phase 3**: Search & Retrieval (`search.ts`)
- **Phase 4**: CLI Interface (`cli.ts`)
- **Phase 5**: MCP Server (`mcp.ts`)

## Testing Strategy

- Unit tests with mocked external dependencies
- Integration test covering full workflow
- Auto-skip if Qdrant/Ollama unavailable