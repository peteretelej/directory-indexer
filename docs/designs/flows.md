# Directory Indexer Flow Diagrams

This document describes the key operational flows in the directory indexer system.

## System Architecture

```mermaid
graph TB
    CLI[CLI Interface] --> Engine[Indexing Engine]
    CLI --> Search[Search Engine]
    CLI --> MCP[MCP Server]
    
    Engine --> SQLite[(SQLite DB)]
    Engine --> Qdrant[(Qdrant Vector Store)]
    Engine --> Files[File System]
    
    Search --> SQLite
    Search --> Qdrant
    Search --> Embed[Embedding Provider]
    
    Engine --> Embed
    MCP --> Engine
    MCP --> Search
    
    Embed --> Ollama[Ollama API]
    Embed --> OpenAI[OpenAI API]
```

## Indexing Flow

### Complete Directory Indexing Process

```mermaid
flowchart TD
    START([CLI: index ~/docs]) --> VALIDATE{Input Validation}
    VALIDATE -->|Invalid| ERROR1[Error: Invalid path]
    VALIDATE -->|Valid| CONVERT[Convert to absolute path]
    
    CONVERT --> CHECK{Directory indexed?}
    CHECK -->|Yes| UPDATE[Update mode]
    CHECK -->|No| SCAN[Full scan mode]
    
    UPDATE --> WALK1[Walk directory tree]
    SCAN --> WALK2[Walk directory tree]
    
    WALK1 --> FILTER1[Apply ignore patterns]
    WALK2 --> FILTER2[Apply ignore patterns]
    
    FILTER1 --> META1[Extract file metadata]
    FILTER2 --> META2[Extract file metadata]
    
    META1 --> COMPARE{Compare with SQLite}
    META2 --> PROCESS[Process all files]
    
    COMPARE -->|Unchanged| SKIP[Skip file]
    COMPARE -->|Changed| PROCESS
    COMPARE -->|New| PROCESS
    
    PROCESS --> READ[Read file content]
    READ --> SIZE{Check file size}
    SIZE -->|Too large| SKIP2[Skip with warning]
    SIZE -->|OK| CHUNK[Chunk content]
    
    CHUNK --> EMBED[Generate embeddings]
    EMBED -->|Success| STORE[Store in SQLite + Qdrant]
    EMBED -->|Failure| ERROR2[Record error in SQLite]
    
    STORE --> NEXT{More files?}
    SKIP --> NEXT
    SKIP2 --> NEXT
    ERROR2 --> NEXT
    
    NEXT -->|Yes| PROCESS
    NEXT -->|No| COMPLETE[Update directory status]
    
    COMPLETE --> END([Indexing complete])
    ERROR1 --> END
```

### File Processing Details

```mermaid
flowchart TD
    FILE[File detected] --> META[Extract metadata]
    META --> HASH[Calculate file hash]
    HASH --> COMPARE{Compare with DB}
    
    COMPARE -->|Same hash & mtime| SKIP[Skip processing]
    COMPARE -->|Different| DELETE[Delete old vectors]
    
    DELETE --> READ[Read file content]
    READ --> TYPE{File type supported?}
    TYPE -->|No| SKIP2[Skip unsupported type]
    TYPE -->|Yes| CHUNK[Split into chunks]
    
    CHUNK --> EMBED_LOOP[For each chunk]
    EMBED_LOOP --> CALL_API[Call embedding API]
    CALL_API -->|Success| VECTOR[Store vector point]
    CALL_API -->|Failure| LOG_ERROR[Log embedding error]
    
    VECTOR --> MORE{More chunks?}
    LOG_ERROR --> MORE
    MORE -->|Yes| EMBED_LOOP
    MORE -->|No| SAVE_META[Save file metadata]
    
    SAVE_META --> DONE[File complete]
    SKIP --> DONE
    SKIP2 --> DONE
```

## Search Flow

### Semantic Search Process

```mermaid
flowchart TD
    START([User: search "Redis timeout"]) --> EMBED[Generate query embedding]
    EMBED -->|Success| SEARCH[Vector search in Qdrant]
    EMBED -->|Failure| ERROR[Error: Embedding failed]
    
    SEARCH --> RESULTS{Results found?}
    RESULTS -->|No| EMPTY[Return empty results]
    RESULTS -->|Yes| ENRICH[Fetch metadata from SQLite]
    
    ENRICH --> FILTER{Directory filter?}
    FILTER -->|Yes| SCOPE[Apply directory scoping]
    FILTER -->|No| RANK[Rank by similarity score]
    
    SCOPE --> RANK
    RANK --> PREVIEW[Generate content previews]
    PREVIEW --> FORMAT[Format results with metadata]
    FORMAT --> RETURN[Return ranked file list]
    
    RETURN --> END([Search complete])
    EMPTY --> END
    ERROR --> END
```

### Content Retrieval Flow

```mermaid
flowchart TD
    START([get_content request]) --> VALIDATE[Validate file path]
    VALIDATE -->|Invalid| ERROR1[Error: File not found]
    VALIDATE -->|Valid| META[Get file metadata from SQLite]
    
    META -->|Not found| ERROR2[Error: File not indexed]
    META -->|Found| CHUNKS{Chunk range specified?}
    
    CHUNKS -->|No| READ_ALL[Read full file content]
    CHUNKS -->|Yes| PARSE[Parse chunk range]
    
    PARSE -->|Invalid range| ERROR3[Error: Invalid chunk range]
    PARSE -->|Valid| GET_CHUNKS[Extract specific chunks]
    
    READ_ALL --> FORMAT1[Format full content]
    GET_CHUNKS --> FORMAT2[Format selected chunks]
    
    FORMAT1 --> RETURN[Return content]
    FORMAT2 --> RETURN
    
    RETURN --> END([Content retrieved])
    ERROR1 --> END
    ERROR2 --> END
    ERROR3 --> END
```

## MCP Server Flow

### MCP Tool Interaction

```mermaid
sequenceDiagram
    participant AI as AI Assistant
    participant MCP as MCP Server
    participant Engine as Indexing Engine
    participant SQLite as SQLite DB
    participant Qdrant as Qdrant
    participant FS as File System
    
    Note over AI,FS: Indexing Request
    AI->>MCP: index(directory_paths)
    MCP->>Engine: index_directories()
    Engine->>FS: Walk directory tree
    FS-->>Engine: File list + metadata
    Engine->>SQLite: Check existing files
    SQLite-->>Engine: File status
    Engine->>FS: Read file content
    FS-->>Engine: File content
    Engine->>Engine: Generate embeddings
    Engine->>SQLite: Store file metadata
    Engine->>Qdrant: Store embeddings
    Engine-->>MCP: Indexing stats
    MCP-->>AI: Success response
    
    Note over AI,FS: Search Request
    AI->>MCP: search(query, options)
    MCP->>Engine: generate_embedding()
    Engine-->>MCP: Query vector
    MCP->>Qdrant: Vector similarity search
    Qdrant-->>MCP: Similar chunks
    MCP->>SQLite: Fetch file metadata
    SQLite-->>MCP: File details
    MCP->>MCP: Rank and format results
    MCP-->>AI: Search results
    
    Note over AI,FS: Content Request
    AI->>MCP: get_content(file_path, chunks)
    MCP->>SQLite: Get file metadata
    SQLite-->>MCP: File info + chunks
    MCP->>FS: Read file content
    FS-->>MCP: File content
    MCP->>MCP: Extract requested chunks
    MCP-->>AI: File content
```

## Error Handling Flows

### File Processing Error Recovery

```mermaid
flowchart TD
    PROCESS[Processing file] --> ERROR{Error occurred?}
    ERROR -->|No| SUCCESS[Process complete]
    ERROR -->|Yes| TYPE{Error type?}
    
    TYPE -->|Read error| LOG1[Log: Cannot read file]
    TYPE -->|Size error| LOG2[Log: File too large]
    TYPE -->|Encoding error| LOG3[Log: Invalid encoding]
    TYPE -->|Embedding error| LOG4[Log: Embedding failed]
    
    LOG1 --> RECORD1[Record in SQLite errors_json]
    LOG2 --> RECORD2[Record in SQLite errors_json]
    LOG3 --> RECORD3[Record in SQLite errors_json]
    LOG4 --> RECORD4[Record in SQLite errors_json]
    
    RECORD1 --> CONTINUE[Continue with next file]
    RECORD2 --> CONTINUE
    RECORD3 --> CONTINUE
    RECORD4 --> CONTINUE
    
    CONTINUE --> NEXT[Process next file]
    SUCCESS --> NEXT
```

### Service Connectivity Flow

```mermaid
flowchart TD
    START[Operation request] --> CHECK_SQLITE{SQLite available?}
    CHECK_SQLITE -->|No| FAIL1[Fail: Database unavailable]
    CHECK_SQLITE -->|Yes| OPERATION{Operation type?}
    
    OPERATION -->|Index/Search| CHECK_SERVICES[Check Qdrant + Embedding]
    OPERATION -->|Get content| SQLITE_ONLY[SQLite operations only]
    OPERATION -->|Status| STATUS_CHECK[Check all services]
    
    CHECK_SERVICES --> QDRANT{Qdrant available?}
    QDRANT -->|No| FAIL2[Fail: Vector store unavailable]
    QDRANT -->|Yes| EMBEDDING{Embedding provider available?}
    
    EMBEDDING -->|No| FAIL3[Fail: Embedding service unavailable]
    EMBEDDING -->|Yes| PROCEED[Proceed with operation]
    
    SQLITE_ONLY --> PROCEED2[Execute SQLite operation]
    STATUS_CHECK --> GATHER[Gather service status]
    GATHER --> REPORT[Report status]
    
    PROCEED --> SUCCESS[Operation successful]
    PROCEED2 --> SUCCESS
    REPORT --> SUCCESS
    
    FAIL1 --> END[Operation failed]
    FAIL2 --> END
    FAIL3 --> END
    SUCCESS --> END
```