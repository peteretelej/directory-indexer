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
  id: string;
  vector: number[];
  payload: {
    filePath: string;
    chunkId: string;
    parentDirectories: string[];
  };
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
      const response = await fetch(`${this.config.storage.qdrantEndpoint}/health`);
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
        throw new Error(`Failed to upsert points: ${response.statusText}`);
      }
    } catch (error) {
      throw new StorageError(`Failed to upsert points to Qdrant`, error as Error);
    }
  }

  async searchPoints(vector: number[], limit: number = 10): Promise<QdrantPoint[]> {
    const collectionName = this.config.storage.qdrantCollection;
    
    try {
      const response = await fetch(`${this.config.storage.qdrantEndpoint}/collections/${collectionName}/points/search`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          vector,
          limit,
          with_payload: true
        })
      });

      if (!response.ok) {
        throw new Error(`Failed to search points: ${response.statusText}`);
      }

      const data = await response.json();
      return data.result.map((item: any) => ({
        id: item.id,
        vector: item.vector,
        payload: item.payload,
        score: item.score
      }));
    } catch (error) {
      throw new StorageError(`Failed to search points in Qdrant`, error as Error);
    }
  }

  async deletePoints(ids: string[]): Promise<void> {
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
}

export class SQLiteStorage {
  private db: Database.Database;

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
      const row = stmt.get(path) as any;
      
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
      const row = stmt.get(path) as any;
      
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

  async getFilesByDirectory(directoryPath: string): Promise<FileRecord[]> {
    try {
      const stmt = this.db.prepare('SELECT * FROM files WHERE path LIKE ?');
      const rows = stmt.all(`${directoryPath}%`) as any[];
      
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

export async function addFile(_fileRecord: FileRecord): Promise<void> {
  throw new Error('addFile requires database instance');
}

export async function getFileByPath(_path: string): Promise<FileRecord | null> {
  throw new Error('getFileByPath requires database instance');
}

export async function createQdrantCollection(_name: string, _vectorDim: number): Promise<void> {
  throw new Error('createQdrantCollection requires client instance');
}

export async function upsertPoints(_points: QdrantPoint[]): Promise<void> {
  throw new Error('upsertPoints requires client instance');
}

export async function searchVectors(_vector: number[], _limit: number): Promise<QdrantPoint[]> {
  throw new Error('searchVectors requires client instance');
}