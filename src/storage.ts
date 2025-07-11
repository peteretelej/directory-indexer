import Database from 'better-sqlite3';
import { Config } from './config.js';
import { FileInfo, ChunkInfo, ensureDirectory } from './utils.js';
import { dirname } from 'path';

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

  async healthCheck(): Promise<boolean> {
    try {
      const response = await fetch(`${this.config.storage.qdrantEndpoint}/healthz`);
      return response.ok;
    } catch {
      return false;
    }
  }

  async createCollection(): Promise<void> {
    const collectionName = this.config.storage.qdrantCollection;
    
    try {
      const checkResponse = await fetch(`${this.config.storage.qdrantEndpoint}/collections/${collectionName}`);
      if (checkResponse.ok) {
        return;
      }

      const createResponse = await fetch(`${this.config.storage.qdrantEndpoint}/collections/${collectionName}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          vectors: {
            size: 768,
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
      const response = await fetch(`${this.config.storage.qdrantEndpoint}/collections/${collectionName}/points`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
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
      
      const response = await fetch(`${this.config.storage.qdrantEndpoint}/collections/${collectionName}/points/search`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(searchBody)
      });

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
      const response = await fetch(`${this.config.storage.qdrantEndpoint}/collections/${collectionName}/points/delete`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ points: ids })
      });

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
      const response = await fetch(`${this.config.storage.qdrantEndpoint}/collections/${collectionName}/points/delete`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
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

      const response = await fetch(`${this.config.storage.qdrantEndpoint}/collections/${collectionName}/points/count`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(countBody)
      });

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
      const scrollBody: Record<string, unknown> = {
        limit,
        with_payload: true,
        with_vector: false
      };
      if (filter) {
        scrollBody.filter = filter;
      }

      const response = await fetch(`${this.config.storage.qdrantEndpoint}/collections/${collectionName}/points/scroll`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(scrollBody)
      });

      if (!response.ok) {
        throw new Error(`Failed to scroll points: ${response.statusText}`);
      }

      const data = await response.json();
      return data.result.points.map((item: { id: string | number; payload: Record<string, unknown> }) => ({
        id: item.id,
        vector: [],
        payload: item.payload,
        score: 0
      }));
    } catch (error) {
      throw new StorageError(`Failed to scroll points in Qdrant`, error as Error);
    }
  }
}

export class SQLiteStorage {
  public db: Database.Database;

  constructor(private config: Config) {
    this.db = this.initializeDatabase();
  }

  private initializeDatabase(): Database.Database {
    try {
      ensureDirectory(dirname(this.config.storage.sqlitePath));
      
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
      `);

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
        modifiedTime: new Date(row.modified_time * 1000),
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

  async getFilesByDirectory(directoryPath: string): Promise<FileRecord[]> {
    try {
      const stmt = this.db.prepare('SELECT * FROM files WHERE path LIKE ?');
      const rows = stmt.all(`${directoryPath}%`) as { id: number; path: string; size: number; modified_time: number; hash: string; parent_dirs: string; chunks_json: string | null; errors_json: string | null }[];
      
      return rows.map(row => ({
        id: row.id,
        path: row.path,
        size: row.size,
        modifiedTime: new Date(row.modified_time * 1000),
        hash: row.hash,
        parentDirs: JSON.parse(row.parent_dirs),
        chunks: row.chunks_json ? JSON.parse(row.chunks_json) : [],
        errors: row.errors_json ? JSON.parse(row.errors_json) : undefined
      }));
    } catch (error) {
      throw new StorageError(`Failed to get files by directory`, error as Error);
    }
  }

  close(): void {
    this.db.close();
  }
}

export async function initializeStorage(config: Config): Promise<{ sqlite: SQLiteStorage; qdrant: QdrantClient }> {
  const sqlite = new SQLiteStorage(config);
  const qdrant = new QdrantClient(config);
  
  await qdrant.createCollection();
  
  return { sqlite, qdrant };
}

export async function initDatabase(dbPath: string): Promise<Database.Database> {
  return new Database(dbPath);
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
      const response = await fetch(`${config.storage.qdrantEndpoint}/collections/${collectionName}`);
      if (!response.ok) {
        issues.push(`Vector collection '${collectionName}' not found (normal during first-time setup)`);
        return { isConsistent: false, issues };
      }
      
      const collectionInfo = await response.json();
      const qdrantPointCount = collectionInfo.result?.points_count || 0;
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
    
    const lastIndexedStmt = sqlite.db.prepare('SELECT MAX(indexed_at) as last_indexed FROM directories WHERE indexed_at > 0');
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
    
    const directoriesDetailStmt = sqlite.db.prepare(`
      SELECT 
        d.path,
        d.status,
        d.indexed_at,
        (SELECT COUNT(*) FROM files f WHERE f.path LIKE d.path || '%') as files_count,
        (SELECT COALESCE(SUM(json_array_length(f.chunks_json)), 0) FROM files f WHERE f.path LIKE d.path || '%' AND f.chunks_json IS NOT NULL) as chunks_count
      FROM directories d
      ORDER BY d.indexed_at DESC
    `);
    const directoryDetails = directoriesDetailStmt.all() as { id: number; path: string; status: 'pending' | 'indexing' | 'completed' | 'failed'; indexed_at: number; files_count: number; chunks_count: number }[];
    
    const directories: DirectoryStatus[] = directoryDetails.map(row => {
      const errorsByDirStmt = sqlite.db.prepare(`
        SELECT errors_json FROM files 
        WHERE path LIKE ? || '%' AND errors_json IS NOT NULL
      `);
      const dirErrors = errorsByDirStmt.all(row.path) as { errors_json: string }[];
      
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
        filesCount: row.files_count,
        chunksCount: row.chunks_count,
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