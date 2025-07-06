import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { 
  CallToolRequestSchema, 
  ListToolsRequestSchema,
  Tool
} from '@modelcontextprotocol/sdk/types.js';
import { readFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { Config } from './config.js';
import { indexDirectories } from './indexing.js';
import { searchContent, findSimilarFiles, getFileContent, getChunkContent } from './search.js';
import { getIndexStatus } from './storage.js';

// Read version from package.json
const __dirname = dirname(fileURLToPath(import.meta.url));
const packageJsonPath = join(__dirname, '../package.json');
const packageJson = JSON.parse(readFileSync(packageJsonPath, 'utf-8'));
const VERSION = packageJson.version;

const MCP_TOOLS: Tool[] = [
  {
    name: 'index',
    description: `Index directories for AI-powered semantic search. This tool processes files in specified directories, extracts text content, generates vector embeddings, and stores them for semantic search capabilities.

When to use this tool:
- Before performing any search operations on new directories
- When you want to add new code repositories, documentation, or text files to the searchable knowledge base
- To update the index when files have been modified (the tool automatically detects and reprocesses changed files)
- When setting up semantic search for a project or workspace

What this tool does:
- Recursively scans directories for supported file types (code, markdown, text, config files)
- Chunks large files into smaller segments for better search precision
- Generates vector embeddings using the configured embedding model
- Stores file metadata and embeddings in a local database
- Skips unchanged files on re-indexing for efficiency
- Supports overlapping directory paths (files are deduplicated automatically)

Supported file types: .md, .txt, .py, .js, .ts, .go, .rs, .java, .json, .yaml, .toml, .env, .conf, and many others

Performance note: Initial indexing may take time for large directories, but subsequent re-indexing is much faster as only changed files are reprocessed.`,
    inputSchema: {
      type: 'object',
      properties: {
        directory_path: {
          type: 'string',
          description: 'Comma-separated list of absolute or relative directory paths to index. Examples: "/home/user/projects" or "./src,./docs,./tests"'
        }
      },
      required: ['directory_path']
    }
  },
  {
    name: 'search',
    description: `Perform semantic search across indexed files using natural language queries. This tool uses vector similarity to find the most relevant content, going beyond simple keyword matching to understand intent and context.

When to use this tool:
- Finding code examples, functions, or patterns ("error handling in Python", "JWT authentication implementation")
- Locating documentation or explanations ("how to configure Redis", "API rate limiting guide")
- Discovering similar functionality across files ("database connection patterns", "logging utilities")
- Research and exploration of codebases ("machine learning models", "test utilities")
- Finding files related to specific features or topics

How semantic search works:
- Searches by meaning and context, not just exact keywords
- Finds conceptually related content even with different terminology
- Returns files ranked by relevance with similarity scores
- Groups results by file to avoid duplicates from multiple matching sections

Response format:
- Returns lightweight metadata including file paths, relevance scores, and chunk IDs
- Use 'get_chunk' or 'get_content' tools to fetch actual content from search results
- Chunks are sorted by relevance score within each file
- Average similarity score calculated across all matching chunks per file

Example queries:
- "error handling patterns" (finds try/catch, error classes, logging)
- "database migration scripts" (finds SQL, schema changes, migration files)
- "authentication middleware" (finds auth logic, JWT handling, middleware functions)`,
    inputSchema: {
      type: 'object',
      properties: {
        query: {
          type: 'string',
          description: 'Natural language search query describing what you are looking for. Can be concepts, functionality, or specific technical terms.'
        },
        limit: {
          type: 'number',
          description: 'Maximum number of files to return (default: 10). Each file may contain multiple matching chunks.',
          default: 10
        }
      },
      required: ['query']
    }
  },
  {
    name: 'similar_files',
    description: `Find files that are semantically similar to a given reference file. This tool analyzes the content and context of a file to discover other files with related functionality, similar patterns, or comparable content.

When to use this tool:
- Discovering related implementations across a codebase ("find files similar to this authentication module")
- Locating alternative approaches or patterns ("find other components like this React component")
- Finding documentation or examples related to a specific file
- Identifying code duplication or similar functionality that could be refactored
- Exploring unfamiliar codebases by finding files similar to known examples
- Locating test files, configuration files, or documentation related to a source file

How similarity detection works:
- Analyzes the semantic content of the reference file
- Compares against all indexed files using vector similarity
- Considers code patterns, function signatures, imports, and documentation
- Returns files ranked by content similarity, not just filename or location similarity
- Works across different file types and programming languages

Use cases:
- Code analysis: "Find files similar to this database model to understand the schema patterns"
- Learning: "Show me other API controllers similar to this one"
- Maintenance: "Find files with similar error handling patterns"
- Architecture: "Locate other services that follow this microservice pattern"

Note: The reference file must be indexed for this tool to work. If the file is not found in the index, an error will be returned.`,
    inputSchema: {
      type: 'object',
      properties: {
        file_path: {
          type: 'string',
          description: 'Absolute or relative path to the reference file. This file must have been previously indexed.'
        },
        limit: {
          type: 'number',
          description: 'Maximum number of similar files to return (default: 10). Results are sorted by similarity score.',
          default: 10
        }
      },
      required: ['file_path']
    }
  },
  {
    name: 'get_content',
    description: `Retrieve the full content of a file or specific chunks within a file. This tool reads files directly from the filesystem and can optionally return only specific portions of indexed files.

When to use this tool:
- After performing a search, to retrieve the actual content of relevant files
- Reading complete files that were identified through semantic search
- Extracting specific sections of large files using chunk ranges
- Accessing source code, documentation, or configuration files for analysis
- Following up on search results with detailed content examination

How chunk selection works:
- If no chunks parameter is provided, returns the entire file content
- Chunk ranges allow selective reading of large files (e.g., "2-5" returns chunks 2, 3, 4, and 5)
- Single chunks can be specified (e.g., "3" returns only chunk 3)
- Chunks are the same segments created during indexing for semantic search
- Useful for large files where you only need specific sections identified by search

File access:
- Reads files directly from the filesystem (not from the search index)
- Works with any readable file, whether indexed or not
- Supports all text-based file formats
- Preserves original formatting and content exactly as stored

Workflow integration:
1. Use 'search' to find relevant files and identify interesting chunk IDs
2. Use 'get_content' to retrieve full file content or specific chunks
3. Analyze the content to understand context and implementation details

Performance note: For large files, using chunk ranges can be more efficient than reading entire files.`,
    inputSchema: {
      type: 'object',
      properties: {
        file_path: {
          type: 'string',
          description: 'Absolute or relative path to the file to retrieve. File must be readable and text-based.'
        },
        chunks: {
          type: 'string',
          description: 'Optional chunk range specification. Examples: "3" (single chunk), "2-5" (chunks 2 through 5), "1-3" (first three chunks). Only works for indexed files.'
        }
      },
      required: ['file_path']
    }
  },
  {
    name: 'get_chunk',
    description: `Retrieve the content of a specific chunk from an indexed file. This tool provides precise access to individual text segments that were identified during semantic search, allowing efficient retrieval of only the most relevant content.

When to use this tool:
- After performing a 'search' operation, to fetch the actual content of specific chunks that matched your query
- When you want to examine only the most relevant sections of a file rather than reading the entire file
- For targeted content analysis where you need specific text segments identified by their chunk IDs
- To build contextual responses using only the most semantically relevant portions of files
- When working with large files and you only need particular sections

How chunks work:
- Files are divided into overlapping text segments during indexing for better search granularity
- Each chunk represents a coherent section of text (typically 512 characters with overlap)
- Chunk IDs are sequential strings ("0", "1", "2", etc.) within each file
- Search results include chunk IDs for the most relevant sections
- This tool retrieves the exact content that was semantically matched

Typical workflow:
1. Use 'search' to find files and get chunk IDs with high relevance scores
2. Use 'get_chunk' to retrieve the specific content of the most relevant chunks
3. Analyze or process only the most pertinent text segments

Efficiency benefits:
- Avoids transferring unnecessary content from large files
- Provides precise access to semantically relevant text
- Reduces token usage by fetching only needed sections
- Enables focused analysis on the most important content

Note: Both the file and the specific chunk must exist in the search index for this tool to work.`,
    inputSchema: {
      type: 'object',
      properties: {
        file_path: {
          type: 'string',
          description: 'Absolute or relative path to the indexed file containing the desired chunk.'
        },
        chunk_id: {
          type: 'string',
          description: 'ID of the specific chunk to retrieve. This is typically obtained from search results and is a sequential string like "0", "1", "2", etc.'
        }
      },
      required: ['file_path', 'chunk_id']
    }
  },
  {
    name: 'server_info',
    description: `Get comprehensive information about the directory indexer server status, configuration, and indexed content. This tool provides a complete overview of the current state of the semantic search system.

When to use this tool:
- To check if the indexer is properly set up and operational
- Before starting work to understand what content is already indexed
- To verify indexing operations completed successfully
- When debugging search issues or unexpected results
- To get an overview of available content for semantic search
- To check system health and identify any configuration problems

Information provided:
- Server version and operational status
- Total count of indexed directories, files, and searchable chunks
- Database size and storage information
- Most recent indexing timestamp
- List of all indexed directories with individual statistics
- File counts and chunk counts per directory
- Indexing status for each directory (completed, failed, in progress)
- Error reports and processing issues
- System consistency checks between database components

Status indicators:
- Operational status of vector database (Qdrant) connection
- Embedding service availability
- Data consistency between SQLite metadata and vector storage
- Recent errors or warnings that may affect search quality

Use this tool to:
- Verify setup before performing search operations
- Understand the scope of available content
- Troubleshoot search or indexing issues
- Plan additional indexing operations
- Monitor system health and performance`,
    inputSchema: {
      type: 'object',
      properties: {},
      additionalProperties: false
    }
  }
];

export async function startMcpServer(config: Config): Promise<void> {
  const server = new Server(
    {
      name: 'directory-indexer',
      version: VERSION
    },
    {
      capabilities: {
        tools: {}
      }
    }
  );

  server.setRequestHandler(ListToolsRequestSchema, async () => {
    return {
      tools: MCP_TOOLS
    };
  });

  server.setRequestHandler(CallToolRequestSchema, async (request) => {
    const { name, arguments: args } = request.params;

    try {
      switch (name) {
        case 'index': {
          if (!args || typeof args.directory_path !== 'string') {
            throw new Error('directory_path is required');
          }
          const paths = args.directory_path.split(',').map((p: string) => p.trim());
          const result = await indexDirectories(paths, config);
          return {
            content: [
              {
                type: 'text',
                text: `Indexed ${result.indexed} files, skipped ${result.skipped} files, ${result.errors.length} errors`
              }
            ]
          };
        }

        case 'search': {
          if (!args || typeof args.query !== 'string') {
            throw new Error('query is required');
          }
          const results = await searchContent(args.query, { limit: (args.limit as number) || 10 });
          return {
            content: [
              {
                type: 'text',
                text: JSON.stringify(results, null, 2)
              }
            ]
          };
        }

        case 'similar_files': {
          if (!args || typeof args.file_path !== 'string') {
            throw new Error('file_path is required');
          }
          const results = await findSimilarFiles(args.file_path, (args.limit as number) || 10);
          return {
            content: [
              {
                type: 'text',
                text: JSON.stringify(results, null, 2)
              }
            ]
          };
        }

        case 'get_content': {
          if (!args || typeof args.file_path !== 'string') {
            throw new Error('file_path is required');
          }
          const content = await getFileContent(args.file_path, args.chunks as string);
          return {
            content: [
              {
                type: 'text',
                text: content
              }
            ]
          };
        }

        case 'get_chunk': {
          if (!args || typeof args.file_path !== 'string' || typeof args.chunk_id !== 'string') {
            throw new Error('file_path and chunk_id are required');
          }
          const content = await getChunkContent(args.file_path, args.chunk_id);
          return {
            content: [
              {
                type: 'text',
                text: content
              }
            ]
          };
        }

        case 'server_info': {
          const status = await getIndexStatus();
          return {
            content: [
              {
                type: 'text',
                text: JSON.stringify({
                  name: 'directory-indexer',
                  version: VERSION,
                  status: status
                }, null, 2)
              }
            ]
          };
        }

        default:
          throw new Error(`Unknown tool: ${name}`);
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      return {
        content: [
          {
            type: 'text',
            text: `Error: ${errorMessage}`
          }
        ],
        isError: true
      };
    }
  });

  const transport = new StdioServerTransport();
  await server.connect(transport);
  
  if (config.verbose) {
    console.error('MCP server started successfully');
  }
}