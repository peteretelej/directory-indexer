import { describe, it, expect, vi } from 'vitest';
import { resetEnvironment } from '../src/reset.js';

vi.mock('../src/storage.js');
vi.mock('../src/utils.js');

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
});