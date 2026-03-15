import { describe, it, expect } from 'vitest';
import { tmpdir } from 'os';
import { join } from 'path';
import { loadConfig } from '../src/config.js';
import { clearDatabase, clearVectorCollection } from '../src/storage.js';

describe('Storage Operations', () => {
  it('should initialize SQLite storage', async () => {
    const { SQLiteStorage } = await import('../src/storage.js');
    const config = await loadConfig();
    config.storage.sqlitePath = ':memory:';
    const storage = new SQLiteStorage(config);
    
    expect(storage).toBeDefined();
    expect(storage.db).toBeDefined();
    
    storage.close();
  });

  it('should create QdrantClient', async () => {
    const { QdrantClient } = await import('../src/storage.js');
    const config = await loadConfig();
    const client = new QdrantClient(config);
    
    expect(client).toBeDefined();
    expect(typeof client.healthCheck).toBe('function');
    expect(typeof client.createCollection).toBe('function');
  });

  it('should handle file operations with invalid data', async () => {
    const config = await loadConfig();
    config.storage.sqlitePath = ':memory:';
    const { SQLiteStorage } = await import('../src/storage.js');
    const storage = new SQLiteStorage(config);
    
    try {
      await storage.upsertFile({
        path: '',
        size: -1,
        modifiedTime: new Date('invalid'),
        hash: '',
        parentDirs: []
      });
    } catch (error) {
      expect(error).toBeInstanceOf(Error);
    } finally {
      storage.close();
    }
  });

  it('should clear database when no file exists', async () => {
    const originalDataDir = process.env.DIRECTORY_INDEXER_DATA_DIR;
    try {
      process.env.DIRECTORY_INDEXER_DATA_DIR = join(tmpdir(), `test-clear-db-nonexistent-${Date.now()}`);
      const config = loadConfig({ verbose: false });
      
      const result = await clearDatabase(config);
      expect(result).toBe(true);
    } finally {
      if (originalDataDir) {
        process.env.DIRECTORY_INDEXER_DATA_DIR = originalDataDir;
      } else {
        delete process.env.DIRECTORY_INDEXER_DATA_DIR;
      }
    }
  });

  it('should handle clearVectorCollection with invalid endpoint', async () => {
    const originalDataDir = process.env.DIRECTORY_INDEXER_DATA_DIR;
    try {
      process.env.DIRECTORY_INDEXER_DATA_DIR = join(tmpdir(), `test-clear-collection-${Date.now()}`);
      const config = loadConfig({ verbose: false });
      config.storage.qdrantEndpoint = 'http://invalid-endpoint:9999';
      
      await expect(clearVectorCollection(config)).rejects.toThrow();
    } finally {
      if (originalDataDir) {
        process.env.DIRECTORY_INDEXER_DATA_DIR = originalDataDir;
      } else {
        delete process.env.DIRECTORY_INDEXER_DATA_DIR;
      }
    }
  });
});

describe('SQLite WAL Mode', () => {
  it('should enable WAL journal mode', async () => {
    const { SQLiteStorage } = await import('../src/storage.js');
    const config = await loadConfig();
    // Use a temp file since :memory: databases don't persist WAL mode
    const tmpPath = join(tmpdir(), `test-wal-mode-${Date.now()}.db`);
    config.storage.sqlitePath = tmpPath;
    const storage = new SQLiteStorage(config);

    try {
      const result = storage.db.pragma('journal_mode') as { journal_mode: string }[];
      expect(result[0].journal_mode).toBe('wal');
    } finally {
      storage.close();
      // Clean up
      await import('fs/promises').then(fs => fs.unlink(tmpPath).catch(() => {}));
      await import('fs/promises').then(fs => fs.unlink(tmpPath + '-wal').catch(() => {}));
      await import('fs/promises').then(fs => fs.unlink(tmpPath + '-shm').catch(() => {}));
    }
  });
});

describe('SQLiteStorage.getDirectories', () => {
  it('should return all directory paths', async () => {
    const { SQLiteStorage } = await import('../src/storage.js');
    const config = await loadConfig();
    config.storage.sqlitePath = ':memory:';
    const storage = new SQLiteStorage(config);

    try {
      storage.db.prepare('INSERT INTO directories (path, status) VALUES (?, ?)').run('/dir1', 'completed');
      storage.db.prepare('INSERT INTO directories (path, status) VALUES (?, ?)').run('/dir2', 'pending');

      const dirs = storage.getDirectories();
      expect(dirs.sort()).toEqual(['/dir1', '/dir2']);
    } finally {
      storage.close();
    }
  });

  it('should return empty array when no directories exist', async () => {
    const { SQLiteStorage } = await import('../src/storage.js');
    const config = await loadConfig();
    config.storage.sqlitePath = ':memory:';
    const storage = new SQLiteStorage(config);

    try {
      const dirs = storage.getDirectories();
      expect(dirs).toEqual([]);
    } finally {
      storage.close();
    }
  });
});

describe('Storage Error Handling', () => {
  it('should handle StorageError', async () => {
    const { StorageError } = await import('../src/storage.js');
    
    const error = new StorageError('Test error', new Error('Cause'));
    expect(error.name).toBe('StorageError');
    expect(error.message).toBe('Test error');
    expect(error.cause).toBeInstanceOf(Error);
  });

  it('should handle Qdrant errors', async () => {
    const { QdrantClient } = await import('../src/storage.js');
    const config = await loadConfig();
    config.storage.qdrantEndpoint = 'http://invalid:9999';
    
    const client = new QdrantClient(config);
    const isHealthy = await client.healthCheck();
    expect(isHealthy).toBe(false);
  });

  it('should handle getIndexStatus with temp directory', async () => {
    const originalDataDir = process.env.DIRECTORY_INDEXER_DATA_DIR;
    try {
      const tempDir = join(tmpdir(), `test-index-status-${Date.now()}`);
      await import('fs/promises').then(fs => fs.mkdir(tempDir, { recursive: true }));
      process.env.DIRECTORY_INDEXER_DATA_DIR = tempDir;
      
      const { getIndexStatus } = await import('../src/storage.js');
      
      const status = await getIndexStatus();
      expect(status.directoriesIndexed).toBeGreaterThanOrEqual(0);
      expect(Array.isArray(status.errors)).toBe(true);
    } finally {
      if (originalDataDir) {
        process.env.DIRECTORY_INDEXER_DATA_DIR = originalDataDir;
      } else {
        delete process.env.DIRECTORY_INDEXER_DATA_DIR;
      }
    }
  });
});