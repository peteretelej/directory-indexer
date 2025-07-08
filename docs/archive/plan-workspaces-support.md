# Directory Indexer: Workspace Support Design

## Overview

Add workspace support using search filtering. Workspaces are named groups of directory paths that filter search results from the existing single Qdrant collection.

## Configuration

### Environment Variables

```json
{
  "mcpServers": {
    "directory-indexer": {
      "command": "npx",
      "args": ["directory-indexer@latest", "serve"],
      "env": {
        "WORKSPACE_COOKING": "C:\\recipes,D:\\notes\\shopping",
        "WORKSPACE_WORK": "C:\\work\\docs,C:\\work\\cases",
        "WORKSPACE_LEARNING": "C:\\courses\\rust,C:\\courses\\python"
      }
    }
  }
}
```

### Parsing Logic

```typescript
interface WorkspaceConfig {
  [name: string]: string[];
}

function parseWorkspaces(env: Record<string, string>): WorkspaceConfig {
  const workspaces: WorkspaceConfig = {};

  for (const [key, value] of Object.entries(env)) {
    if (key.startsWith("WORKSPACE_")) {
      const name = key.replace("WORKSPACE_", "").toLowerCase();
      workspaces[name] = value.split(",").map((p) => p.trim());
    }
  }

  return workspaces;
}
```

## Storage

No changes to existing storage schema. Filter using existing `filePath` field in Qdrant payload:

```typescript
{
  id: "uuid",
  vector: [0.1, 0.2, ...],
  payload: {
    filePath: "/home/user/recipes/pasta.md",
    chunkId: "chunk_1",
    parentDirectories: ["/home/user", "/home/user/recipes"]
  }
}
```

## Search Implementation

```typescript
async function searchContent(
  query: string,
  options: { workspace?: string; limit?: number } = {}
): Promise<SearchResult[]> {
  const queryVector = await generateEmbedding(query);
  const allResults = await qdrant.searchPoints(queryVector, { limit: 1000 });

  // Filter by workspace if specified
  let filteredResults = allResults;
  if (options.workspace) {
    const workspacePaths = getWorkspacePaths(options.workspace);
    filteredResults = allResults.filter((result) =>
      workspacePaths.some((path) => result.payload.filePath.startsWith(path))
    );
  }

  return groupAndRankResults(filteredResults).slice(0, options.limit || 10);
}

function getWorkspacePaths(workspace: string): string[] {
  const config = getConfig();
  return config.workspaces[workspace] || [];
}
```

## MCP Tool Updates

### `search` Tool Schema

```typescript
{
  name: 'search',
  inputSchema: {
    type: 'object',
    properties: {
      query: { type: 'string' },
      workspace: { type: 'string' },
      limit: { type: 'number', default: 10 }
    },
    required: ['query']
  }
}
```

### `server_info` Response

```json
{
  "name": "directory-indexer",
  "version": "0.0.10",
  "status": {
    "directoriesIndexed": 3,
    "filesIndexed": 1247,
    "chunksIndexed": 3891,
    "databaseSize": "15.2 MB",
    "lastIndexed": "2025-01-15T10:30:00Z",
    "errors": []
  },
  "workspaces": [
    {
      "name": "cooking",
      "paths": ["C:\\recipes", "D:\\notes\\shopping"],
      "filesCount": 45,
      "chunksCount": 234
    },
    {
      "name": "work",
      "paths": ["C:\\work\\docs", "C:\\work\\cases"],
      "filesCount": 892,
      "chunksCount": 2156
    }
  ]
}
```

## Implementation Plan

### Phase 1: Configuration

```typescript
// File: src/config.ts
interface Config {
  // ... existing fields
  workspaces: WorkspaceConfig;
}

// Add parseWorkspaces() function
// Add getWorkspacePaths() helper
```

### Phase 2: Search Filtering

```typescript
// File: src/search.ts
// Update searchContent() to accept workspace parameter
// Add workspace filtering logic

// File: src/mcp.ts
// Update search tool schema
// Pass workspace parameter to searchContent()
```

### Phase 3: Statistics

```typescript
// File: src/storage.ts
// Add getWorkspaceStats() function

// File: src/mcp.ts
// Update server_info response to include workspaces array
```

### Phase 4: Testing

```typescript
// Tests:
// - Workspace configuration parsing
// - Search filtering by workspace
// - Overlapping workspace handling
// - Invalid workspace handling
```

## Edge Cases

### Overlapping Workspaces

```bash
"WORKSPACE_ALL_DOCS": "/home/user/docs"
"WORKSPACE_PROJECTS": "/home/user/docs/projects"

# File: /home/user/docs/projects/api.md appears in both workspaces
```

### Invalid Workspace

```typescript
{"workspace": "nonexistent"}
// Returns: [] (empty results)
```

### No Workspace Specified

```typescript
{"query": "pasta recipes"}
// Returns: Results from all indexed files
```
