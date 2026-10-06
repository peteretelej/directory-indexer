import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { tmpdir } from 'os';
import { join } from 'path';
import { promises as fs, realpathSync, symlinkSync } from 'fs';
import { loadConfig } from '../src/config.js';
import { indexDirectories, scanDirectory } from '../src/indexing.js';
import { findSimilarFiles } from '../src/search.js';
import { getFileInfo, normalizePath } from '../src/utils.js';
import { initializeStorage } from '../src/storage.js';
import { generateEmbedding } from '../src/embedding.js';
import type { SQLiteStorage, QdrantClient, FileRecord } from '../src/storage.js';

vi.mock('../src/storage.js', () => ({
  initializeStorage: vi.fn(),
  ensureCollectionDimensions: vi.fn()
}));

vi.mock('../src/embedding.js', () => ({
  generateEmbedding: vi.fn()
}));

const mockInitializeStorage = vi.mocked(initializeStorage);
const mockGenerateEmbedding = vi.mocked(generateEmbedding);

interface StorageMocks {
  sqlite: {
    upsertDirectory: ReturnType<typeof vi.fn>;
    upsertFile: ReturnType<typeof vi.fn>;
    getFile: ReturnType<typeof vi.fn>;
    getFilesByDirectory: ReturnType<typeof vi.fn>;
    deleteFile: ReturnType<typeof vi.fn>;
  };
  qdrant: {
    deletePointsByFilePath: ReturnType<typeof vi.fn>;
    upsertPoints: ReturnType<typeof vi.fn>;
    searchPoints: ReturnType<typeof vi.fn>;
  };
}

function makeStorageMocks(): StorageMocks {
  const sqlite = {
    upsertDirectory: vi.fn().mockResolvedValue(undefined),
    upsertFile: vi.fn().mockResolvedValue(undefined),
    getFile: vi.fn().mockResolvedValue(undefined),
    getFilesByDirectory: vi.fn().mockResolvedValue([]),
    deleteFile: vi.fn().mockResolvedValue(undefined)
  };
  const qdrant = {
    deletePointsByFilePath: vi.fn().mockResolvedValue(undefined),
    upsertPoints: vi.fn().mockResolvedValue(undefined),
    searchPoints: vi.fn().mockResolvedValue([])
  };
  mockInitializeStorage.mockResolvedValue({
    sqlite: sqlite as unknown as SQLiteStorage,
    qdrant: qdrant as unknown as QdrantClient
  });
  return { sqlite, qdrant };
}

async function recordFor(filePath: string, overrides: Partial<FileRecord> = {}): Promise<FileRecord> {
  const info = await getFileInfo(filePath);
  return {
    id: 1,
    path: info.path,
    size: info.size,
    modifiedTime: new Date(info.modifiedTime),
    hash: info.hash,
    parentDirs: info.parentDirs,
    chunks: [],
    ...overrides
  };
}

describe('Indexing Robustness', () => {
  let tempRoot: string;
  let consoleErrorSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(async () => {
    vi.clearAllMocks();
    tempRoot = await fs.mkdtemp(join(realpathSync(tmpdir()), 'indexing-robust-'));
    consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  afterEach(async () => {
    consoleErrorSpy.mockRestore();
    await fs.rm(tempRoot, { recursive: true, force: true });
  });

  describe('F6: failed files self-repair', () => {
    it('persists the failure to errors_json and reprocesses the file on the next run', async () => {
      const { sqlite, qdrant } = makeStorageMocks();
      const config = await loadConfig();
      config.verbose = false;
      const filePath = join(tempRoot, 'a.txt');
      await fs.writeFile(filePath, 'hello world');

      mockGenerateEmbedding.mockRejectedValueOnce(new Error('embedding provider down'));
      const firstRun = await indexDirectories([tempRoot], config);

      expect(firstRun.failed).toBe(1);
      expect(firstRun.indexed).toBe(0);
      expect(sqlite.upsertFile).toHaveBeenCalledTimes(2);
      const failureCall = sqlite.upsertFile.mock.calls[1];
      const failureMessage = String((failureCall[2] as string[])[0]);
      expect(failureMessage).toContain('Failed to process');
      expect(failureMessage).toContain('embedding provider down');

      const stored = await recordFor(filePath, {
        chunks: [{ id: '0', content: 'hello world', startIndex: 0, endIndex: 'hello world'.length }],
        errors: [failureMessage]
      });
      sqlite.getFile.mockResolvedValue(stored);
      mockGenerateEmbedding.mockResolvedValue([0.1, 0.2, 0.3, 0.4]);
      sqlite.upsertFile.mockClear();
      qdrant.deletePointsByFilePath.mockClear();
      mockGenerateEmbedding.mockClear();

      const secondRun = await indexDirectories([tempRoot], config);

      expect(secondRun.indexed).toBe(1);
      expect(secondRun.failed).toBe(0);
      expect(qdrant.deletePointsByFilePath).toHaveBeenCalledWith(normalizePath(filePath));
      expect(mockGenerateEmbedding).toHaveBeenCalledTimes(1);
      expect(sqlite.upsertFile).toHaveBeenCalledTimes(1);
      expect(sqlite.upsertFile.mock.calls[0].length).toBe(2);
    });

    it('still skips unchanged files whose errors are only informational or absent', async () => {
      const { sqlite } = makeStorageMocks();
      const config = await loadConfig();
      config.verbose = false;
      const skippedPath = join(tempRoot, 'skipped.txt');
      const cleanPath = join(tempRoot, 'clean.txt');
      await fs.writeFile(skippedPath, 'previously non-UTF-8');
      await fs.writeFile(cleanPath, 'plain content');

      sqlite.getFile.mockImplementation(async (path: string) => {
        if (path === normalizePath(skippedPath)) {
          return recordFor(skippedPath, {
            errors: ['Skipped: file appears to be non-UTF-8 encoded']
          });
        }
        if (path === normalizePath(cleanPath)) {
          return recordFor(cleanPath);
        }
        return undefined;
      });

      const result = await indexDirectories([tempRoot], config);

      expect(result.skipped).toBe(2);
      expect(result.indexed).toBe(0);
      expect(result.failed).toBe(0);
      expect(mockGenerateEmbedding).not.toHaveBeenCalled();
    });
  });

  describe('F12: per-directory failure status', () => {
    it('marks the failing directory failed even when the error text omits the directory path', async () => {
      const { sqlite } = makeStorageMocks();
      const config = await loadConfig();
      config.verbose = false;
      const alphaDir = join(tempRoot, 'alpha');
      const betaDir = join(tempRoot, 'beta');
      await fs.mkdir(alphaDir);
      await fs.mkdir(betaDir);
      await fs.writeFile(join(alphaDir, 'a.txt'), 'alpha content');
      await fs.writeFile(join(betaDir, 'b.txt'), 'beta content');

      mockGenerateEmbedding.mockImplementation(async (text: string) => {
        if (text.includes('alpha content')) {
          throw new Error('provider exploded');
        }
        return [0.1, 0.2];
      });

      const result = await indexDirectories([alphaDir, betaDir], config);

      expect(result.failed).toBe(1);
      expect(result.indexed).toBe(1);
      const statusCalls = sqlite.upsertDirectory.mock.calls;
      expect(statusCalls).toContainEqual([normalizePath(alphaDir), 'failed']);
      expect(statusCalls).toContainEqual([normalizePath(betaDir), 'completed']);
      expect(statusCalls).not.toContainEqual([normalizePath(alphaDir), 'completed']);
    });
  });

  describe('F16: older mtime falls through to hash verify', () => {
    it('skips an older mtime with identical hash and reprocesses one with a different hash', async () => {
      const { sqlite, qdrant } = makeStorageMocks();
      const config = await loadConfig();
      config.verbose = false;
      const samePath = join(tempRoot, 'same.txt');
      const diffPath = join(tempRoot, 'diff.txt');
      await fs.writeFile(samePath, 'unchanged body');
      await fs.writeFile(diffPath, 'changed body');

      const sameRecord = await recordFor(samePath);
      sameRecord.modifiedTime = new Date(sameRecord.modifiedTime.getTime() + 60000);
      const diffRecord = await recordFor(diffPath, { hash: 'different-hash' });
      diffRecord.modifiedTime = new Date(diffRecord.modifiedTime.getTime() + 60000);

      sqlite.getFile.mockImplementation(async (path: string) => {
        if (path === normalizePath(samePath)) return sameRecord;
        if (path === normalizePath(diffPath)) return diffRecord;
        return undefined;
      });
      mockGenerateEmbedding.mockResolvedValue([0.1, 0.2]);

      const result = await indexDirectories([tempRoot], config);

      expect(result.skipped).toBe(1);
      expect(result.indexed).toBe(1);
      expect(qdrant.deletePointsByFilePath).toHaveBeenCalledTimes(1);
      expect(qdrant.deletePointsByFilePath).toHaveBeenCalledWith(normalizePath(diffPath));
    });
  });

  describe('F46: symlink/junction cycles terminate', () => {
    it('terminates on a real directory cycle and returns the real files', async () => {
      const realDir = join(tempRoot, 'real');
      await fs.mkdir(realDir);
      await fs.writeFile(join(realDir, 'f.txt'), 'cycle test content');

      const linkPath = join(realDir, 'loop');
      try {
        if (process.platform === 'win32') {
          symlinkSync(tempRoot, linkPath, 'junction');
        } else {
          symlinkSync(tempRoot, linkPath, 'dir');
        }
      } catch (error) {
        console.log(
          `Skipping F46 cycle check: cannot create a directory link here (${(error as Error).message})`
        );
        return;
      }

      const files = await scanDirectory(tempRoot, {
        ignorePatterns: [],
        maxFileSize: 10485760,
        respectGitignore: false
      });

      const paths = files.map(f => f.path);
      expect(paths).toContain(normalizePath(join(realDir, 'f.txt')));
      expect(paths.filter(p => p.endsWith('f.txt'))).toHaveLength(1);
    });
  });

  describe('F18: embed input cap for non-indexed files', () => {
    it('caps the text handed to the provider at 24000 chars', async () => {
      makeStorageMocks();
      const filePath = join(tempRoot, 'large.txt');
      await fs.writeFile(filePath, 'x'.repeat(40000));

      const results = await findSimilarFiles(filePath);

      expect(mockGenerateEmbedding).toHaveBeenCalledTimes(1);
      expect(mockGenerateEmbedding.mock.calls[0][0]).toHaveLength(24000);
      expect(results).toEqual([]);
    });
  });
});
