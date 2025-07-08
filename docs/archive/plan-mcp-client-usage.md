# Improved MCP Documentation for src/mcp.ts

## Agent Strategy Guidance (Add to top of file as comments)

```typescript
/**
 * AGENT USAGE STRATEGY GUIDE
 *
 * Recommended workflow for MCP clients:
 * 1. START WITH search - Most queries should begin here
 * 2. PRIORITIZE TOP RESULTS - Focus on highest scoring matches first (>0.7 score)
 * 3. SMART CONTENT STRATEGY:
 *    - Small files (<2KB): Use get_content to read entirely
 *    - Large files (>2KB): Use get_chunk for specific relevant sections
 *    - Multiple relevant chunks: Consider get_content with chunk range
 * 4. EXPAND CONTEXT - Use similar_files to find related content when needed
 * 5. CHECK STATUS FIRST - Use server_info if unsure what's indexed
 */
```

## Enhanced Tool Descriptions

### search - Primary Discovery Tool

```typescript
{
  name: 'search',
  description: `Search indexed files using natural language queries. PRIMARY TOOL - start here for most user requests.

AGENT STRATEGY:
- Use this as your FIRST tool call for finding information
- Focus on top 3-5 results with similarity scores >0.7 for high relevance
- Results include file size hints to guide content retrieval strategy
- Each result shows chunk IDs for targeted content extraction

When to use this tool:
- Any request to find documentation, code, configs, or explanations
- Discovering content about specific topics, concepts, or functionality
- Initial exploration of what information is available

Content retrieval strategy:
- Files <2KB: Use get_content for full file reading
- Files >2KB with specific chunks: Use get_chunk with returned chunk IDs
- Multiple relevant chunks in same file: Use get_content with chunk range

Response format includes:
- filePath: Full path to file
- score: Relevance score (0-1, prioritize >0.7)
- fileSizeBytes: Size in bytes (helps decide content strategy)
- matchingChunks: Number of chunks that matched the query
- chunks: Array of {chunkId, score} for targeted retrieval

Example workflow:
1. search("API authentication")
2. Review top 3 results with scores >0.7
3. For small files: get_content(filePath)
4. For large files: get_chunk(filePath, highestScoringChunkId)
5. If needed: similar_files(topResult) to expand context`,
  inputSchema: {
    type: 'object',
    properties: {
      query: {
        type: 'string',
        description: 'Natural language search query. Be specific for better results.'
      },
      limit: {
        type: 'number',
        description: 'Maximum files to return (default: 10). Start with 5-10 for focused results.',
        default: 10
      }
    },
    required: ['query']
  }
}
```

### get_content - Smart Content Retrieval

```typescript
{
  name: 'get_content',
  description: `Retrieve file content with intelligent sizing strategy. Use for complete file reading or targeted chunk extraction.

AGENT STRATEGY:
- Small files (<2KB): Read entirely for full context
- Medium files (2-10KB): Consider chunk ranges if search provided specific chunks
- Large files (>10KB): Only use for specific chunk ranges or when complete file needed
- Always check file size from search results before deciding

When to use this tool:
- Reading complete documentation, configuration files, or small code files
- Getting specific sections when you have chunk ranges from search
- Full file analysis when chunks aren't sufficient

Chunk specification:
- Use chunk ranges like "2-5" to get chunks 2, 3, 4, and 5
- Single chunks like "3" for just one section
- Chunks are sequential per file: 0, 1, 2, etc.`,
  inputSchema: {
    type: 'object',
    properties: {
      file_path: {
        type: 'string',
        description: 'File path from search results. Can be absolute or relative.'
      },
      chunks: {
        type: 'string',
        description: 'Optional chunk range. Examples: "3" (single), "2-5" (range)'
      }
    },
    required: ['file_path']
  }
}
```

### get_chunk - Precise Content Extraction

```typescript
{
  name: 'get_chunk',
  description: `Get specific chunk content from search results. Use for precise content extraction from large files.

AGENT STRATEGY:
- Use when search returns specific high-scoring chunks (score >0.7)
- More efficient than get_content for large files when you need specific sections
- Get highest-scoring chunk first; if incomplete, try adjacent chunks

When to use this tool:
- Search returned specific chunk IDs with high scores
- Large file (>10KB) where you only need specific sections
- Extracting precise information without full file context

Chunk IDs are sequential per file (0, 1, 2, 3...) and correspond to ~500-word sections.`,
  inputSchema: {
    type: 'object',
    properties: {
      file_path: {
        type: 'string',
        description: 'File path containing the chunk (from search results)'
      },
      chunk_id: {
        type: 'string',
        description: 'Chunk ID from search results (sequential: "0", "1", "2"...)'
      }
    },
    required: ['file_path', 'chunk_id']
  }
}
```

### similar_files - Context Expansion

```typescript
{
  name: 'similar_files',
  description: `Find files similar to a reference file. Use to expand context after finding initial relevant content.

AGENT STRATEGY:
- Use AFTER search finds relevant files, not as initial discovery
- Great for finding related documentation, alternative implementations, or patterns
- Most useful with high-quality reference files (clear topic, good content)

When to use this tool:
- After search identifies a highly relevant file (score >0.8)
- Building comprehensive responses that need multiple examples
- Finding alternative approaches or implementations
- Discovering related documentation or guides

Response format includes:
- filePath: Full path to similar file
- score: Similarity score (0-1, prioritize >0.7)
- fileSizeBytes: Size in bytes (helps decide content strategy)`,
  inputSchema: {
    type: 'object',
    properties: {
      file_path: {
        type: 'string',
        description: 'Reference file path (from search results). Must be indexed.'
      },
      limit: {
        type: 'number',
        description: 'Max similar files (default: 10).',
        default: 10
      }
    },
    required: ['file_path']
  }
}
```

### server_info - Status and Discovery

```typescript
{
  name: 'server_info',
  description: `Get server status and indexed content overview. Use to understand scope before searching.

AGENT STRATEGY:
- Call this FIRST if user asks about capabilities or available content
- Use before search when unsure what's indexed
- Check system health if search/indexing seems problematic

When to use this tool:
- User asks "What can you search?" or "What's indexed?"
- Before searching if unsure about available content
- Troubleshooting search issues or empty results

Information provided:
- Total indexed files, directories, and chunks
- List of indexed directories with file counts
- Database size and last indexing time
- Any errors or issues with indexing`,
  inputSchema: {
    type: 'object',
    properties: {},
    additionalProperties: false
  }
}
```

## Additional Agent Guidelines

### Content Retrieval Strategy

- **Small files (<2KB)**: Read entirely with get_content
- **Medium files (2-10KB)**: Use chunks if search provided specific sections
- **Large files (>10KB)**: Use targeted chunks only

### Score Prioritization

- **>0.8**: High priority, definitely relevant
- **0.7-0.8**: Good relevance, check first
- **0.5-0.7**: Medium relevance, consider if needed
- **<0.5**: Low relevance, usually skip

### Recommended Tool Sequences

1. **Discovery**: search() → get_content()/get_chunk() → similar_files() if needed
2. **Status check**: server_info() → search() → content retrieval
3. **Comprehensive**: search() → multiple targeted retrievals → similar_files() for expansion
