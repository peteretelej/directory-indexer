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
    description: `Index directories to make their files searchable. Processes files to create vector embeddings for semantic search.

When to use this tool:
- User specifically requests indexing a directory as a knowledge base
- Adding new documentation, code repositories, or file collections to search
- Updating index when many files have changed

How it works:
- Recursively scans directories for supported file types
- Extracts text content and splits into chunks
- Generates vector embeddings for semantic similarity
- Stores in database for fast retrieval

Examples:
- Index documentation: "/home/user/docs/project-wiki"
- Index codebase: "/home/user/projects/api-server"
- Index multiple directories: "/home/user/docs,/home/user/configs"

Indexing can take several minutes for large directories. Most users will already have directories indexed and can directly use search tool. Use server_info to check current indexing status first.`,
    inputSchema: {
      type: 'object',
      properties: {
        directory_path: {
          type: 'string',
          description: 'Comma-separated list of absolute directory paths to index. Must be absolute paths since MCP server runs independently. Examples: "/home/user/projects" (Unix) or "C:\\Users\\user\\projects" (Windows)'
        }
      },
      required: ['directory_path']
    }
  },
  {
    name: 'search',
    description: `Search indexed files using natural language queries. Finds files containing content semantically similar to the query.

When to use this tool:
- Find documentation, guides, or explanations about specific topics
- Locate code files implementing certain functionality or patterns
- Discover configuration files, scripts, or settings related to a topic
- Search for files covering specific concepts or technologies

How it works:
- Converts query to vector embedding using semantic similarity
- Searches all indexed file chunks for relevant content
- Groups results by file and calculates average relevance scores
- Returns files ranked by relevance score

Examples:
- "database configuration and connection pooling setup" - finds config files, documentation about DB setup
- "comprehensive error handling patterns and exception management" - finds code files with exception handling
- "JWT authentication implementation and session management" - finds auth-related code and docs
- "REST API documentation and endpoint specifications" - finds API guides, endpoint definitions
- "Docker deployment scripts and CI/CD pipeline configuration" - finds deployment automation

Returns files with similarity scores and chunk information. Use get_content to retrieve full file content or get_chunk to retrieve specific chunk content by chunk ID.
- Groups results by file to avoid duplicates from multiple matching sections

Response format:
- Returns lightweight metadata including file paths, relevance scores, and chunk IDs
- Use 'get_chunk' or 'get_content' tools to fetch actual content from search results
- Chunks are sorted by relevance score within each file
- Average similarity score calculated across all matching chunks per file

Example queries:
- "error handling patterns and exception management strategies" (finds try/catch, error classes, logging)
- "database migration scripts and schema versioning approaches" (finds SQL, schema changes, migration files)
- "authentication middleware and JWT token validation logic" (finds auth logic, JWT handling, middleware functions)`,
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
        },
        workspace: {
          type: 'string',
          description: 'Optional workspace name to filter search results. Only files within the workspace directories will be searched. Use server_info to see available workspaces.'
        }
      },
      required: ['query']
    }
  },
  {
    name: 'similar_files',
    description: `Find files with content similar to a reference file. Uses semantic similarity to find related documents, code files, or any text content.

When to use this tool:
- Find documentation similar to a specific guide or README
- Locate related code files, configuration files, or scripts
- Discover alternative implementations or approaches
- Find files covering similar topics or concepts

How it works:
- Analyzes the semantic content of the reference file
- Compares against all indexed files using vector similarity
- Returns files ranked by content similarity score

Examples:
- Given "deployment-guide.md" - finds other deployment docs, CI/CD guides, infrastructure setup
- Given "troubleshooting.md" - finds other troubleshooting guides, FAQ files, error documentation
- Given "config.yaml" - finds other configuration files, settings, environment setups
- Given "auth.py" - finds other authentication modules, security code, middleware

Returns file paths with similarity scores. Use get_content to read full files or get_chunk for specific sections.`,
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
        },
        workspace: {
          type: 'string',
          description: 'Optional workspace name to filter results. Only files within the workspace directories will be considered. Use server_info to see available workspaces.'
        }
      },
      required: ['file_path']
    }
  },
  {
    name: 'get_content',
    description: `Retrieve the full content of a file or specific chunks. Reads files directly from the filesystem.

When to use this tool:
- Get complete file content after finding files through search
- Read documentation, code files, or configuration files for analysis
- Extract specific sections of large files using chunk ranges
- Access any text-based file content

How it works:
- Reads files directly from filesystem (not from search index)
- Returns entire file by default
- Can return specific chunk ranges for indexed files
- Preserves original formatting and content

Examples:
- Get full file: file_path="/home/user/docs/api.md"
- Get specific chunks: file_path="/home/user/code/main.py", chunks="2-5"
- Get single chunk: file_path="/home/user/config.json", chunks="1"

Returns file content as text. Use this after search or similar_files to read actual content.`,
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
    description: `Retrieve content of a specific chunk from an indexed file. Gets exact text segments identified during search.

When to use this tool:
- Get specific relevant sections after performing a search
- Access only the most pertinent parts of large files
- Retrieve content from high-scoring chunks identified in search results
- Avoid reading entire files when only specific sections are needed

How it works:
- Files are split into overlapping text chunks during indexing
- Each chunk has a sequential ID ("0", "1", "2", etc.)
- Search results include chunk IDs for relevant sections
- Returns the exact content that was semantically matched

Examples:
- After search returns chunk "3" from "api-docs.md" with high score
- Get chunk content: file_path="/docs/api-docs.md", chunk_id="3"
- Returns the specific text segment that matched your query

Returns chunk content as text. Use this with chunk IDs from search results to get precise content sections.`,
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
    description: `Get information about server status and indexed content. Shows what directories and files are available for search.

When to use this tool:
- Check what content is already indexed before performing searches
- Verify system is working properly
- See indexing statistics and status
- Understand scope of available searchable content

How it works:
- Reports total indexed directories, files, and chunks
- Shows database size and last indexing time
- Lists all indexed directories with file counts
- Reports any errors or issues

Examples:
- Check before searching: "What content is indexed?"
- Verify after indexing: "Did the indexing complete successfully?"
- Monitor system: "How many files are searchable?"

Returns server version, indexing statistics, directory list, and any errors. Use this to understand what content is available for search and similar_files tools.`,
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
          const options = { 
            limit: (args.limit as number) || 10,
            workspace: args.workspace as string | undefined
          };
          const results = await searchContent(args.query, options);
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
          const results = await findSimilarFiles(
            args.file_path, 
            (args.limit as number) || 10,
            args.workspace as string | undefined
          );
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