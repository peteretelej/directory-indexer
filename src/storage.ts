import Database from 'better-sqlite3';
import { Config } from './config.js';
import { FileInfo, ChunkInfo } from './utils.js';
import { mkdirSync } from 'node:fs';
import { dirname } from 'path';

/**
 * Escape special characters in a string for use in a SQL LIKE pattern.
 * Escapes %, _, and \ using \ as the escape character.
 */
function escapeLike(value: string): string {
  return value.replace(/\\/g, '\\\\').replace(/%/g, '\\%').replace(/_/g, '\\_');
}

/**
 * Build a SQL WHERE clause that matches files within a directory,
 * handling both `/` and `\` as path separators so queries work
 * regardless of how paths were stored.
 *
 * Returns [clause, ...params] where clause uses `?` placeholders.
 */
function directoryLikeClause(column: string, dirPath: string): { clause: string; params: string[] } {
  const escaped = escapeLike(dirPath);
  return {
    clause: `(${column} = ? OR ${column} LIKE ? ESCAPE '\\' OR ${column} LIKE ? ESCAPE '\\')`,
    params: [dirPath, `${escaped}/%`, `${escaped}\\\\%`]
  };
}

// Registry of all open SQLiteStorage instances for graceful shutdown
const openStorageInstances = new Set<SQLiteStorage>();

// Shared storage handles memoized per SQLite database path
const sharedStorage = new Map<string, { sqlite: SQLiteStorage; qdrant: QdrantClient }>();

/**
 * Close all open SQLiteStorage instances. Called during graceful shutdown.
 */
export function closeAllStorage(): void {
  for (const instance of openStorageInstances) {
    try {
      instance.close();
    } catch {
      // Ignore errors during shutdown cleanup
    }
  }
  sharedStorage.clear();
}

export interface DirectoryRecord {
  id: number;
  path: string;
  status: 'pending' | 'indexing' | 'completed' | 'failed';
  indexedAt: Date;
}

export interface FileRecord {
  id: number;
  path: string;
  size: number;
  modifiedTime: Date;
  hash: string;
  parentDirs: string[];
  chunks: ChunkInfo[];
  errors?: string[];
}

export interface QdrantPoint {
  id: string | number;
  vector: number[];
  payload: {
    filePath: string;
    chunkId: string;
    fileHash: string;
    content: string;
    parentDirectories: string[];
  };
  score?: number;
}

export class StorageError extends Error {
  constructor(message: string, public override cause?: Error) {
    super(message);
    this.name = 'StorageError';
  }
}

export class QdrantClient {
  constructor(private config: Config) {}

  private qdrantFetch(path: string, init?: Parameters<typeof fetch>[1]): ReturnType<typeof fetch> {
    const headers: Record<string, string> = { 'Content-Type': 'application/json' };
    if (this.config.storage.qdrantApiKey) {
      headers['api-key'] = this.config.storage.qdrantApiKey;
    }
    return fetch(`${this.config.storage.qdrantEndpoint}${path}`, { ...init, headers });
  }

  async healthCheck(): Promise<boolean> {
    try {
      const response = await this.qdrantFetch('/healthz');
      return response.ok;
    } catch {
      return false;
    }
  }

  async createCollection(dimension: number): Promise<void> {
    const collectionName = this.config.storage.qdrantCollection;
    
    try {
      const checkResponse = await this.qdrantFetch(`/collections/${collectionName}`);
      if (checkResponse.ok) {
        return;
      }

      const createResponse = await this.qdrantFetch(`/collections/${collectionName}`, {
        method: 'PUT',
        body: JSON.stringify({
          vectors: {
            size: dimension,
            distance: 'Cosine'
          }
        })
      });

      if (!createResponse.ok) {
        throw new Error(`Failed to create collection: ${createResponse.statusText}`);
      }
    } catch (error) {
      throw new StorageError(`Failed to create Qdrant collection`, error as Error);
    }
  }

  async upsertPoints(points: QdrantPoint[]): Promise<void> {
    const collectionName = this.config.storage.qdrantCollection;
    
    try {
      const response = await this.qdrantFetch(`/collections/${collectionName}/points`, {
        method: 'PUT',
        body: JSON.stringify({ points })
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`Failed to upsert points: ${response.status} ${response.statusText} - ${errorText}`);
      }
    } catch (error) {
      throw new StorageError(`Failed to upsert points to Qdrant`, error as Error);
    }
  }

  async searchPoints(vector: number[], limit: number = 10, filter?: Record<string, unknown>): Promise<QdrantPoint[]> {
    const collectionName = this.config.storage.qdrantCollection;
    
    try {
      const searchBody: Record<string, unknown> = {
        vector,
        limit,
        with_payload: true
      };
      
      if (filter) {
        searchBody.filter = filter;
      }
      
      const response = await this.qdrantFetch(`/collections/${collectionName}/points/search`, {
        method: 'POST',
        body: JSON.stringify(searchBody)
      });

      if (response.status === 404) {
        return [];
      }

      if (!response.ok) {
        const errorBody = await response.text();
        throw new Error(`Failed to search points: ${response.statusText} - ${errorBody}`);
      }

      const data = await response.json();
      return data.result.map((item: { id: string | number; vector: number[]; payload: Record<string, unknown>; score: number }) => ({
        id: item.id,
        vector: item.vector,
        payload: item.payload,
        score: item.score
      }));
    } catch (error) {
      throw new StorageError(`Failed to search points in Qdrant`, error as Error);
    }
  }

  async deletePoints(ids: (string | number)[]): Promise<void> {
    const collectionName = this.config.storage.qdrantCollection;
    
    try {
      const response = await this.qdrantFetch(`/collections/${collectionName}/points/delete`, {
        method: 'POST',
        body: JSON.stringify({ points: ids })
      });

      if (response.status === 404) {
        return;
      }

      if (!response.ok) {
        throw new Error(`Failed to delete points: ${response.statusText}`);
      }
    } catch (error) {
      throw new StorageError(`Failed to delete points from Qdrant`, error as Error);
    }
  }

  async deletePointsByFilePath(filePath: string): Promise<void> {
    const collectionName = this.config.storage.qdrantCollection;
    
    try {
      const response = await this.qdrantFetch(`/collections/${collectionName}/points/delete`, {
        method: 'POST',
        body: JSON.stringify({
          filter: {
            must: [
              {
                key: 'filePath',
                match: { value: filePath }
              }
            ]
          }
        })
      });

      if (response.status === 404) {
        return;
      }

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`Failed to delete points by file path: ${response.status} ${response.statusText} - ${errorText}`);
      }
    } catch (error) {
      throw new StorageError(`Failed to delete points by file path from Qdrant`, error as Error);
    }
  }

  async countPoints(filter?: Record<string, unknown>): Promise<number> {
    const collectionName = this.config.storage.qdrantCollection;
    
    try {
      const countBody: Record<string, unknown> = { exact: true };
      if (filter) {
        countBody.filter = filter;
      }

      const response = await this.qdrantFetch(`/collections/${collectionName}/points/count`, {
        method: 'POST',
        body: JSON.stringify(countBody)
      });

      if (response.status === 404) {
        return 0;
      }

      if (!response.ok) {
        throw new Error(`Failed to count points: ${response.statusText}`);
      }

      const data = await response.json();
      return data.result.count;
    } catch (error) {
      throw new StorageError(`Failed to count points in Qdrant`, error as Error);
    }
  }

  async scrollPoints(filter?: Record<string, unknown>, limit: number = 1000): Promise<QdrantPoint[]> {
    const collectionName = this.config.storage.qdrantCollection;

    try {
      const points: QdrantPoint[] = [];
      let offset: string | number | null = null;

      do {
        const scrollBody: Record<string, unknown> = {
          limit,
          with_payload: true,
          with_vector: false
        };
        if (filter) {
          scrollBody.filter = filter;
        }
        if (offset !== null) {
          scrollBody.offset = offset;
        }

        const response = await this.qdrantFetch(`/collections/${collectionName}/points/scroll`, {
          method: 'POST',
          body: JSON.stringify(scrollBody)
        });

        if (response.status === 404) {
          return [];
        }

        if (!response.ok) {
          throw new Error(`Failed to scroll points: ${response.statusText}`);
        }

        const data = await response.json();
        points.push(...data.result.points.map((item: { id: string | number; payload: Record<string, unknown> }) => ({
          id: item.id,
          vector: [],
          payload: item.payload,
          score: 0
        })));
        offset = data.result.next_page_offset ?? null;
      } while (offset !== null);

      return points;
    } catch (error) {
      throw new StorageError(`Failed to scroll points in Qdrant`, error as Error);
    }
  }

  async getCollectionInfo(): Promise<{ vectors_count?: number; vectorSize?: number } | null> {
    const collectionName = this.config.storage.qdrantCollection;
    
    try {
      const response = await this.qdrantFetch(`/collections/${collectionName}`);
      
      if (!response.ok) {
        if (response.status === 404) {
          return null;
        }
        throw new Error(`Failed to get collection info: ${response.statusText}`);
      }

      const data = await response.json();
      const vectors = data.result?.config?.params?.vectors;
      const info: { vectors_count?: number; vectorSize?: number } = {
        vectors_count: data.result?.points_count || data.result?.vectors_count || 0
      };
      if (vectors && typeof vectors === 'object' && typeof vectors.size === 'number') {
        info.vectorSize = vectors.size;
      }
      return info;
    } catch (error) {
      throw new StorageError(`Failed to get collection info from Qdrant`, error as Error);
    }
  }

  async deleteCollection(): Promise<void> {
    const collectionName = this.config.storage.qdrantCollection;
    
    try {
      const response = await this.qdrantFetch(`/collections/${collectionName}`, {
        method: 'DELETE'
      });

      if (!response.ok && response.status !== 404) {
        throw new Error(`Failed to delete collection: ${response.statusText}`);
      }
    } catch (error) {
      throw new StorageError(`Failed to delete collection from Qdrant`, error as Error);
    }
  }
}

export class SQLiteStorage {
  public db: Database.Database;

  constructor(private config: Config) {
    this.db = this.initializeDatabase();
    openStorageInstances.add(this);
  }

  private initializeDatabase(): Database.Database {
    try {
      mkdirSync(dirname(this.config.storage.sqlitePath), { recursive: true });

      const db = new Database(this.config.storage.sqlitePath);
      
      db.exec(`
        CREATE TABLE IF NOT EXISTS directories (
          id INTEGER PRIMARY KEY,
          path TEXT UNIQUE NOT NULL,
          status TEXT DEFAULT 'pending',
          indexed_at INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS files (
          id INTEGER PRIMARY KEY,
          path TEXT UNIQUE NOT NULL,
          size INTEGER NOT NULL,
          modified_time INTEGER NOT NULL,
          hash TEXT NOT NULL,
          parent_dirs TEXT NOT NULL,
          chunks_json TEXT,
          errors_json TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_files_path ON files(path);
        CREATE INDEX IF NOT EXISTS idx_files_hash ON files(hash);
        CREATE INDEX IF NOT EXISTS idx_directories_path ON directories(path);

        CREATE TABLE IF NOT EXISTS meta (
          key TEXT PRIMARY KEY,
          value TEXT
        );
      `);

      db.pragma('journal_mode = WAL');
      db.pragma('busy_timeout = 5000');

      return db;
    } catch (error) {
      throw new StorageError(`Failed to initialize SQLite database`, error as Error);
    }
  }

  async getDirectory(path: string): Promise<DirectoryRecord | null> {
    try {
      const stmt = this.db.prepare('SELECT * FROM directories WHERE path = ?');
      const row = stmt.get(path) as { id: number; path: string; status: 'pending' | 'indexing' | 'completed' | 'failed'; indexed_at: number } | undefined;
      
      if (!row) return null;
      
      return {
        id: row.id,
        path: row.path,
        status: row.status,
        indexedAt: new Date(row.indexed_at)
      };
    } catch (error) {
      throw new StorageError(`Failed to get directory record`, error as Error);
    }
  }

  async getDirectoryByCaseInsensitive(path: string): Promise<DirectoryRecord | null> {
    try {
      const stmt = this.db.prepare('SELECT * FROM directories WHERE path = ? COLLATE NOCASE');
      const row = stmt.get(path) as { id: number; path: string; status: 'pending' | 'indexing' | 'completed' | 'failed'; indexed_at: number } | undefined;

      if (!row) return null;

      return {
        id: row.id,
        path: row.path,
        status: row.status,
        indexedAt: new Date(row.indexed_at)
      };
    } catch (error) {
      throw new StorageError(`Failed to get directory record`, error as Error);
    }
  }

  async upsertDirectory(path: string, status: DirectoryRecord['status']): Promise<void> {
    try {
      const stmt = this.db.prepare(`
        INSERT OR REPLACE INTO directories (path, status, indexed_at)
        VALUES (?, ?, ?)
      `);
      
      stmt.run(path, status, Date.now());
    } catch (error) {
      throw new StorageError(`Failed to upsert directory record`, error as Error);
    }
  }

  async getFile(path: string): Promise<FileRecord | null> {
    try {
      const stmt = this.db.prepare('SELECT * FROM files WHERE path = ?');
      const row = stmt.get(path) as { id: number; path: string; size: number; modified_time: number; hash: string; parent_dirs: string; chunks_json: string | null; errors_json: string | null } | undefined;
      
      if (!row) return null;
      
      return {
        id: row.id,
        path: row.path,
        size: row.size,
        modifiedTime: new Date(row.modified_time),
        hash: row.hash,
        parentDirs: JSON.parse(row.parent_dirs),
        chunks: row.chunks_json ? JSON.parse(row.chunks_json) : [],
        errors: row.errors_json ? JSON.parse(row.errors_json) : undefined
      };
    } catch (error) {
      throw new StorageError(`Failed to get file record`, error as Error);
    }
  }

  async upsertFile(fileInfo: FileInfo, chunks: ChunkInfo[] = [], errors: string[] = []): Promise<void> {
    try {
      const stmt = this.db.prepare(`
        INSERT OR REPLACE INTO files (path, size, modified_time, hash, parent_dirs, chunks_json, errors_json)
        VALUES (?, ?, ?, ?, ?, ?, ?)
      `);
      
      stmt.run(
        fileInfo.path,
        fileInfo.size,
        fileInfo.modifiedTime.getTime(),
        fileInfo.hash,
        JSON.stringify(fileInfo.parentDirs),
        chunks.length > 0 ? JSON.stringify(chunks) : null,
        errors.length > 0 ? JSON.stringify(errors) : null
      );
    } catch (error) {
      throw new StorageError(`Failed to upsert file record`, error as Error);
    }
  }

  async deleteFile(path: string): Promise<void> {
    try {
      const stmt = this.db.prepare('DELETE FROM files WHERE path = ?');
      stmt.run(path);
    } catch (error) {
      throw new StorageError(`Failed to delete file record`, error as Error);
    }
  }

  getDirectories(): string[] {
    const rows = this.db.prepare('SELECT path FROM directories').all() as { path: string }[];
    return rows.map(row => row.path);
  }

  async getFilesByDirectory(directoryPath: string): Promise<FileRecord[]> {
    try {
      const { clause, params } = directoryLikeClause('path', directoryPath);
      const stmt = this.db.prepare(`SELECT * FROM files WHERE ${clause}`);
      const rows = stmt.all(...params) as { id: number; path: string; size: number; modified_time: number; hash: string; parent_dirs: string; chunks_json: string | null; errors_json: string | null }[];

      return rows.map(row => ({
        id: row.id,
        path: row.path,
        size: row.size,
        modifiedTime: new Date(row.modified_time),
        hash: row.hash,
        parentDirs: JSON.parse(row.parent_dirs),
        chunks: row.chunks_json ? JSON.parse(row.chunks_json) : [],
        errors: row.errors_json ? JSON.parse(row.errors_json) : undefined
      }));
    } catch (error) {
      throw new StorageError(`Failed to get files by directory`, error as Error);
    }
  }

  deleteDirectory(path: string): void {
    this.db.prepare('DELETE FROM directories WHERE path = ?').run(path);
  }

  deleteFilesByDirectory(directoryPath: string): number {
    const { clause, params } = directoryLikeClause('path', directoryPath);
    const result = this.db.prepare(`DELETE FROM files WHERE ${clause}`).run(...params);
    return result.changes;
  }

  getMeta(key: string): string | null {
    const row = this.db.prepare('SELECT value FROM meta WHERE key = ?').get(key) as { value: string } | undefined;
    return row?.value ?? null;
  }

  setMeta(key: string, value: string): void {
    this.db.prepare('INSERT OR REPLACE INTO meta (key, value) VALUES (?, ?)').run(key, value);
  }

  close(): void {
    openStorageInstances.delete(this);
    this.db.close();
  }
}

export async function initializeStorage(config: Config): Promise<{ sqlite: SQLiteStorage; qdrant: QdrantClient }> {
  const dbPath = config.storage.sqlitePath;
  const cached = sharedStorage.get(dbPath);
  if (cached) {
    return cached;
  }

  const sqlite = new SQLiteStorage(config);
  const qdrant = new QdrantClient(config);
  const storage = { sqlite, qdrant };
  sharedStorage.set(dbPath, storage);

  return storage;
}

/**
 * Close and evict the shared storage handle for one database path so the
 * next initializeStorage call for it opens a fresh connection. Reset and
 * cache-eviction flows use this; callers must never close the shared
 * handle themselves.
 */
export function closeSharedStorageForPath(dbPath: string): void {
  const shared = sharedStorage.get(dbPath);
  if (!shared) {
    return;
  }
  sharedStorage.delete(dbPath);
  shared.sqlite.close();
}

export async function ensureCollectionDimensions(
  sqlite: SQLiteStorage,
  qdrant: QdrantClient,
  config: Config,
  dimension: number
): Promise<void> {
  const info = await qdrant.getCollectionInfo();

  if (!info) {
    await qdrant.createCollection(dimension);
  } else if (info.vectorSize !== undefined && info.vectorSize !== dimension) {
    const provider = sqlite.getMeta('embedding_provider');
    const recordedDims = sqlite.getMeta('embedding_dims');
    const recorded = provider && recordedDims
      ? ` (meta table records provider '${provider}' with ${recordedDims} dimensions)`
      : '';
    throw new StorageError(
      `Qdrant collection '${config.storage.qdrantCollection}' was created with ${info.vectorSize}-dimensional vectors ` +
      `but the active embedding provider produces ${dimension}-dimensional vectors${recorded}. ` +
      `Run \`directory-indexer reset\` to delete the existing collection, then re-index.`
    );
  }

  sqlite.setMeta('embedding_provider', config.embedding.provider);
  sqlite.setMeta('embedding_dims', String(dimension));
}

export async function initDatabase(dbPath: string): Promise<Database.Database> {
  return new Database(dbPath);
}

export interface ResetStats {
  sqliteExists: boolean;
  sqliteSize?: string;
  qdrantCollectionExists: boolean;
  qdrantVectorCount?: number;
}

export interface ResetResult {
  sqliteDeleted: boolean;
  qdrantDeleted: boolean;
  warnings: string[];
}

export async function getResetPreview(config: Config): Promise<ResetStats> {
  const stats: ResetStats = {
    sqliteExists: false,
    qdrantCollectionExists: false
  };

  if (await import('fs').then(fs => fs.existsSync(config.storage.sqlitePath))) {
    stats.sqliteExists = true;
    try {
      const fileStats = await import('fs').then(fs => fs.statSync(config.storage.sqlitePath));
      const sizeInMB = (fileStats.size / (1024 * 1024)).toFixed(1);
      stats.sqliteSize = `${sizeInMB} MB`;
    } catch {
      stats.sqliteSize = 'unknown size';
    }
  }

  try {
    const qdrant = new QdrantClient(config);
    const isHealthy = await qdrant.healthCheck();
    
    if (isHealthy) {
      const collectionInfo = await qdrant.getCollectionInfo();
      if (collectionInfo) {
        stats.qdrantCollectionExists = true;
        stats.qdrantVectorCount = collectionInfo.vectors_count || 0;
      }
    }
  } catch {
    // Qdrant unavailable
  }

  return stats;
}

export async function clearDatabase(config: Config): Promise<boolean> {
  try {
    closeSharedStorageForPath(config.storage.sqlitePath);
    if (!await import('fs').then(fs => fs.existsSync(config.storage.sqlitePath))) {
      return true; // Already clean
    }

    await import('fs/promises').then(fs => fs.unlink(config.storage.sqlitePath));
    return true;
  } catch (error) {
    throw new StorageError(`Failed to delete SQLite database: ${error instanceof Error ? error.message : 'Unknown error'}`, error as Error);
  }
}

export async function clearVectorCollection(config: Config): Promise<boolean> {
  try {
    const qdrant = new QdrantClient(config);
    const isHealthy = await qdrant.healthCheck();
    
    if (!isHealthy) {
      throw new StorageError(`Qdrant unavailable at ${config.storage.qdrantEndpoint}`);
    }
    
    const collectionInfo = await qdrant.getCollectionInfo();
    if (collectionInfo) {
      await qdrant.deleteCollection();
    }
    
    return true;
  } catch (error) {
    throw new StorageError(`Failed to reset Qdrant collection: ${error instanceof Error ? error.message : 'Unknown error'}`, error as Error);
  }
}

export interface DirectoryStatus {
  path: string;
  status: string;
  filesCount: number;
  chunksCount: number;
  lastIndexed: string | null;
  errors: string[];
}

export interface WorkspaceStatus {
  name: string;
  paths: string[];
  isValid: boolean;
  filesCount: number;
  chunksCount: number;
  health: {
    status: 'healthy' | 'warning' | 'error';
    issues: string[];
    recommendations: string[];
  };
}

export interface IndexStatus {
  directoriesIndexed: number;
  filesIndexed: number;
  chunksIndexed: number;
  databaseSize: string;
  lastIndexed: string | null;
  errors: string[];
  directories: DirectoryStatus[];
  workspaces: WorkspaceStatus[];
  workspaceHealth: {
    healthy: number;
    warnings: number;
    errors: number;
    criticalIssues: string[];
    recommendations: string[];
  };
  qdrantConsistency: {
    isConsistent: boolean;
    issues: string[];
  };
}

async function calculateWorkspaceStatistics(sqlite: SQLiteStorage, config: Config): Promise<WorkspaceStatus[]> {
  const { getAvailableWorkspaces, getWorkspacePaths } = await import('./config.js');
  const workspaces: WorkspaceStatus[] = [];
  
  // Get all indexed directories for comparison
  const directoriesStmt = sqlite.db.prepare('SELECT path, status FROM directories');
  const indexedDirectories = directoriesStmt.all() as { path: string; status: string }[];
  const indexedDirectoryPaths = indexedDirectories.map(d => d.path);
  
  // Get Qdrant client for efficient workspace filtering
  const qdrant = new QdrantClient(config);
  
  for (const workspaceName of getAvailableWorkspaces(config)) {
    const workspacePaths = getWorkspacePaths(config, workspaceName);
    const workspaceConfig = config.workspaces[workspaceName];
    
    let filesCount = 0;
    let chunksCount = 0;
    
    if (workspacePaths.length > 0) {
      try {
        // Build workspace filter using same logic as search
        const workspaceFilter = {
          must: [
            {
              key: "parentDirectories",
              match: { any: workspacePaths }
            }
          ]
        };
        
        // Get chunk count efficiently using Qdrant
        chunksCount = await qdrant.countPoints(workspaceFilter);
        
        // Get unique file paths using Qdrant scroll
        const points = await qdrant.scrollPoints(workspaceFilter, 10000);
        const uniqueFilePaths = new Set(points.map(p => p.payload.filePath));
        filesCount = uniqueFilePaths.size;
        
      } catch {
        // Fallback to 0 if Qdrant is unavailable
        filesCount = 0;
        chunksCount = 0;
      }
    }
    
    // Check workspace health
    const health = analyzeWorkspaceHealth(workspacePaths, workspaceConfig.isValid, filesCount, indexedDirectoryPaths);
    
    workspaces.push({
      name: workspaceName,
      paths: workspacePaths,
      isValid: workspaceConfig.isValid,
      filesCount,
      chunksCount,
      health
    });
  }
  
  return workspaces;
}

function analyzeWorkspaceHealth(
  workspacePaths: string[], 
  isValid: boolean, 
  filesCount: number, 
  indexedDirectoryPaths: string[]
): { status: 'healthy' | 'warning' | 'error'; issues: string[]; recommendations: string[] } {
  const issues: string[] = [];
  const recommendations: string[] = [];
  
  // Check if workspace paths exist and are valid
  if (!isValid) {
    issues.push('One or more workspace directories do not exist on the filesystem');
    recommendations.push('Verify workspace directory paths exist and are accessible');
  }
  
  // Check if workspace paths are indexed
  const unindexedPaths: string[] = [];
  const partiallyIndexedPaths: string[] = [];
  
  for (const workspacePath of workspacePaths) {
    // Check if this exact path is indexed
    const exactMatch = indexedDirectoryPaths.includes(workspacePath);
    
    if (exactMatch) {
      // Perfect match - workspace directory is directly indexed
      continue;
    }
    
    // Check if workspace path is covered by a parent directory that is indexed
    const isChildOfIndexed = indexedDirectoryPaths.some(indexedPath => {
      // Workspace path must start with indexed path and be a subdirectory
      return workspacePath.startsWith(indexedPath + '/') || workspacePath.startsWith(indexedPath + '\\');
    });
    
    if (isChildOfIndexed) {
      partiallyIndexedPaths.push(workspacePath);
    } else {
      unindexedPaths.push(workspacePath);
    }
  }
  
  // Report unindexed paths
  if (unindexedPaths.length > 0) {
    issues.push(`Workspace directories not indexed: ${unindexedPaths.join(', ')}`);
    recommendations.push(`Run indexing on these directories: ${unindexedPaths.join(', ')}`);
  }
  
  // Report partially indexed paths (where parent directory is indexed)
  if (partiallyIndexedPaths.length > 0) {
    issues.push(`Workspace directories indexed as part of parent directory: ${partiallyIndexedPaths.join(', ')}`);
    recommendations.push('Consider indexing workspace directories directly for better organization');
  }
  
  // Check if workspace is empty (no files found)
  if (isValid && filesCount === 0 && unindexedPaths.length === 0) {
    issues.push('Workspace contains no indexed files');
    recommendations.push('Verify the workspace directory contains files and re-index if necessary');
  }
  
  // Determine overall health status
  let status: 'healthy' | 'warning' | 'error';
  if (!isValid || unindexedPaths.length > 0) {
    status = 'error';
  } else if (issues.length > 0) {
    status = 'warning';
  } else {
    status = 'healthy';
  }
  
  return { status, issues, recommendations };
}

function calculateWorkspaceHealthSummary(workspaces: WorkspaceStatus[]): {
  healthy: number;
  warnings: number; 
  errors: number;
  criticalIssues: string[];
  recommendations: string[];
} {
  let healthy = 0;
  let warnings = 0;
  let errors = 0;
  const criticalIssues: string[] = [];
  const recommendations: string[] = [];
  
  for (const workspace of workspaces) {
    switch (workspace.health.status) {
      case 'healthy':
        healthy++;
        break;
      case 'warning':
        warnings++;
        break;
      case 'error':
        errors++;
        criticalIssues.push(`${workspace.name}: ${workspace.health.issues.join(', ')}`);
        break;
    }
    
    // Collect unique recommendations
    for (const rec of workspace.health.recommendations) {
      if (!recommendations.includes(rec)) {
        recommendations.push(rec);
      }
    }
  }
  
  return {
    healthy,
    warnings,
    errors,
    criticalIssues,
    recommendations
  };
}

async function checkQdrantConsistency(sqlite: SQLiteStorage, config: Config): Promise<{ isConsistent: boolean; issues: string[] }> {
  const issues: string[] = [];
  
  try {
    const qdrant = new QdrantClient(config);
    const isHealthy = await qdrant.healthCheck();
    
    if (!isHealthy) {
      issues.push('Qdrant vector database is not running or accessible');
      return { isConsistent: false, issues };
    }
    
    const filesWithChunksStmt = sqlite.db.prepare('SELECT COUNT(*) as count FROM files WHERE chunks_json IS NOT NULL');
    const filesWithChunks = filesWithChunksStmt.get() as { count: number };
    
    const totalChunksStmt = sqlite.db.prepare('SELECT SUM(json_array_length(chunks_json)) as count FROM files WHERE chunks_json IS NOT NULL');
    const totalChunks = totalChunksStmt.get() as { count: number | null };
    
    if (filesWithChunks.count > 0 && (totalChunks.count || 0) === 0) {
      issues.push('Files exist but no chunks found - possible data corruption');
    }
    
    const collectionName = config.storage.qdrantCollection;
    try {
      const collectionInfo = await qdrant.getCollectionInfo();
      if (!collectionInfo) {
        issues.push(`Vector collection '${collectionName}' not found (normal during first-time setup)`);
        return { isConsistent: false, issues };
      }
      
      const qdrantPointCount = collectionInfo.vectors_count || 0;
      const sqliteChunkCount = totalChunks.count || 0;
      
      if (Math.abs(qdrantPointCount - sqliteChunkCount) > 0) {
        if (qdrantPointCount > sqliteChunkCount) {
          issues.push(`Extra vectors in database: ${qdrantPointCount} vectors vs ${sqliteChunkCount} indexed chunks (normal during cleanup)`);
        } else {
          issues.push(`Missing vectors: ${sqliteChunkCount} indexed chunks vs ${qdrantPointCount} vectors (normal during indexing)`);
        }
      }
    } catch (error) {
      issues.push(`Cannot verify vector database status: ${error}`);
    }
    
  } catch (error) {
    issues.push(`Database status check failed: ${error}`);
  }
  
  return {
    isConsistent: issues.length === 0,
    issues
  };
}

export async function getIndexStatus(): Promise<IndexStatus> {
  const config = await import('./config.js').then(m => m.loadConfig());
  const sqlite = new SQLiteStorage(config);
  
  try {
    const directoriesStmt = sqlite.db.prepare('SELECT COUNT(*) as count FROM directories WHERE status = ?');
    const directoriesCount = directoriesStmt.get('completed') as { count: number };
    
    const filesStmt = sqlite.db.prepare('SELECT COUNT(*) as count FROM files');
    const filesCount = filesStmt.get() as { count: number };
    
    const chunksStmt = sqlite.db.prepare('SELECT SUM(json_array_length(chunks_json)) as count FROM files WHERE chunks_json IS NOT NULL');
    const chunksCount = chunksStmt.get() as { count: number | null };
    
    const lastIndexedStmt = sqlite.db.prepare("SELECT MAX(indexed_at) as last_indexed FROM directories WHERE indexed_at > 0 AND status = 'completed'");
    const lastIndexedResult = lastIndexedStmt.get() as { last_indexed: number | null };
    
    const errorsStmt = sqlite.db.prepare('SELECT errors_json FROM files WHERE errors_json IS NOT NULL');
    const errorRows = errorsStmt.all() as { errors_json: string }[];
    
    const allErrors: string[] = [];
    errorRows.forEach(row => {
      try {
        const errors = JSON.parse(row.errors_json);
        allErrors.push(...errors);
      } catch {
        allErrors.push('Failed to parse error JSON');
      }
    });
    
    const directoriesDetailStmt = sqlite.db.prepare('SELECT path, status, indexed_at FROM directories ORDER BY indexed_at DESC');
    const directoryDetails = directoriesDetailStmt.all() as { path: string; status: 'pending' | 'indexing' | 'completed' | 'failed'; indexed_at: number }[];

    const directories: DirectoryStatus[] = directoryDetails.map(row => {
      const { clause, params } = directoryLikeClause('path', row.path);

      const filesByDirStmt = sqlite.db.prepare(`SELECT COUNT(*) as count FROM files WHERE ${clause}`);
      const filesCount = filesByDirStmt.get(...params) as { count: number };

      const chunksByDirStmt = sqlite.db.prepare(`SELECT COALESCE(SUM(json_array_length(chunks_json)), 0) as count FROM files WHERE ${clause} AND chunks_json IS NOT NULL`);
      const chunksCount = chunksByDirStmt.get(...params) as { count: number };

      const errorsByDirStmt = sqlite.db.prepare(`
        SELECT errors_json FROM files
        WHERE ${clause} AND errors_json IS NOT NULL
      `);
      const dirErrors = errorsByDirStmt.all(...params) as { errors_json: string }[];
      
      const dirErrorsList: string[] = [];
      dirErrors.forEach(errorRow => {
        try {
          const errors = JSON.parse(errorRow.errors_json);
          dirErrorsList.push(...errors);
        } catch {
          dirErrorsList.push('Failed to parse error JSON');
        }
      });
      
      return {
        path: row.path,
        status: row.status,
        filesCount: filesCount.count,
        chunksCount: chunksCount.count,
        lastIndexed: row.indexed_at && row.indexed_at > 0 ? new Date(row.indexed_at).toISOString() : null,
        errors: dirErrorsList
      };
    });
    
    const qdrantConsistency = await checkQdrantConsistency(sqlite, config);
    const workspaces = await calculateWorkspaceStatistics(sqlite, config);
    
    // Calculate workspace health summary
    const workspaceHealth = calculateWorkspaceHealthSummary(workspaces);
    
    const fs = await import('fs');
    let databaseSize = '0 KB';
    try {
      const stats = fs.statSync(config.storage.sqlitePath);
      const sizeInBytes = stats.size;
      if (sizeInBytes > 1024 * 1024) {
        databaseSize = `${(sizeInBytes / (1024 * 1024)).toFixed(2)} MB`;
      } else if (sizeInBytes > 1024) {
        databaseSize = `${(sizeInBytes / 1024).toFixed(2)} KB`;
      } else {
        databaseSize = `${sizeInBytes} bytes`;
      }
    } catch {
      databaseSize = 'Unknown';
    }
    
    return {
      directoriesIndexed: directoriesCount.count,
      filesIndexed: filesCount.count,
      chunksIndexed: chunksCount.count || 0,
      databaseSize,
      lastIndexed: lastIndexedResult.last_indexed ? new Date(lastIndexedResult.last_indexed).toISOString() : null,
      errors: allErrors,
      directories,
      workspaces,
      workspaceHealth,
      qdrantConsistency
    };
  } finally {
    sqlite.close();
  }
}