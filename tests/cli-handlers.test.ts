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
});