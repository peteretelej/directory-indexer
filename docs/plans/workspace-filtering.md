# Workspace Filtering Design

## Problem Statement

Currently, workspace filtering uses post-filtering approach:
1. Search entire vector database for query
2. Get large result set (e.g., `limit * 5`)
3. Filter results by workspace paths after retrieval
4. Return top results

This is inefficient because:
- Searches entire database regardless of workspace size
- Wastes computation on irrelevant results
- Requires larger search limits to compensate for filtering
- No performance benefit over single-collection approach

## Goal

Implement true Qdrant-level filtering so workspace searches only examine relevant chunks, improving both performance and accuracy.

## Current Data Structure

Each point in Qdrant has this payload:
```typescript
{
  filePath: string;           // "/home/user/work/notes/meeting.md"
  chunkId: string;           // "0", "1", "2"
  fileHash: string;          // "abc123..."
  content: string;           // actual chunk text
  parentDirectories: string[]; // ["/home/", "/home/user/", "/home/user/work/", "/home/user/work/notes/"]
}
```

## Qdrant Filtering Capabilities Analysis

Based on the Qdrant API documentation, we have these relevant filtering options:

### 1. Match Any (IN operator)
```json
{
  "key": "parentDirectories",
  "match": { "any": ["/home/user/work/", "/home/user/personal/"] }
}
```

### 2. Nested Array Filtering
Since `parentDirectories` is an array, Qdrant can efficiently filter chunks that have any of the workspace directories as a parent.

### 3. No Native Prefix Matching
Qdrant doesn't have built-in "starts with" filtering for strings, but we can work around this using the existing `parentDirectories` array.

## Proposed Solutions

### Solution 1: Use Existing parentDirectories (Recommended)

**Implementation:**
```typescript
function buildWorkspaceFilter(workspacePaths: string[]): Record<string, unknown> {
  return {
    must: [
      {
        key: "parentDirectories",
        match: { any: workspacePaths }
      }
    ]
  };
}

// In search.ts
const filter = workspace ? buildWorkspaceFilter(workspacePaths) : undefined;
const points = await qdrant.searchPoints(queryEmbedding, limit, filter);
```

**Pros:**
- Uses existing data structure
- No reindexing required
- Efficient Qdrant array filtering
- Clean and simple implementation

**Cons:**
- Requires exact directory path matches
- Won't work for subdirectory workspaces (e.g., workspace="/home/user/work/projects" but indexed "/home/user/work/")

### Solution 2: Add Workspace Keys

**Implementation:**
```typescript
// Enhanced payload structure
{
  filePath: string;
  chunkId: string;
  fileHash: string;
  content: string;
  parentDirectories: string[];
  workspaceKeys: string[];  // ["workspace_work", "workspace_personal"]
}

// When indexing, calculate workspace membership
function calculateWorkspaceKeys(filePath: string, config: Config): string[] {
  const keys: string[] = [];
  for (const [workspaceName, workspacePaths] of Object.entries(config.workspaces)) {
    if (isFileInWorkspace(filePath, workspacePaths)) {
      keys.push(`workspace_${workspaceName}`);
    }
  }
  return keys;
}

// Filter by workspace
function buildWorkspaceFilter(workspaceName: string): Record<string, unknown> {
  return {
    must: [
      {
        key: "workspaceKeys",
        match: { value: `workspace_${workspaceName}` }
      }
    ]
  };
}
```

**Pros:**
- Explicit workspace membership
- Handles complex workspace hierarchies
- Very fast filtering (exact key matches)
- Supports overlapping workspaces

**Cons:**
- Requires full reindexing
- More storage overhead
- Workspace changes require reindexing

### Solution 3: Enhanced Directory Keys

**Implementation:**
```typescript
// Add normalized directory hierarchy
{
  filePath: string;
  chunkId: string;
  fileHash: string;
  content: string;
  parentDirectories: string[];
  directoryKeys: string[];  // ["d_home", "d_home_user", "d_home_user_work"]
}

// Generate directory keys for efficient filtering
function generateDirectoryKeys(filePath: string): string[] {
  const parts = normalizePath(filePath).split('/').filter(Boolean);
  const keys: string[] = [];
  let currentPath = '';
  
  for (const part of parts.slice(0, -1)) { // exclude filename
    currentPath += '/' + part;
    keys.push('d_' + currentPath.replace(/[^a-zA-Z0-9]/g, '_'));
  }
  
  return keys;
}
```

**Pros:**
- Very flexible filtering
- Supports any directory hierarchy
- Fast key-based filtering

**Cons:**
- Requires reindexing
- Complex key generation logic
- Storage overhead

## Recommended Approach

**Use Solution 1 (parentDirectories) with fallback to post-filtering**

```typescript
async function searchContent(query: string, options: SearchOptions = {}): Promise<SearchResult[]> {
  const { limit = 10, threshold = 0.0, workspace } = options;
  
  // ... setup code ...
  
  let points: QdrantPoint[];
  
  if (workspace && workspacePaths.length > 0) {
    // Try Qdrant filtering first
    const filter = buildWorkspaceFilter(workspacePaths);
    points = await qdrant.searchPoints(queryEmbedding, limit * 2, filter);
    
    // If insufficient results, fall back to post-filtering
    if (points.length < limit && points.length < limit * 2) {
      console.debug('Insufficient results from Qdrant filter, falling back to post-filtering');
      points = await qdrant.searchPoints(queryEmbedding, limit * 5);
      points = points.filter(point => 
        isFileInWorkspace(point.payload.filePath, workspacePaths)
      );
    }
  } else {
    // No workspace filtering
    points = await qdrant.searchPoints(queryEmbedding, limit * 2);
  }
  
  // ... rest of processing ...
}

function buildWorkspaceFilter(workspacePaths: string[]): Record<string, unknown> {
  // Normalize paths to match parentDirectories format
  const normalizedPaths = workspacePaths.map(path => {
    const normalized = normalizePath(path);
    return normalized.endsWith('/') ? normalized : normalized + '/';
  });
  
  return {
    must: [
      {
        key: "parentDirectories", 
        match: { any: normalizedPaths }
      }
    ]
  };
}
```

## Migration Strategy

### Phase 1: Implement Qdrant Filtering
1. Add `buildWorkspaceFilter()` function
2. Modify `searchPoints()` calls to use filter
3. Add fallback to post-filtering for edge cases
4. Test with existing data

### Phase 2: Optimize (Optional)
1. If parentDirectories approach proves insufficient
2. Implement Solution 2 (workspace keys)
3. Add migration command to reindex with workspace keys

### Phase 3: Remove Post-filtering
1. Once Qdrant filtering is proven reliable
2. Remove post-filtering fallback
3. Simplify search logic

## Performance Impact

**Expected improvements:**
- 60-90% reduction in search computation for workspace queries
- More accurate results (no arbitrary search limit multipliers)
- Better scalability as collection size grows

**Monitoring:**
- Track filter hit rates
- Compare search times before/after
- Monitor result quality

## Testing Strategy

1. **Unit tests:** Filter generation logic
2. **Integration tests:** Workspace filtering accuracy
3. **Performance tests:** Search time comparisons
4. **Edge cases:** Empty workspaces, invalid paths, overlapping workspaces

## Implementation Notes

### Workspace Path Normalization
Ensure workspace paths match the format stored in `parentDirectories`:
```typescript
// Current parentDirectories format: ["/home/", "/home/user/", "/home/user/work/"]
// Workspace paths need trailing slash for exact matching
```

### Fallback Strategy
Keep post-filtering as fallback for:
- Complex workspace hierarchies not covered by parentDirectories
- Edge cases where Qdrant filtering returns insufficient results
- Backward compatibility during migration

### Configuration
Add setting to control filtering strategy:
```json
{
  "search": {
    "workspaceFiltering": "qdrant", // "qdrant" | "post" | "hybrid"
    "fallbackThreshold": 0.5 // Fall back to post-filtering if < 50% of expected results
  }
}
```

This approach provides immediate performance benefits while maintaining reliability through fallback mechanisms.