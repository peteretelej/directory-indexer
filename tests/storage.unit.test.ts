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

function jsonResponse(body: unknown, ok = true, status = 200) {
  return {
    ok,
    status,
    statusText: ok ? 'OK' : 'Not Found',
    json: async () => body,
    text: async () => JSON.stringify(body)
  };
}

describe('QdrantClient auth headers', () => {
  it('should send the api-key header on every request type when the key is set', async () => {
    const { QdrantClient } = await import('../src/storage.js');
    const config = await loadConfig();
    config.storage.qdrantApiKey = 'secret-key';
    const collection = config.storage.qdrantCollection;
    const client = new QdrantClient(config);

    const fetchMock = vi.fn(async (...args: Parameters<typeof fetch>) => {
      const [input, init] = args;
      const url = String(input);
      const method = init?.method ?? 'GET';
      if (url.endsWith('/healthz')) return jsonResponse({ status: 'ok' });
      if (method === 'PUT' && url.endsWith(`/collections/${collection}`)) return jsonResponse({ result: {} });
      if (method === 'GET' && url.endsWith(`/collections/${collection}`)) return jsonResponse({}, false, 404);
      if (url.endsWith('/points/search')) return jsonResponse({ result: [] });
      if (url.endsWith('/points/delete')) return jsonResponse({ result: {} });
      if (url.endsWith('/points/count')) return jsonResponse({ result: { count: 2 } });
      if (url.endsWith('/points/scroll')) return jsonResponse({ result: { points: [], next_page_offset: null } });
      if (url.endsWith('/points')) return jsonResponse({ result: {} });
      return jsonResponse({}, false, 404);
    });
    vi.stubGlobal('fetch', fetchMock);

    try {
      await client.healthCheck();
      await client.createCollection(1536);
      await client.getCollectionInfo();
      await client.upsertPoints([{
        id: '0f1e2d3c-4b5a-6978-8796-a5b4c3d2e1f0',
        vector: [0.1],
        payload: { filePath: '/a.md', chunkId: '0', fileHash: 'h', content: 'c', parentDirectories: [] }
      }]);
      await client.searchPoints([0.1], 5);
      await client.deletePointsByFilePath('/a.md');
      await client.countPoints();
      await client.scrollPoints();
    } finally {
      vi.unstubAllGlobals();
    }

    const requests = fetchMock.mock.calls.map(call => {
      const [input, init] = call as Parameters<typeof fetch>;
      return {
        url: String(input),
        method: init?.method ?? 'GET',
        headers: (init?.headers ?? {}) as Record<string, string>
      };
    });

    expect(requests.length).toBeGreaterThanOrEqual(8);
    expect(requests.some(r => r.url.endsWith('/healthz'))).toBe(true);
    expect(requests.some(r => r.method === 'PUT' && r.url.endsWith(`/collections/${collection}`))).toBe(true);
    expect(requests.some(r => r.method === 'GET' && r.url.endsWith(`/collections/${collection}`))).toBe(true);
    expect(requests.some(r => r.method === 'PUT' && r.url.endsWith('/points'))).toBe(true);
    expect(requests.some(r => r.url.endsWith('/points/search'))).toBe(true);
    expect(requests.some(r => r.url.endsWith('/points/delete'))).toBe(true);
    expect(requests.some(r => r.url.endsWith('/points/count'))).toBe(true);
    expect(requests.some(r => r.url.endsWith('/points/scroll'))).toBe(true);
    for (const request of requests) {
      expect(request.headers['api-key']).toBe('secret-key');
    }
  });

  it('should send no api-key header when the key is not set', async () => {
    const { QdrantClient } = await import('../src/storage.js');
    const config = await loadConfig();
    config.storage.qdrantApiKey = undefined;
    const client = new QdrantClient(config);

    const fetchMock = vi.fn(async (..._args: Parameters<typeof fetch>) => jsonResponse({ status: 'ok' }));
    vi.stubGlobal('fetch', fetchMock);

    try {
      await client.healthCheck();
      await client.getCollectionInfo();
    } finally {
      vi.unstubAllGlobals();
    }

    expect(fetchMock).toHaveBeenCalledTimes(2);
    for (const call of fetchMock.mock.calls) {
      const [, init] = call as Parameters<typeof fetch>;
      const headers = (init?.headers ?? {}) as Record<string, string>;
      expect(headers['api-key']).toBeUndefined();
      expect(headers['Content-Type']).toBe('application/json');
    }
  });
});

describe('ensureCollectionDimensions', () => {
  it('should create a missing collection with the given dimension and record meta', async () => {
    const { SQLiteStorage, QdrantClient, ensureCollectionDimensions } = await import('../src/storage.js');

    async function ensureWithFreshInstances(dimension: number) {
      const config = await loadConfig();
      config.storage.sqlitePath = ':memory:';
      const sqlite = new SQLiteStorage(config);
      const qdrant = new QdrantClient(config);

      const createBodies: unknown[] = [];
      const fetchMock = vi.fn(async (...args: Parameters<typeof fetch>) => {
        const [, init] = args;
        if ((init?.method ?? 'GET') === 'PUT') {
          createBodies.push(JSON.parse(String(init?.body)));
          return jsonResponse({ result: {} });
        }
        return jsonResponse({}, false, 404);
      });
      vi.stubGlobal('fetch', fetchMock);

      try {
        await ensureCollectionDimensions(sqlite, qdrant, config, dimension);
      } finally {
        vi.unstubAllGlobals();
      }

      expect(createBodies).toEqual([{ vectors: { size: dimension, distance: 'Cosine' } }]);
      expect(sqlite.getMeta('embedding_provider')).toBe(config.embedding.provider);
      expect(sqlite.getMeta('embedding_dims')).toBe(String(dimension));
      return sqlite;
    }

    const first = await ensureWithFreshInstances(1536);
    first.close();
    const second = await ensureWithFreshInstances(384);
    second.close();
  });

  it('should throw reset guidance when the existing collection has a different size', async () => {
    const { SQLiteStorage, QdrantClient, StorageError, ensureCollectionDimensions } = await import('../src/storage.js');
    const config = await loadConfig();
    config.storage.sqlitePath = ':memory:';
    const sqlite = new SQLiteStorage(config);
    const qdrant = new QdrantClient(config);

    const fetchMock = vi.fn(async () => jsonResponse({
      result: {
        points_count: 3,
        config: { params: { vectors: { size: 768, distance: 'Cosine' } } }
      }
    }));
    vi.stubGlobal('fetch', fetchMock);

    let error: Error | null = null;
    try {
      await ensureCollectionDimensions(sqlite, qdrant, config, 1536);
    } catch (caught) {
      error = caught as Error;
    } finally {
      vi.unstubAllGlobals();
    }

    expect(error).toBeInstanceOf(StorageError);
    expect(error!.message).toContain('768');
    expect(error!.message).toContain('1536');
    expect(error!.message).toContain('directory-indexer reset');
    expect(fetchMock).toHaveBeenCalledTimes(1);
    sqlite.close();
  });
});

describe('QdrantClient missing collection tolerance', () => {
  it('should treat a missing collection as empty results and successful deletes', async () => {
    const { QdrantClient } = await import('../src/storage.js');
    const config = await loadConfig();
    const client = new QdrantClient(config);

    const fetchMock = vi.fn(async () => jsonResponse({}, false, 404));
    vi.stubGlobal('fetch', fetchMock);

    try {
      expect(await client.searchPoints([0.1], 5)).toEqual([]);
      expect(await client.countPoints()).toBe(0);
      expect(await client.scrollPoints()).toEqual([]);
      await expect(client.deletePointsByFilePath('/gone.md')).resolves.toBeUndefined();
    } finally {
      vi.unstubAllGlobals();
    }
  });
});

describe('Qdrant point IDs', () => {
  it('should be deterministic and UUID-shaped for the same path and chunk', async () => {
    const { pointIdFor } = await import('../src/indexing.js');

    const first = pointIdFor('/docs/report.md', '3');
    const second = pointIdFor('/docs/report.md', '3');

    expect(second).toBe(first);
    expect(first).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/);
  });

  it('should give different IDs to files whose hashes collided under the old scheme', async () => {
    const { pointIdFor } = await import('../src/indexing.js');

    const hashA = `000f4240${'0'.repeat(56)}`;
    const hashB = `001e8480${'0'.repeat(56)}`;
    const oldPointId = (hash: string, chunkIndex: number) =>
      (parseInt(hash.slice(0, 8), 16) % 1000000) * 1000 + chunkIndex;

    expect(oldPointId(hashA, 0)).toBe(oldPointId(hashB, 0));
    expect(pointIdFor('/a.md', '0')).not.toBe(pointIdFor('/b.md', '0'));
  });
});