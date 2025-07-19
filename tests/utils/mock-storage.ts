import { Config } from '../../src/config.js';
import { FileInfo } from '../../src/utils.js';
import { QdrantPoint, DirectoryRecord, FileRecord } from '../../src/storage.js';

export interface MockQdrantClient {
  healthCheck(): Promise<boolean>;
  createCollection(): Promise<void>;
  upsertPoints(points: QdrantPoint[]): Promise<void>;
  searchVectors(vector: number[], limit: number, threshold?: number): Promise<QdrantPoint[]>;
  deletePoints(condition: any): Promise<void>;
  deleteCollection(): Promise<void>;
  getPoint(id: string): Promise<QdrantPoint | null>;
}

export interface MockSQLiteStorage {
  db: any;
  upsertDirectory(path: string, status: 'pending' | 'indexing' | 'completed' | 'failed'): Promise<void>;
  getDirectory(path: string): Promise<DirectoryRecord | null>;
  upsertFile(fileInfo: FileInfo): Promise<void>;
  getFile(path: string): Promise<FileRecord | null>;
  close(): void;
}

export class InMemoryQdrantClient implements MockQdrantClient {
  private points: Map<string, QdrantPoint> = new Map();
  private collectionExists = false;

  constructor(_config: Config) {
    // Config not used in mock implementation
  }

  async healthCheck(): Promise<boolean> {
    return true; // Always healthy in mock
  }

  async createCollection(): Promise<void> {
    this.collectionExists = true;
  }

  async upsertPoints(points: QdrantPoint[]): Promise<void> {
    if (!this.collectionExists) {
      await this.createCollection();
    }
    
    for (const point of points) {
      this.points.set(String(point.id), point);
    }
  }

  async searchVectors(vector: number[], limit: number, threshold = 0.0): Promise<QdrantPoint[]> {
    if (!this.collectionExists) {
      return [];
    }

    const results = Array.from(this.points.values())
      .map(point => {
        // Simple cosine similarity calculation
        const similarity = this.cosineSimilarity(vector, point.vector);
        return { ...point, score: similarity };
      })
      .filter(point => point.score >= threshold)
      .sort((a, b) => (b.score || 0) - (a.score || 0))
      .slice(0, limit);

    return results;
  }

  async deletePoints(condition: any): Promise<void> {
    if (condition.filter?.must) {
      const mustConditions = condition.filter.must;
      for (const mustCondition of mustConditions) {
        if (mustCondition.key === 'filePath' && mustCondition.match?.value) {
          const filePath = mustCondition.match.value;
          for (const [id, point] of this.points.entries()) {
            if (point.payload.filePath === filePath) {
              this.points.delete(id);
            }
          }
        }
      }
    }
  }

  async deleteCollection(): Promise<void> {
    this.points.clear();
    this.collectionExists = false;
  }

  async getPoint(id: string): Promise<QdrantPoint | null> {
    return this.points.get(id) || null;
  }

  private cosineSimilarity(a: number[], b: number[]): number {
    if (a.length !== b.length) return 0;
    
    let dotProduct = 0;
    let normA = 0;
    let normB = 0;
    
    for (let i = 0; i < a.length; i++) {
      dotProduct += a[i] * b[i];
      normA += a[i] * a[i];
      normB += b[i] * b[i];
    }
    
    const magnitude = Math.sqrt(normA) * Math.sqrt(normB);
    return magnitude === 0 ? 0 : dotProduct / magnitude;
  }
}

export class InMemorySQLiteStorage implements MockSQLiteStorage {
  private directories: Map<string, DirectoryRecord> = new Map();
  private files: Map<string, FileRecord> = new Map();
  private nextId = 1;

  constructor(_config: Config) {
    // Config not used in mock implementation
  }

  get db(): any {
    return {
      // Mock DB object for compatibility
      close: () => this.close()
    };
  }

  async upsertDirectory(path: string, status: 'pending' | 'indexing' | 'completed' | 'failed'): Promise<void> {
    const existing = this.directories.get(path);
    const record: DirectoryRecord = {
      id: existing?.id || this.nextId++,
      path,
      status,
      indexedAt: new Date()
    };
    this.directories.set(path, record);
  }

  async getDirectory(path: string): Promise<DirectoryRecord | null> {
    return this.directories.get(path) || null;
  }

  async upsertFile(fileInfo: FileInfo): Promise<void> {
    const existing = this.files.get(fileInfo.path);
    const record: FileRecord = {
      id: existing?.id || this.nextId++,
      path: fileInfo.path,
      size: fileInfo.size,
      modifiedTime: fileInfo.modifiedTime,
      hash: fileInfo.hash,
      parentDirs: fileInfo.parentDirs,
      chunks: [], // Will be populated separately
      errors: []
    };
    this.files.set(fileInfo.path, record);
  }

  async getFile(path: string): Promise<FileRecord | null> {
    return this.files.get(path) || null;
  }

  close(): void {
    // No-op for in-memory storage
  }
}

export function createMockStorageClients(config: Config): {
  qdrantClient: MockQdrantClient;
  sqliteStorage: MockSQLiteStorage;
} {
  return {
    qdrantClient: new InMemoryQdrantClient(config),
    sqliteStorage: new InMemorySQLiteStorage(config)
  };
}