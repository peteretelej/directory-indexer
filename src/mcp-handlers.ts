import { Config } from './config.js';
import { indexDirectories } from './indexing.js';
import { searchContent, findSimilarFiles, getFileContent, getChunkContent } from './search.js';
import { getIndexStatus } from './storage.js';
import { validateIndexPrerequisites, validateSearchPrerequisites } from './prerequisites.js';
import { CallToolResult } from '@modelcontextprotocol/sdk/types.js';

// Type-safe interfaces for MCP tool arguments
interface IndexToolArgs {
  directory_path: string;
}

interface SearchToolArgs {
  query: string;
  limit?: number;
  workspace?: string;
}

interface SimilarFilesToolArgs {
  file_path: string;
  limit?: number;
  workspace?: string;
}

interface GetContentToolArgs {
  file_path: string;
  chunks?: string;
}

interface GetChunkToolArgs {
  file_path: string;
  chunk_id: string;
}

// Type guard functions
function isIndexToolArgs(args: unknown): args is IndexToolArgs {
  return typeof args === 'object' && args !== null && 
         typeof (args as IndexToolArgs).directory_path === 'string';
}

function isSearchToolArgs(args: unknown): args is SearchToolArgs {
  return typeof args === 'object' && args !== null && 
         typeof (args as SearchToolArgs).query === 'string';
}

function isSimilarFilesToolArgs(args: unknown): args is SimilarFilesToolArgs {
  return typeof args === 'object' && args !== null && 
         typeof (args as SimilarFilesToolArgs).file_path === 'string';
}

function isGetContentToolArgs(args: unknown): args is GetContentToolArgs {
  return typeof args === 'object' && args !== null && 
         typeof (args as GetContentToolArgs).file_path === 'string';
}

function isGetChunkToolArgs(args: unknown): args is GetChunkToolArgs {
  return typeof args === 'object' && args !== null && 
         typeof (args as GetChunkToolArgs).file_path === 'string' &&
         typeof (args as GetChunkToolArgs).chunk_id === 'string';
}

export async function handleIndexTool(args: unknown, config: Config): Promise<CallToolResult> {
  if (!isIndexToolArgs(args)) {
    throw new Error('directory_path is required');
  }
  
  // Validate prerequisites before proceeding
  await validateIndexPrerequisites(config);
  
  const paths = args.directory_path.split(',').map((p: string) => p.trim());
  const result = await indexDirectories(paths, config);
  
  let responseText = `Indexed ${result.indexed} files, skipped ${result.skipped} files, cleaned up ${result.deleted} deleted files, ${result.failed} failed`;
  
  if (result.errors.length > 0) {
    responseText += `\nErrors: [\n`;
    result.errors.forEach(error => {
      responseText += `  '${error}'\n`;
    });
    responseText += `]`;
  }
  
  return {
    content: [
      {
        type: 'text',
        text: responseText
      }
    ]
  };
}

async function validateWorkspace(workspace?: string): Promise<{ workspace?: string; message?: string }> {
  if (!workspace) return { workspace };
  
  const config = (await import('./config.js')).loadConfig();
  const { getAvailableWorkspaces } = await import('./config.js');
  const availableWorkspaces = getAvailableWorkspaces(config);
  
  if (availableWorkspaces.includes(workspace)) {
    return { workspace };
  }
  
  // Invalid workspace - search all content with informative message
  const message = availableWorkspaces.length > 0
    ? `Note: Workspace '${workspace}' not found. Searching all content instead. Available workspaces: ${availableWorkspaces.join(', ')}. Use server_info tool to see workspace details.`
    : `Note: Workspace '${workspace}' not found and no workspaces are configured. Searching all indexed content.`;
  
  return { workspace: undefined, message };
}

export async function handleSearchTool(args: unknown): Promise<CallToolResult> {
  if (!isSearchToolArgs(args)) {
    throw new Error('query is required');
  }
  
  // Validate prerequisites before proceeding
  const config = (await import('./config.js')).loadConfig();
  await validateSearchPrerequisites(config);
  
  const { workspace, message } = await validateWorkspace(args.workspace);
  const results = await searchContent(args.query, { limit: args.limit || 10, workspace });
  
  const response = message 
    ? `${message}\n\n${JSON.stringify(results, null, 2)}`
    : JSON.stringify(results, null, 2);
  
  return {
    content: [{ type: 'text', text: response }]
  };
}

export async function handleSimilarFilesTool(args: unknown): Promise<CallToolResult> {
  if (!isSimilarFilesToolArgs(args)) {
    throw new Error('file_path is required');
  }
  
  // Validate prerequisites before proceeding
  const config = (await import('./config.js')).loadConfig();
  await validateSearchPrerequisites(config);
  
  const { workspace, message } = await validateWorkspace(args.workspace);
  const results = await findSimilarFiles(args.file_path, args.limit || 10, workspace);
  
  const response = message 
    ? `${message}\n\n${JSON.stringify(results, null, 2)}`
    : JSON.stringify(results, null, 2);
  
  return {
    content: [{ type: 'text', text: response }]
  };
}

export async function handleGetContentTool(args: unknown): Promise<CallToolResult> {
  if (!isGetContentToolArgs(args)) {
    throw new Error('file_path is required');
  }
  
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

export async function handleGetChunkTool(args: unknown): Promise<CallToolResult> {
  if (!isGetChunkToolArgs(args)) {
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

export async function handleServerInfoTool(version: string): Promise<CallToolResult> {
  const status = await getIndexStatus();
  
  return {
    content: [
      {
        type: 'text',
        text: JSON.stringify({
          name: 'directory-indexer',
          version: version,
          status: status
        }, null, 2)
      }
    ]
  };
}

export function formatErrorResponse(error: unknown): CallToolResult {
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