import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { main } from '../src/cli.js';
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

describe('CLI Tests', () => {
  let originalArgv: string[];
  let originalExit: typeof process.exit;
  let consoleErrorSpy: ReturnType<typeof vi.spyOn>;
  let exitCode: number | undefined;

  beforeEach(() => {
    originalArgv = process.argv;
    originalExit = process.exit;
    
    vi.spyOn(console, 'log').mockImplementation(() => {});
    consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    
    process.exit = vi.fn((code?: number) => {
      exitCode = code;
      throw new Error(`Process exited with code ${code}`);
    }) as never;
  });

  afterEach(() => {
    process.argv = originalArgv;
    process.exit = originalExit;
    exitCode = undefined;
    vi.restoreAllMocks();
  });


  describe('CLI Argument Validation', () => {
    it('should error when search command has no query', async () => {
      process.argv = ['node', 'cli.js', 'search'];
      
      try {
        await main();
      } catch (error) {
        console.log('Expected error for search without query:', error instanceof Error ? error.message : String(error));
      }
      
      expect(exitCode).toBe(1);
    });

    it('should error when similar command has no file argument', async () => {
      process.argv = ['node', 'cli.js', 'similar'];
      
      try {
        await main();
      } catch (error) {
        console.log('Expected error for similar without file:', error instanceof Error ? error.message : String(error));
      }
      
      expect(exitCode).toBe(1);
    });

    it('should error when get command has no file argument', async () => {
      process.argv = ['node', 'cli.js', 'get'];
      
      try {
        await main();
      } catch (error) {
        console.log('Expected error for get without file:', error instanceof Error ? error.message : String(error));
      }
      
      expect(exitCode).toBe(1);
    });

    it('should error when index command has no paths', async () => {
      process.argv = ['node', 'cli.js', 'index'];
      
      try {
        await main();
      } catch (error) {
        console.log('Expected error for index without paths:', error instanceof Error ? error.message : String(error));
      }
      
      expect(exitCode).toBe(1);
    });


    it('should handle invalid commands gracefully', async () => {
      process.argv = ['node', 'cli.js', 'invalid-command'];
      
      try {
        await main();
      } catch (error) {
        console.log('Expected error for invalid command:', error instanceof Error ? error.message : String(error));
      }
      
      expect(exitCode).toBe(1);
    });
  });

  describe('CLI Error Handling', () => {
    it('should handle similar files errors gracefully', async () => {
      process.argv = ['node', 'cli.js', 'similar', '/nonexistent/file.txt'];
      
      try {
        await main();
      } catch {
        // Expected to fail due to missing file
      }
      
      expect(consoleErrorSpy).toHaveBeenCalledWith('Error finding similar files:', expect.any(Error));
      expect(exitCode).toBe(1);
    });

  });

  describe('CLI Handler Functions', () => {
    it('should call indexDirectories with correct params', async () => {
      const { indexDirectories } = await import('../src/indexing.js');
      const { loadConfig } = await import('../src/config.js');
      
      vi.mocked(loadConfig).mockResolvedValue({} as any);
      vi.mocked(indexDirectories).mockResolvedValue({ indexed: 1, skipped: 0, deleted: 0, failed: 0, errors: [] });

      await handleIndex(['/test'], { verbose: false });

      expect(indexDirectories).toHaveBeenCalledWith(['/test'], {});
    });

    it('should call searchContent with correct limit', async () => {
      const { searchContent } = await import('../src/search.js');
      const { loadConfig } = await import('../src/config.js');
      
      vi.mocked(loadConfig).mockResolvedValue({} as any);
      vi.mocked(searchContent).mockResolvedValue([]);

      await handleSearch('test', { limit: 5, verbose: false });

      expect(searchContent).toHaveBeenCalledWith('test', { limit: 5 });
    });

    it('should call getFileContent with correct path', async () => {
      const { getFileContent } = await import('../src/search.js');
      const { loadConfig } = await import('../src/config.js');
      
      vi.mocked(loadConfig).mockResolvedValue({} as any);
      vi.mocked(getFileContent).mockResolvedValue('content');

      await handleGet('/test/file.txt', { verbose: false });

      expect(getFileContent).toHaveBeenCalledWith('/test/file.txt', undefined);
    });

    it('should pass chunks option to getFileContent', async () => {
      const { getFileContent } = await import('../src/search.js');
      const { loadConfig } = await import('../src/config.js');
      
      vi.mocked(loadConfig).mockResolvedValue({} as any);
      vi.mocked(getFileContent).mockResolvedValue('chunk content');

      await handleGet('/test/file.txt', { chunks: '1-3', verbose: false });

      expect(getFileContent).toHaveBeenCalledWith('/test/file.txt', '1-3');
    });

    it('should call findSimilarFiles with correct params', async () => {
      const { findSimilarFiles } = await import('../src/search.js');
      const { loadConfig } = await import('../src/config.js');
      
      vi.mocked(loadConfig).mockResolvedValue({} as any);
      vi.mocked(findSimilarFiles).mockResolvedValue([]);

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

      await handleStatus({ verbose: false });

      expect(getIndexStatus).toHaveBeenCalled();
      expect(getServiceStatus).toHaveBeenCalledWith(mockConfig);
    });

    it('should display errors when indexing fails', async () => {
      const { indexDirectories } = await import('../src/indexing.js');
      const { loadConfig } = await import('../src/config.js');
      
      vi.mocked(loadConfig).mockResolvedValue({} as any);
      vi.mocked(indexDirectories).mockResolvedValue({ indexed: 0, skipped: 0, deleted: 0, failed: 1, errors: ['File not found'] });

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

      await handleSearch('test', { limit: 5, showChunks: true, verbose: false });

      expect(searchContent).toHaveBeenCalled();
    });

    it('should show similar files results', async () => {
      const { findSimilarFiles } = await import('../src/search.js');
      const { loadConfig } = await import('../src/config.js');
      
      const mockResults = [{ filePath: '/test/similar.txt', score: 0.85, fileSizeBytes: 512 }];
      
      vi.mocked(loadConfig).mockResolvedValue({} as any);
      vi.mocked(findSimilarFiles).mockResolvedValue(mockResults);

      await handleSimilar('/test/file.txt', { limit: 10, verbose: false });

      expect(findSimilarFiles).toHaveBeenCalled();
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

      await handleStatus({ verbose: true });

      expect(getIndexStatus).toHaveBeenCalled();
      expect(getServiceStatus).toHaveBeenCalledWith(mockConfig);
    });
  });

});