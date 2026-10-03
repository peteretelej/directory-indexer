import { Config } from './config.js';
import { indexDirectories } from './indexing.js';
import { searchContent, findSimilarFiles, getFileContent, getChunkContent } from './search.js';
import { getIndexStatus, SQLiteStorage, initializeStorage } from './storage.js';
import { validateIndexPrerequisites, validateSearchPrerequisites } from './prerequisites.js';
import { validatePathWithinIndexedDirs, resolveIndexedDirectories } from './path-validation.js';
import { normalizePath } from './utils.js';
import { log } from './logger.js';
import { CallToolResult } from '@modelcontextprotocol/sdk/types.js';
import type { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { resolve, sep } from 'path';
import { realpathSync } from 'fs';

// MCP server reference for sending client-visible log notifications
let mcpServer: Server | null = null;

/**
 * Set the MCP server reference for logging notifications.
 * Called from startMcpServer() after server creation.
 */
export function setMcpServer(server: Server): void {
  mcpServer = server;
}

function notifyClient(level: 'info' | 'error', data: Record<string, unknown>): void {
  mcpServer?.sendLoggingMessage({ level, data })?.catch(() => {});
}

/**
 * Normalize a directory path for use as a mutex key.
 * Resolves symlinks, makes absolute, and strips trailing separators
 * so that '/repo', '/repo/', and symlinks all map to the same key.
 */
function normalizeMutexKey(dirPath: string): string {
  let normalized: string;
  try {
    normalized = realpathSync(resolve(dirPath));
  } catch {
    // Directory may not exist yet; fall back to resolve only
    normalized = resolve(dirPath);
  }
  // Strip trailing separator (but keep root '/' or 'C:\')
  while (normalized.length > 1 && normalized.endsWith(sep)) {
    normalized = normalized.slice(0, -sep.length);
  }
  return normalized;
}

// Workspace-level indexing mutex: keyed by normalized directory path
const indexingMutex = new Map<string, Promise<void>>();

/**
 * Serialize an operation against prior holders of the given mutex keys.
 * The registered ticket resolves only when this operation's `fn` finishes,
 * so later arrivals chain behind the whole operation instead of acquiring
 * at check time; `fn`'s rejection propagates to its own caller only, and
 * release runs unconditionally in the finally.
 */
async function withDirectoryMutex<T>(keys: string[], fn: () => Promise<T>): Promise<T> {
  const holders = keys.map(key => indexingMutex.get(key));
  const prior = Promise.all(holders.map(holder => holder ?? Promise.resolve()));
  let release!: () => void;
  const done = new Promise<void>(resolve => { release = resolve; });
  const ticket = prior.then(() => done);
  for (const key of keys) indexingMutex.set(key, ticket);
  if (holders.some(Boolean)) {
    log('info', 'Waiting for ongoing indexing', { directories: keys });
  }
  await prior;
  try {
    return await fn();
  } finally {
    release();
    for (const key of keys) {
      if (indexingMutex.get(key) === ticket) indexingMutex.delete(key);
    }
  }
}

// Cached set of resolved indexed directory paths for path validation
let indexedDirsCache: Set<string> = new Set();
let indexedDirsCacheInitialized = false;

/**
 * Refresh the indexed directories cache from storage.
 * Exported for testability.
 */
export function refreshIndexedDirsCache(storage: SQLiteStorage): void {
  indexedDirsCache = resolveIndexedDirectories(storage);
  indexedDirsCacheInitialized = true;
}

/**
 * Ensure the cache is populated, lazily initializing from SQLite only (no Qdrant).
 * Uses a boolean flag so an empty result doesn't re-trigger initialization.
 */
async function ensureIndexedDirsCache(config: Config): Promise<void> {
  if (!indexedDirsCacheInitialized) {
    const { sqlite } = await initializeStorage(config);
    refreshIndexedDirsCache(sqlite);
  }
}

// Type-safe interfaces for MCP tool arguments
interface IndexToolArgs {
  directory_paths: string[];
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

interface DeleteIndexToolArgs {
  directory_path: string;
}

// Type guard functions
function isIndexToolArgs(args: unknown): args is IndexToolArgs {
  return typeof args === 'object' && args !== null &&
         Array.isArray((args as IndexToolArgs).directory_paths);
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

function isDeleteIndexToolArgs(args: unknown): args is DeleteIndexToolArgs {
  return typeof args === 'object' && args !== null &&
         typeof (args as DeleteIndexToolArgs).directory_path === 'string';
}

export async function handleIndexTool(args: unknown, config: Config): Promise<CallToolResult> {
  if (!isIndexToolArgs(args)) {
    throw new Error('directory_paths is required and must be an array');
  }

  const paths = args.directory_paths.map((p: string) => p.trim());
  const mutexKeys = paths.map(normalizeMutexKey);

  return withDirectoryMutex(mutexKeys, async () => {
    // Validate prerequisites before proceeding
    await validateIndexPrerequisites(config);

    log('info', 'Index start', { directories: paths });
    notifyClient('info', { event: 'index_start', directories: paths });

    try {
      const result = await indexDirectories(paths, config);

      // Refresh the indexed directories cache after successful indexing
      const { sqlite } = await initializeStorage(config);
      refreshIndexedDirsCache(sqlite);

      log('info', 'Index complete', { result });
      notifyClient('info', { event: 'index_complete', result });

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
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      log('error', 'Index error', { error: errorMessage, directories: paths });
      notifyClient('error', { event: 'index_error', error: errorMessage });
      throw new Error(
        `Indexing failed for ${paths.join(', ')}. Verify the directory exists and is readable. Use 'server_info' to check current status.`
      );
    }
  });
}

async function validateWorkspace(workspace?: string): Promise<string | undefined> {
  if (!workspace) return workspace;

  const config = (await import('./config.js')).loadConfig();
  const { getAvailableWorkspaces } = await import('./config.js');
  const availableWorkspaces = getAvailableWorkspaces(config);

  if (availableWorkspaces.includes(workspace)) {
    return workspace;
  }

  throw new Error(
    availableWorkspaces.length > 0
      ? `Workspace '${workspace}' not found. Available workspaces: ${availableWorkspaces.join(', ')}`
      : `Workspace '${workspace}' not found and no workspaces are configured.`
  );
}

export async function handleSearchTool(args: unknown): Promise<CallToolResult> {
  if (!isSearchToolArgs(args)) {
    throw new Error('query is required');
  }

  // Validate prerequisites before proceeding
  const config = (await import('./config.js')).loadConfig();
  await validateSearchPrerequisites(config);

  const workspace = await validateWorkspace(args.workspace);
  const results = await searchContent(args.query, { limit: args.limit || 10, workspace });

  return {
    content: [{ type: 'text', text: JSON.stringify(results, null, 2) }]
  };
}

export async function handleSimilarFilesTool(args: unknown): Promise<CallToolResult> {
  if (!isSimilarFilesToolArgs(args)) {
    throw new Error('file_path is required');
  }

  // Validate prerequisites before proceeding
  const config = (await import('./config.js')).loadConfig();
  await validateSearchPrerequisites(config);

  await ensureIndexedDirsCache(config);
  validatePathWithinIndexedDirs(args.file_path, indexedDirsCache);

  const workspace = await validateWorkspace(args.workspace);
  const results = await findSimilarFiles(args.file_path, args.limit || 10, workspace);

  return {
    content: [{ type: 'text', text: JSON.stringify(results, null, 2) }]
  };
}

export async function handleGetContentTool(args: unknown, config?: Config): Promise<CallToolResult> {
  if (!isGetContentToolArgs(args)) {
    throw new Error('file_path is required');
  }

  // Lazily populate cache and validate path
  const resolvedConfig = config || (await import('./config.js')).loadConfig();
  await ensureIndexedDirsCache(resolvedConfig);
  validatePathWithinIndexedDirs(args.file_path, indexedDirsCache);

  try {
    const content = await getFileContent(args.file_path, args.chunks);

    return {
      content: [
        {
          type: 'text',
          text: content
        }
      ]
    };
  } catch (error) {
    const msg = error instanceof Error ? error.message : String(error);
    if (msg.includes('ENOENT') || msg.toLowerCase().includes('not found') || msg.toLowerCase().includes('no such file')) {
      throw new Error(
        `File not found: ${args.file_path}. The file may have been moved or deleted. Use 'search' to find similar content.`
      );
    }
    throw error;
  }
}

export async function handleGetChunkTool(args: unknown, config?: Config): Promise<CallToolResult> {
  if (!isGetChunkToolArgs(args)) {
    throw new Error('file_path and chunk_id are required');
  }

  // Lazily populate cache and validate path
  const resolvedConfig = config || (await import('./config.js')).loadConfig();
  await ensureIndexedDirsCache(resolvedConfig);
  validatePathWithinIndexedDirs(args.file_path, indexedDirsCache);

  try {
    const content = await getChunkContent(args.file_path, args.chunk_id);

    return {
      content: [
        {
          type: 'text',
          text: content
        }
      ]
    };
  } catch (error) {
    const msg = error instanceof Error ? error.message : String(error);
    if (msg.includes('ENOENT') || msg.toLowerCase().includes('not found') || msg.toLowerCase().includes('no such file')) {
      throw new Error(
        `File not found: ${args.file_path}. The file may have been moved or deleted. Use 'search' to find similar content.`
      );
    }
    throw error;
  }
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

export async function handleDeleteIndexTool(args: unknown, config: Config): Promise<CallToolResult> {
  if (!isDeleteIndexToolArgs(args)) {
    throw new Error('directory_path is required');
  }

  const trimmedPath = args.directory_path.trim();
  const dirPath = normalizePath(trimmedPath);
  const mutexKey = normalizeMutexKey(trimmedPath);

  return withDirectoryMutex([mutexKey], async () => {
    const { sqlite, qdrant } = await initializeStorage(config);

    // Check if the directory is actually indexed
    let directory = await sqlite.getDirectory(dirPath);
    if (!directory && process.platform === 'win32') {
      directory = await sqlite.getDirectoryByCaseInsensitive(dirPath);
    }
    if (!directory) {
      throw new Error(
        `Directory '${dirPath}' is not indexed. Use 'server_info' to see indexed directories.`
      );
    }

    const storedPath = directory.path;

    // Get files for this directory to clean up Qdrant points
    const files = await sqlite.getFilesByDirectory(storedPath);
    const vectorErrors: string[] = [];
    for (const file of files) {
      try {
        await qdrant.deletePointsByFilePath(file.path);
      } catch (error) {
        const msg = error instanceof Error ? error.message : String(error);
        log('warning', 'Failed to delete Qdrant points for file', {
          path: file.path,
          error: msg
        });
        vectorErrors.push(`${file.path}: ${msg}`);
      }
    }

    // If any vector deletions failed, abort to prevent orphaned vectors
    if (vectorErrors.length > 0) {
      throw new Error(
        `Aborting delete: failed to remove vector embeddings for ${vectorErrors.length} file(s). ` +
        `SQLite rows were NOT deleted to avoid orphaned vectors.\n` +
        vectorErrors.join('\n')
      );
    }

    // Delete file records and directory record from SQLite
    const deletedFiles = sqlite.deleteFilesByDirectory(storedPath);
    sqlite.deleteDirectory(storedPath);

    // Refresh the indexed directories cache
    refreshIndexedDirsCache(sqlite);

    const chunksCount = files.reduce((sum, f) => sum + (f.chunks?.length || 0), 0);

    log('info', 'Index deleted', { directory: storedPath, files: deletedFiles, chunks: chunksCount });
    notifyClient('info', { event: 'index_deleted', directory: storedPath });

    return {
      content: [
        {
          type: 'text',
          text: `Deleted index for ${storedPath}: removed ${deletedFiles} files and ${chunksCount} chunks`
        }
      ]
    };
  });
}

export function formatErrorResponse(error: unknown): CallToolResult {
  const errorMessage = error instanceof Error ? error.message : 'Unknown error';
  log('error', 'Tool error', { error: errorMessage });
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