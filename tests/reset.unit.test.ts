import { describe, it, expect, vi } from 'vitest';
import { resetEnvironment } from '../src/reset.js';

vi.mock('../src/storage.js');
vi.mock('../src/utils.js');

vi.mock('readline', async () => {
  const { EventEmitter } = await import('node:events');
  const { vi } = await import('vitest');
  return {
    createInterface: vi.fn((): import('node:readline').Interface => {
      const rl = new EventEmitter() as unknown as import('node:readline').Interface & {
        question: (question: string, callback: (answer: string) => void) => void;
      };
      rl.question = () => {};
      return rl;
    })
  };
});

describe('Reset Unit Tests', () => {

  it('should call getResetPreview', async () => {
    const { getResetPreview, clearDatabase, clearVectorCollection } = await import('../src/storage.js');

    vi.mocked(getResetPreview).mockResolvedValue({
      sqliteExists: false,
      sqliteSize: '0 KB',
      qdrantCollectionExists: false,
      qdrantVectorCount: 0
    });
    vi.mocked(clearDatabase).mockResolvedValue(true);
    vi.mocked(clearVectorCollection).mockResolvedValue(true);
    vi.spyOn(console, 'log').mockImplementation(() => {});

    const config = { storage: { sqlitePath: ':memory:', qdrantCollection: 'test' } } as any;

    await resetEnvironment(config, { force: true });

    expect(getResetPreview).toHaveBeenCalledWith(config);
  });

  it('should call clearDatabase and clearVectorCollection', async () => {
    const { getResetPreview, clearDatabase, clearVectorCollection } = await import('../src/storage.js');

    vi.mocked(getResetPreview).mockResolvedValue({
      sqliteExists: true,
      sqliteSize: '1 MB',
      qdrantCollectionExists: true,
      qdrantVectorCount: 100
    });
    vi.mocked(clearDatabase).mockResolvedValue(true);
    vi.mocked(clearVectorCollection).mockResolvedValue(true);
    vi.spyOn(console, 'log').mockImplementation(() => {});

    const config = { storage: { sqlitePath: '/test.db', qdrantCollection: 'test' } } as any;

    await resetEnvironment(config, { force: true, verbose: true });

    expect(clearDatabase).toHaveBeenCalledWith(config);
    expect(clearVectorCollection).toHaveBeenCalledWith(config);
  });

  it('should throw error when user cancels', async () => {
    const { getResetPreview } = await import('../src/storage.js');
    const { readlineSync } = await import('../src/utils.js');

    vi.mocked(getResetPreview).mockResolvedValue({
      sqliteExists: false,
      sqliteSize: '0 KB',
      qdrantCollectionExists: false,
      qdrantVectorCount: 0
    });
    vi.mocked(readlineSync).mockResolvedValue('n');

    const config = { storage: { sqlitePath: ':memory:', qdrantCollection: 'test' } } as any;

    await expect(resetEnvironment(config, { force: false })).rejects.toThrow('Reset cancelled by user');
  });

  it('should call clearDatabase when force is true', async () => {
    const { getResetPreview, clearDatabase, clearVectorCollection } = await import('../src/storage.js');

    vi.mocked(getResetPreview).mockResolvedValue({
      sqliteExists: false,
      sqliteSize: '0 KB',
      qdrantCollectionExists: false,
      qdrantVectorCount: 0
    });
    vi.mocked(clearDatabase).mockResolvedValue(true);
    vi.mocked(clearVectorCollection).mockResolvedValue(true);
    vi.spyOn(console, 'log').mockImplementation(() => {});

    const config = { storage: { sqlitePath: ':memory:', qdrantCollection: 'test' } } as any;

    await resetEnvironment(config, { force: true });

    expect(clearDatabase).toHaveBeenCalled();
    expect(clearVectorCollection).toHaveBeenCalled();
  });

  it('aborts before SQLite when clearing the Qdrant collection fails', async () => {
    const { getResetPreview, clearDatabase, clearVectorCollection } = await import('../src/storage.js');

    vi.mocked(getResetPreview).mockResolvedValue({
      sqliteExists: true,
      sqliteSize: '1 MB',
      qdrantCollectionExists: true,
      qdrantVectorCount: 100
    });
    vi.mocked(clearVectorCollection).mockRejectedValue(new Error('qdrant unavailable'));
    vi.mocked(clearDatabase).mockResolvedValue(true);
    vi.mocked(clearDatabase).mockClear();
    vi.spyOn(console, 'log').mockImplementation(() => {});

    const config = { storage: { sqlitePath: '/test.db', qdrantCollection: 'test' } } as any;

    await expect(resetEnvironment(config, { force: true, verbose: true })).rejects.toThrow('qdrant unavailable');

    expect(clearDatabase).not.toHaveBeenCalled();
  });

  it('reports honest failure when SQLite cleanup fails after vectors are deleted', async () => {
    const { getResetPreview, clearDatabase, clearVectorCollection } = await import('../src/storage.js');

    vi.mocked(getResetPreview).mockResolvedValue({
      sqliteExists: true,
      sqliteSize: '1 MB',
      qdrantCollectionExists: true,
      qdrantVectorCount: 100
    });
    vi.mocked(clearVectorCollection).mockResolvedValue(true);
    vi.mocked(clearDatabase).mockRejectedValue(new Error('database file locked'));
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    logSpy.mockClear();

    const config = { storage: { sqlitePath: '/test.db', qdrantCollection: 'test' } } as any;

    await expect(resetEnvironment(config, { force: true })).rejects.toThrow('database file locked');

    const output = logSpy.mock.calls.map(call => call.join(' ')).join('\n');
    expect(output).toContain('Vector data deleted from Qdrant collection: test');
    expect(output).toContain('SQLite cleanup failed');
    expect(output).toContain('database file locked');
    expect(output).toContain('directory-indexer reset');
    expect(output).not.toContain('ready for fresh indexing');
  });

  it('prints the success message only when both stores clear cleanly', async () => {
    const { getResetPreview, clearDatabase, clearVectorCollection } = await import('../src/storage.js');

    vi.mocked(getResetPreview).mockResolvedValue({
      sqliteExists: true,
      sqliteSize: '1 MB',
      qdrantCollectionExists: true,
      qdrantVectorCount: 100
    });
    vi.mocked(clearVectorCollection).mockResolvedValue(true);
    vi.mocked(clearDatabase).mockResolvedValue(true);
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    logSpy.mockClear();

    const config = { storage: { sqlitePath: '/test.db', qdrantCollection: 'test' } } as any;

    await resetEnvironment(config, { force: true });

    const output = logSpy.mock.calls.map(call => call.join(' ')).join('\n');
    expect(output).toContain('Reset complete. Directory-indexer is ready for fresh indexing.');
  });

  it('resolves with an empty answer when the prompt closes before any input', async () => {
    const { createInterface } = await import('readline');
    const actual = await vi.importActual<typeof import('../src/utils.js')>('../src/utils.js');

    const promise = actual.readlineSync('\nContinue? (y/N): ');
    const rl = vi.mocked(createInterface).mock.results[0]?.value;
    expect(rl).toBeDefined();
    rl?.emit('close');

    let timer: ReturnType<typeof setTimeout> | undefined;
    try {
      const hung = new Promise<never>((_, reject) => {
        timer = setTimeout(() => reject(new Error('readlineSync did not settle on stdin close')), 1000);
      });
      await expect(Promise.race([promise, hung])).resolves.toBe('');
    } finally {
      clearTimeout(timer);
    }
  }, 2000);

  it('reports cancellation when the prompt resolves without an answer', async () => {
    const { getResetPreview } = await import('../src/storage.js');
    const { readlineSync } = await import('../src/utils.js');

    vi.mocked(getResetPreview).mockResolvedValue({
      sqliteExists: true,
      sqliteSize: '1 MB',
      qdrantCollectionExists: true,
      qdrantVectorCount: 100
    });
    vi.mocked(readlineSync).mockResolvedValue('');

    const config = { storage: { sqlitePath: '/test.db', qdrantCollection: 'test' } } as any;

    await expect(resetEnvironment(config, {})).rejects.toThrow('Reset cancelled by user');
  }, 2000);

  it('omits the Qdrant endpoint line for the default endpoint', async () => {
    const { getResetPreview, clearDatabase, clearVectorCollection } = await import('../src/storage.js');
    const { readlineSync } = await import('../src/utils.js');

    vi.mocked(getResetPreview).mockResolvedValue({
      sqliteExists: true,
      sqliteSize: '1 MB',
      qdrantCollectionExists: true,
      qdrantVectorCount: 100
    });
    vi.mocked(clearDatabase).mockResolvedValue(true);
    vi.mocked(clearVectorCollection).mockResolvedValue(true);
    vi.mocked(readlineSync).mockResolvedValue('y');
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    logSpy.mockClear();

    const config = { storage: { sqlitePath: '/test.db', qdrantCollection: 'test', qdrantEndpoint: 'http://127.0.0.1:6333' } } as any;

    await resetEnvironment(config, {});

    const output = logSpy.mock.calls.map(call => call.join(' ')).join('\n');
    expect(output).not.toContain('Qdrant endpoint:');
  });

  it('shows the Qdrant endpoint line for a custom endpoint', async () => {
    const { getResetPreview, clearDatabase, clearVectorCollection } = await import('../src/storage.js');
    const { readlineSync } = await import('../src/utils.js');

    vi.mocked(getResetPreview).mockResolvedValue({
      sqliteExists: true,
      sqliteSize: '1 MB',
      qdrantCollectionExists: true,
      qdrantVectorCount: 100
    });
    vi.mocked(clearDatabase).mockResolvedValue(true);
    vi.mocked(clearVectorCollection).mockResolvedValue(true);
    vi.mocked(readlineSync).mockResolvedValue('y');
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    logSpy.mockClear();

    const config = { storage: { sqlitePath: '/test.db', qdrantCollection: 'test', qdrantEndpoint: 'http://qdrant.internal:6333' } } as any;

    await resetEnvironment(config, {});

    const output = logSpy.mock.calls.map(call => call.join(' ')).join('\n');
    expect(output).toContain('Qdrant endpoint: http://qdrant.internal:6333');
  });
});
