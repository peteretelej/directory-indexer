import { describe, it, expect, vi } from 'vitest';
import { 
  handleIndex, 
  handleSearch, 
  handleSimilar,
  handleGet,
  handleServe,
  handleReset,
  handleStatus
} from '../src/cli-handlers.js';

vi.mock('../src/indexing.js');
vi.mock('../src/search.js');
vi.mock('../src/config.js');
vi.mock('../src/prerequisites.js');
vi.mock('../src/storage.js');
vi.mock('../src/mcp.js');
vi.mock('../src/reset.js');

describe('CLI Handlers - Unit Tests', () => {
  
  it('should call indexDirectories with correct params', async () => {
    const { indexDirectories } = await import('../src/indexing.js');
    const { loadConfig } = await import('../src/config.js');
    
    vi.mocked(loadConfig).mockResolvedValue({} as any);
    vi.mocked(indexDirectories).mockResolvedValue({ indexed: 1, skipped: 0, deleted: 0, failed: 0, errors: [] });
    vi.spyOn(console, 'log').mockImplementation(() => {});

    await handleIndex(['/test'], { verbose: false });

    expect(indexDirectories).toHaveBeenCalledWith(['/test'], {});
  });

  it('should call searchContent with correct limit', async () => {
    const { searchContent } = await import('../src/search.js');
    const { loadConfig } = await import('../src/config.js');
    
    vi.mocked(loadConfig).mockResolvedValue({} as any);
    vi.mocked(searchContent).mockResolvedValue([]);
    vi.spyOn(console, 'log').mockImplementation(() => {});

    await handleSearch('test', { limit: 5, verbose: false });

    expect(searchContent).toHaveBeenCalledWith('test', { limit: 5 });
  });

  it('should call getFileContent with correct path', async () => {
    const { getFileContent } = await import('../src/search.js');
    const { loadConfig } = await import('../src/config.js');
    
    vi.mocked(loadConfig).mockResolvedValue({} as any);
    vi.mocked(getFileContent).mockResolvedValue('content');
    vi.spyOn(console, 'log').mockImplementation(() => {});

    await handleGet('/test/file.txt', { verbose: false });

    expect(getFileContent).toHaveBeenCalledWith('/test/file.txt', undefined);
  });

  it('should pass chunks option to getFileContent', async () => {
    const { getFileContent } = await import('../src/search.js');
    const { loadConfig } = await import('../src/config.js');
    
    vi.mocked(loadConfig).mockResolvedValue({} as any);
    vi.mocked(getFileContent).mockResolvedValue('chunk content');
    vi.spyOn(console, 'log').mockImplementation(() => {});

    await handleGet('/test/file.txt', { chunks: '1-3', verbose: false });

    expect(getFileContent).toHaveBeenCalledWith('/test/file.txt', '1-3');
  });

  it('should call findSimilarFiles with correct params', async () => {
    const { findSimilarFiles } = await import('../src/search.js');
    const { loadConfig } = await import('../src/config.js');
    
    vi.mocked(loadConfig).mockResolvedValue({} as any);
    vi.mocked(findSimilarFiles).mockResolvedValue([]);
    vi.spyOn(console, 'log').mockImplementation(() => {});

    await handleSimilar('/test/file.txt', { limit: 10, verbose: false });

    expect(findSimilarFiles).toHaveBeenCalledWith('/test/file.txt', 10);
  });

  it('should call startMcpServer with config', async () => {
    const { startMcpServer } = await import('../src/mcp.js');
    const { loadConfig } = await import('../src/config.js');
    
    const mockConfig = { test: 'config' };
    vi.mocked(loadConfig).mockResolvedValue(mockConfig as any);
    vi.mocked(startMcpServer).mockResolvedValue(undefined);

    await handleServe({ verbose: false });

    expect(startMcpServer).toHaveBeenCalledWith(mockConfig);
  });

  it('should call resetEnvironment with config and options', async () => {
    const { resetEnvironment } = await import('../src/reset.js');
    const { loadConfig } = await import('../src/config.js');
    
    const mockConfig = { test: 'config' };
    vi.mocked(loadConfig).mockResolvedValue(mockConfig as any);
    vi.mocked(resetEnvironment).mockResolvedValue(undefined);

    await handleReset({ force: true, verbose: false });

    expect(resetEnvironment).toHaveBeenCalledWith(mockConfig, { force: true, verbose: false });
  });

  it('should call getIndexStatus and getServiceStatus', async () => {
    const { getIndexStatus } = await import('../src/storage.js');
    const { getServiceStatus } = await import('../src/prerequisites.js');
    const { loadConfig } = await import('../src/config.js');
    
    const mockConfig = { test: 'config' };
    const mockStatus = { directoriesIndexed: 0, filesIndexed: 0, chunksIndexed: 0, databaseSize: '0B', directories: [], errors: [], workspaces: [], workspaceHealth: { healthy: 0, warnings: 0, errors: 0, criticalIssues: [], recommendations: [] }, qdrantConsistency: { isConsistent: true, issues: [] } };
    const mockServiceStatus = { qdrant: true, embedding: true, embeddingProvider: 'ollama' };
    
    vi.mocked(loadConfig).mockResolvedValue(mockConfig as any);
    vi.mocked(getIndexStatus).mockResolvedValue(mockStatus as any);
    vi.mocked(getServiceStatus).mockResolvedValue(mockServiceStatus as any);
    vi.spyOn(console, 'log').mockImplementation(() => {});

    await handleStatus({ verbose: false });

    expect(getIndexStatus).toHaveBeenCalled();
    expect(getServiceStatus).toHaveBeenCalledWith(mockConfig);
  });

  it('should display errors when indexing fails', async () => {
    const { indexDirectories } = await import('../src/indexing.js');
    const { loadConfig } = await import('../src/config.js');
    
    vi.mocked(loadConfig).mockResolvedValue({} as any);
    vi.mocked(indexDirectories).mockResolvedValue({ indexed: 0, skipped: 0, deleted: 0, failed: 1, errors: ['File not found'] });
    vi.spyOn(console, 'log').mockImplementation(() => {});

    await handleIndex(['/test'], { verbose: false });

    expect(indexDirectories).toHaveBeenCalled();
  });

  it('should show search results with chunks when requested', async () => {
    const { searchContent } = await import('../src/search.js');
    const { loadConfig } = await import('../src/config.js');
    
    const mockResults = [{
      filePath: '/test/file.txt',
      score: 0.95,
      matchingChunks: 2,
      fileSizeBytes: 1024,
      chunks: [{ chunkId: 'chunk-1', score: 0.9 }]
    }];
    
    vi.mocked(loadConfig).mockResolvedValue({} as any);
    vi.mocked(searchContent).mockResolvedValue(mockResults);
    vi.spyOn(console, 'log').mockImplementation(() => {});

    await handleSearch('test', { limit: 5, showChunks: true, verbose: false });

    expect(searchContent).toHaveBeenCalled();
  });

  it('should show similar files results', async () => {
    const { findSimilarFiles } = await import('../src/search.js');
    const { loadConfig } = await import('../src/config.js');
    
    const mockResults = [{ filePath: '/test/similar.txt', score: 0.85, fileSizeBytes: 512 }];
    
    vi.mocked(loadConfig).mockResolvedValue({} as any);
    vi.mocked(findSimilarFiles).mockResolvedValue(mockResults);
    vi.spyOn(console, 'log').mockImplementation(() => {});

    await handleSimilar('/test/file.txt', { limit: 10, verbose: false });

    expect(findSimilarFiles).toHaveBeenCalled();
  });

  it('should handle CLI command errors', async () => {
    const { main } = await import('../src/cli.js');
    
    // Mock process.argv to trigger error paths
    const originalArgv = process.argv;
    const originalExit = process.exit;
    
    vi.spyOn(process, 'exit').mockImplementation(() => {
      throw new Error('Process exit called');
    });
    
    try {
      process.argv = ['node', 'cli.js', 'search'];
      await expect(main()).rejects.toThrow();
    } finally {
      process.argv = originalArgv;
      process.exit = originalExit;
    }
  });

  it('should handle reset cancellation', async () => {
    const { resetEnvironment } = await import('../src/reset.js');
    const { loadConfig } = await import('../src/config.js');
    
    const mockConfig = { test: 'config' };
    vi.mocked(loadConfig).mockResolvedValue(mockConfig as any);
    vi.mocked(resetEnvironment).mockRejectedValue(new Error('Reset cancelled by user'));

    try {
      await handleReset({ force: false, verbose: false });
    } catch (error) {
      expect(error).toBeInstanceOf(Error);
    }
  });

  it('should handle status with verbose errors', async () => {
    const { getIndexStatus } = await import('../src/storage.js');
    const { getServiceStatus } = await import('../src/prerequisites.js');
    const { loadConfig } = await import('../src/config.js');
    
    const mockConfig = { test: 'config' };
    const mockStatus = { 
      directoriesIndexed: 0, filesIndexed: 0, chunksIndexed: 0, databaseSize: '0B', 
      directories: [{ path: '/test', status: 'completed', filesCount: 1, chunksCount: 2, lastIndexed: null, errors: ['test error'] }], 
      errors: ['global error'], 
      workspaces: [], 
      workspaceHealth: { healthy: 0, warnings: 0, errors: 0, criticalIssues: [], recommendations: [] }, 
      qdrantConsistency: { isConsistent: false, issues: ['test issue'] } 
    };
    const mockServiceStatus = { qdrant: true, embedding: true, embeddingProvider: 'ollama' };
    
    vi.mocked(loadConfig).mockResolvedValue(mockConfig as any);
    vi.mocked(getIndexStatus).mockResolvedValue(mockStatus as any);
    vi.mocked(getServiceStatus).mockResolvedValue(mockServiceStatus as any);
    vi.spyOn(console, 'log').mockImplementation(() => {});

    await handleStatus({ verbose: true });

    expect(getIndexStatus).toHaveBeenCalled();
    expect(getServiceStatus).toHaveBeenCalledWith(mockConfig);
  });
});