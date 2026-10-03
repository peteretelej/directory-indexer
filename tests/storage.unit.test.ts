import { describe, it, expect, vi } from 'vitest';
import { tmpdir } from 'os';
import { join, dirname } from 'path';
import { existsSync } from 'fs';
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

  it('should roundtrip modifiedTime with millisecond precision', async () => {
    const config = await loadConfig();
    config.storage.sqlitePath = ':memory:';
    const { SQLiteStorage } = await import('../src/storage.js');
    const storage = new SQLiteStorage(config);

    try {
      const modifiedTime = new Date('2026-09-20T10:30:45.123Z');
      const fileInfo = {
        path: '/test/file.md',
        size: 100,
        modifiedTime,
        hash: 'abc123',
        parentDirs: ['/test']
      };
      await storage.upsertFile(fileInfo);

      const record = await storage.getFile(fileInfo.path);
      expect(record).not.toBeNull();
      expect(record!.modifiedTime.getTime()).toBe(modifiedTime.getTime());

      const records = await storage.getFilesByDirectory('/test');
      expect(records).toHaveLength(1);
      expect(records[0].modifiedTime.getTime()).toBe(modifiedTime.getTime());
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

describe('SQLiteStorage fresh install', () => {
  it('should create missing nested directories before opening the database', async () => {
    const { SQLiteStorage } = await import('../src/storage.js');
    const config = await loadConfig();
    const dbPath = join(tmpdir(), `test-fresh-install-${Date.now()}`, 'nested', 'deeper', 'index.db');
    config.storage.sqlitePath = dbPath;

    const storage = new SQLiteStorage(config);
    try {
      expect(existsSync(dbPath)).toBe(true);
    } finally {
      storage.close();
      await import('fs/promises').then(fs => fs.rm(dirname(dirname(dirname(dbPath))), { recursive: true, force: true }));
    }
  });
});

async function withIsolatedStatusEnv(run: () => Promise<void>): Promise<void> {
  const originalDataDir = process.env.DIRECTORY_INDEXER_DATA_DIR;
  const originalEndpoint = process.env.QDRANT_ENDPOINT;
  const tempDir = join(tmpdir(), `test-status-${Date.now()}-${Math.random().toString(36).slice(2)}`);
  await import('fs/promises').then(fs => fs.mkdir(tempDir, { recursive: true }));
  process.env.DIRECTORY_INDEXER_DATA_DIR = tempDir;
  process.env.QDRANT_ENDPOINT = 'http://invalid-endpoint:9999';

  try {
    await run();
  } finally {
    if (originalDataDir) {
      process.env.DIRECTORY_INDEXER_DATA_DIR = originalDataDir;
    } else {
      delete process.env.DIRECTORY_INDEXER_DATA_DIR;
    }
    if (originalEndpoint) {
      process.env.QDRANT_ENDPOINT = originalEndpoint;
    } else {
      delete process.env.QDRANT_ENDPOINT;
    }
    await import('fs/promises').then(fs => fs.rm(tempDir, { recursive: true, force: true }));
  }
}

describe('getIndexStatus directory counts', () => {
  it('should count files and chunks for directory paths with LIKE metacharacters', async () => {
    await withIsolatedStatusEnv(async () => {
      const { SQLiteStorage, getIndexStatus } = await import('../src/storage.js');
      const config = loadConfig({ verbose: false });
      const storage = new SQLiteStorage(config);

      const insertDir = storage.db.prepare("INSERT INTO directories (path, status, indexed_at) VALUES (?, 'completed', 100)");
      const insertFile = storage.db.prepare('INSERT INTO files (path, size, modified_time, hash, parent_dirs, chunks_json) VALUES (?, ?, ?, ?, ?, ?)');
      const chunks = (n: number) => JSON.stringify(Array.from({ length: n }, (_, i) => ({ id: `c${i}`, content: 'x', startIndex: 0, endIndex: 1 })));

      insertDir.run('/data/reports_2026%final');
      insertFile.run('/data/reports_2026%final/a.md', 1, 1, 'h1', '[]', chunks(1));
      insertFile.run('/data/reports_2026%final/b.md', 1, 1, 'h2', '[]', chunks(2));
      insertDir.run('/data/reportsX2026Zfinal');
      insertFile.run('/data/reportsX2026Zfinal/decoy.md', 1, 1, 'h3', '[]', chunks(1));
      storage.close();

      const status = await getIndexStatus();
      const counted = status.directories.find(d => d.path === '/data/reports_2026%final');
      expect(counted).toBeDefined();
      expect(counted!.filesCount).toBe(2);
      expect(counted!.chunksCount).toBe(3);
      const decoy = status.directories.find(d => d.path === '/data/reportsX2026Zfinal');
      expect(decoy).toBeDefined();
      expect(decoy!.filesCount).toBe(1);
      expect(decoy!.chunksCount).toBe(1);
    });
  });
});

describe('getIndexStatus lastIndexed', () => {
  it('should report lastIndexed from completed directories only', async () => {
    await withIsolatedStatusEnv(async () => {
      const { SQLiteStorage, getIndexStatus } = await import('../src/storage.js');
      const config = loadConfig({ verbose: false });
      const storage = new SQLiteStorage(config);

      const insertDir = storage.db.prepare('INSERT INTO directories (path, status, indexed_at) VALUES (?, ?, ?)');
      insertDir.run('/data/ok', 'completed', 1000);
      insertDir.run('/data/broken', 'failed', 2000);
      insertDir.run('/data/running', 'indexing', 3000);
      storage.close();

      const status = await getIndexStatus();
      expect(status.lastIndexed).toBe(new Date(1000).toISOString());
    });
  });
});

describe('QdrantClient.scrollPoints pagination', () => {
  it('should follow next_page_offset across pages', async () => {
    const { QdrantClient } = await import('../src/storage.js');
    const config = await loadConfig();
    const client = new QdrantClient(config);

    const page = (points: { id: number; payload: Record<string, unknown> }[], next: number | null) => ({
      ok: true,
      json: async () => ({ result: { points, next_page_offset: next } })
    });
    const fetchMock = vi.fn()
      .mockResolvedValueOnce(page([{ id: 1, payload: { filePath: '/a' } }, { id: 2, payload: { filePath: '/b' } }], 2))
      .mockResolvedValueOnce(page([{ id: 3, payload: { filePath: '/c' } }], null));
    vi.stubGlobal('fetch', fetchMock);

    try {
      const points = await client.scrollPoints({ must: [] }, 2);
      expect(points.map(p => p.id)).toEqual([1, 2, 3]);
      expect(fetchMock).toHaveBeenCalledTimes(2);
      const secondBody = JSON.parse(fetchMock.mock.calls[1][1].body);
      expect(secondBody.offset).toBe(2);
    } finally {
      vi.unstubAllGlobals();
    }
  });

  it('should return points from a single-page response', async () => {
    const { QdrantClient } = await import('../src/storage.js');
    const config = await loadConfig();
    const client = new QdrantClient(config);

    const fetchMock = vi.fn().mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        result: {
          points: [{ id: 7, payload: { filePath: '/only' } }],
          next_page_offset: null
        }
      })
    });
    vi.stubGlobal('fetch', fetchMock);

    try {
      const points = await client.scrollPoints();
      expect(points.map(p => p.id)).toEqual([7]);
      expect(fetchMock).toHaveBeenCalledTimes(1);
    } finally {
      vi.unstubAllGlobals();
    }
  });
});