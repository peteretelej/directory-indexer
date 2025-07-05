import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { 
  CallToolRequestSchema, 
  ListToolsRequestSchema,
  Tool
} from '@modelcontextprotocol/sdk/types.js';
import { Config } from './config.js';
import { indexDirectories } from './indexing.js';
import { searchContent, findSimilarFiles, getFileContent } from './search.js';
import { getIndexStatus } from './storage.js';

const MCP_TOOLS: Tool[] = [
  {
    name: 'index',
    description: 'Index directories for semantic search',
    inputSchema: {
      type: 'object',
      properties: {
        directory_path: {
          type: 'string',
          description: 'Comma-separated list of directory paths to index'
        }
      },
      required: ['directory_path']
    }
  },
  {
    name: 'search',
    description: 'Search indexed content semantically',
    inputSchema: {
      type: 'object',
      properties: {
        query: {
          type: 'string',
          description: 'Search query'
        },
        limit: {
          type: 'number',
          description: 'Maximum number of results (default: 10)',
          default: 10
        }
      },
      required: ['query']
    }
  },
  {
    name: 'similar_files',
    description: 'Find files similar to a given file',
    inputSchema: {
      type: 'object',
      properties: {
        file_path: {
          type: 'string',
          description: 'Path to the file to find similar files for'
        },
        limit: {
          type: 'number',
          description: 'Maximum number of results (default: 10)',
          default: 10
        }
      },
      required: ['file_path']
    }
  },
  {
    name: 'get_content',
    description: 'Get file content',
    inputSchema: {
      type: 'object',
      properties: {
        file_path: {
          type: 'string',
          description: 'Path to the file to retrieve'
        },
        chunks: {
          type: 'string',
          description: 'Optional chunk range (e.g., "2-5")'
        }
      },
      required: ['file_path']
    }
  },
  {
    name: 'server_info',
    description: 'Get server information and status',
    inputSchema: {
      type: 'object',
      properties: {}
    }
  }
];

export async function startMcpServer(config: Config): Promise<void> {
  const server = new Server(
    {
      name: 'directory-indexer',
      version: '0.0.10'
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
          const paths = args.directory_path.split(',').map((p: string) => p.trim());
          const result = await indexDirectories(paths, config);
          return {
            content: [
              {
                type: 'text',
                text: `Indexed ${result.filesProcessed} files, ${result.chunksCreated} chunks, ${result.errors.length} errors`
              }
            ]
          };
        }

        case 'search': {
          const results = await searchContent(args.query, { limit: args.limit || 10 });
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
          const results = await findSimilarFiles(args.file_path, args.limit || 10);
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
          const content = await getFileContent(args.file_path, args.chunks);
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
                  version: '0.0.10',
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