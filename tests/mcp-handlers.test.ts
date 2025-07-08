import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
  handleIndexTool,
  handleSearchTool,
  handleSimilarFilesTool,
  handleGetContentTool,
  handleGetChunkTool,
  handleServerInfoTool,
  formatErrorResponse
} from '../src/mcp-handlers.js';
import { loadConfig } from '../src/config.js';

// Mock all the dependencies
vi.mock('../src/indexing.js', () => ({
  indexDirectories: vi.fn()
}));

vi.mock('../src/search.js', () => ({
  searchContent: vi.fn(),
  findSimilarFiles: vi.fn(),
  getFileContent: vi.fn(),
  getChunkContent: vi.fn()
}));

vi.mock('../src/storage.js', () => ({
  getIndexStatus: vi.fn()
}));

vi.mock('../src/config.js', () => ({
  ...vi.importActual('../src/config.js'),
  loadConfig: vi.fn(),
  getAvailableWorkspaces: vi.fn()
}));

describe('MCP Handlers Unit Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('handleIndexTool', () => {
    it('should handle valid index request', async () => {
      const { indexDirectories } = await import('../src/indexing.js');
      vi.mocked(indexDirectories).mockResolvedValue({
        indexed: 5,
        skipped: 2,
        failed: 1,
        errors: ['error1']
      });

      const config = loadConfig();
      const args = { directory_path: '/path1,/path2' };

      const result = await handleIndexTool(args, config);

      expect(indexDirectories).toHaveBeenCalledWith(['/path1', '/path2'], config);
      expect(result).toEqual({
        content: [{
          type: 'text',
          text: `Indexed 5 files, skipped 2 files, 1 failed
Errors: [
  'error1'
]`
        }]
      });
    });

    it('should throw error for missing directory_path', async () => {
      const config = loadConfig();

      await expect(handleIndexTool({}, config)).rejects.toThrow('directory_path is required');
      await expect(handleIndexTool(null, config)).rejects.toThrow('directory_path is required');
      await expect(handleIndexTool({ directory_path: 123 }, config)).rejects.toThrow('directory_path is required');
    });
  });

  describe('handleSearchTool', () => {
    it('should handle valid search request', async () => {
      const { searchContent } = await import('../src/search.js');
      const { loadConfig, getAvailableWorkspaces } = await import('../src/config.js');
      const mockResults = [{ filePath: '/test.md', score: 0.9, fileSizeBytes: 1024, matchingChunks: 2, chunks: [] }];
      vi.mocked(searchContent).mockResolvedValue(mockResults);
      vi.mocked(loadConfig).mockReturnValue({ workspaces: { docs: { paths: ['/docs'], isValid: true } } } as any);
      vi.mocked(getAvailableWorkspaces).mockReturnValue(['docs']);

      const args = { query: 'test search', limit: 5, workspace: 'docs' };

      const result = await handleSearchTool(args);

      expect(searchContent).toHaveBeenCalledWith('test search', { limit: 5, workspace: 'docs' });
      expect(result).toEqual({
        content: [{
          type: 'text',
          text: JSON.stringify(mockResults, null, 2)
        }]
      });
    });

    it('should use default limit when not provided', async () => {
      const { searchContent } = await import('../src/search.js');
      vi.mocked(searchContent).mockResolvedValue([]);

      const args = { query: 'test' };

      await handleSearchTool(args);

      expect(searchContent).toHaveBeenCalledWith('test', { limit: 10, workspace: undefined });
    });

    it('should handle invalid workspace by searching all content', async () => {
      const { searchContent } = await import('../src/search.js');
      const { loadConfig, getAvailableWorkspaces } = await import('../src/config.js');
      const mockResults = [{ filePath: '/test.md', score: 0.9, fileSizeBytes: 1024, matchingChunks: 2, chunks: [] }];
      vi.mocked(searchContent).mockResolvedValue(mockResults);
      vi.mocked(loadConfig).mockReturnValue({ workspaces: { docs: { paths: ['/docs'], isValid: true } } } as any);
      vi.mocked(getAvailableWorkspaces).mockReturnValue(['docs']);

      const args = { query: 'test search', workspace: 'invalid' };

      const result = await handleSearchTool(args);

      expect(searchContent).toHaveBeenCalledWith('test search', { limit: 10, workspace: undefined });
      expect(result.content[0].text).toContain('Workspace \'invalid\' not found');
      expect(result.content[0].text).toContain('Available workspaces: docs');
    });

    it('should handle invalid workspace when no workspaces configured', async () => {
      const { searchContent } = await import('../src/search.js');
      const { loadConfig, getAvailableWorkspaces } = await import('../src/config.js');
      const mockResults = [{ filePath: '/test.md', score: 0.9, fileSizeBytes: 1024, matchingChunks: 2, chunks: [] }];
      vi.mocked(searchContent).mockResolvedValue(mockResults);
      vi.mocked(loadConfig).mockReturnValue({ workspaces: {} } as any);
      vi.mocked(getAvailableWorkspaces).mockReturnValue([]);

      const args = { query: 'test search', workspace: 'invalid' };

      const result = await handleSearchTool(args);

      expect(searchContent).toHaveBeenCalledWith('test search', { limit: 10, workspace: undefined });
      expect(result.content[0].text).toContain('no workspaces are configured');
    });

    it('should throw error for missing query', async () => {
      await expect(handleSearchTool({})).rejects.toThrow('query is required');
      await expect(handleSearchTool(null)).rejects.toThrow('query is required');
      await expect(handleSearchTool({ query: 123 })).rejects.toThrow('query is required');
    });
  });

  describe('handleSimilarFilesTool', () => {
    it('should handle valid similar files request', async () => {
      const { findSimilarFiles } = await import('../src/search.js');
      const { loadConfig, getAvailableWorkspaces } = await import('../src/config.js');
      const mockResults = [{ filePath: '/similar.md', score: 0.8, fileSizeBytes: 512 }];
      vi.mocked(findSimilarFiles).mockResolvedValue(mockResults);
      vi.mocked(loadConfig).mockReturnValue({ workspaces: { code: { paths: ['/code'], isValid: true } } } as any);
      vi.mocked(getAvailableWorkspaces).mockReturnValue(['code']);

      const args = { file_path: '/test.md', limit: 3, workspace: 'code' };

      const result = await handleSimilarFilesTool(args);

      expect(findSimilarFiles).toHaveBeenCalledWith('/test.md', 3, 'code');
      expect(result).toEqual({
        content: [{
          type: 'text',
          text: JSON.stringify(mockResults, null, 2)
        }]
      });
    });

    it('should use default limit when not provided', async () => {
      const { findSimilarFiles } = await import('../src/search.js');
      vi.mocked(findSimilarFiles).mockResolvedValue([]);

      const args = { file_path: '/test.md' };

      await handleSimilarFilesTool(args);

      expect(findSimilarFiles).toHaveBeenCalledWith('/test.md', 10, undefined);
    });

    it('should handle invalid workspace by searching all content', async () => {
      const { findSimilarFiles } = await import('../src/search.js');
      const { loadConfig, getAvailableWorkspaces } = await import('../src/config.js');
      const mockResults = [{ filePath: '/similar.md', score: 0.8, fileSizeBytes: 512 }];
      vi.mocked(findSimilarFiles).mockResolvedValue(mockResults);
      vi.mocked(loadConfig).mockReturnValue({ workspaces: { code: { paths: ['/code'], isValid: true } } } as any);
      vi.mocked(getAvailableWorkspaces).mockReturnValue(['code']);

      const args = { file_path: '/test.md', workspace: 'invalid' };

      const result = await handleSimilarFilesTool(args);

      expect(findSimilarFiles).toHaveBeenCalledWith('/test.md', 10, undefined);
      expect(result.content[0].text).toContain('Workspace \'invalid\' not found');
      expect(result.content[0].text).toContain('Available workspaces: code');
    });

    it('should handle invalid workspace when no workspaces configured', async () => {
      const { findSimilarFiles } = await import('../src/search.js');
      const { loadConfig, getAvailableWorkspaces } = await import('../src/config.js');
      const mockResults = [{ filePath: '/similar.md', score: 0.8, fileSizeBytes: 512 }];
      vi.mocked(findSimilarFiles).mockResolvedValue(mockResults);
      vi.mocked(loadConfig).mockReturnValue({ workspaces: {} } as any);
      vi.mocked(getAvailableWorkspaces).mockReturnValue([]);

      const args = { file_path: '/test.md', workspace: 'invalid' };

      const result = await handleSimilarFilesTool(args);

      expect(findSimilarFiles).toHaveBeenCalledWith('/test.md', 10, undefined);
      expect(result.content[0].text).toContain('no workspaces are configured');
    });

    it('should throw error for missing file_path', async () => {
      await expect(handleSimilarFilesTool({})).rejects.toThrow('file_path is required');
      await expect(handleSimilarFilesTool(null)).rejects.toThrow('file_path is required');
      await expect(handleSimilarFilesTool({ file_path: 123 })).rejects.toThrow('file_path is required');
    });
  });

  describe('handleGetContentTool', () => {
    it('should handle valid get content request', async () => {
      const { getFileContent } = await import('../src/search.js');
      vi.mocked(getFileContent).mockResolvedValue('file content here');

      const args = { file_path: '/test.md', chunks: '1-3' };

      const result = await handleGetContentTool(args);

      expect(getFileContent).toHaveBeenCalledWith('/test.md', '1-3');
      expect(result).toEqual({
        content: [{
          type: 'text',
          text: 'file content here'
        }]
      });
    });

    it('should handle request without chunks parameter', async () => {
      const { getFileContent } = await import('../src/search.js');
      vi.mocked(getFileContent).mockResolvedValue('full file content');

      const args = { file_path: '/test.md' };

      await handleGetContentTool(args);

      expect(getFileContent).toHaveBeenCalledWith('/test.md', undefined);
    });

    it('should throw error for missing file_path', async () => {
      await expect(handleGetContentTool({})).rejects.toThrow('file_path is required');
      await expect(handleGetContentTool(null)).rejects.toThrow('file_path is required');
      await expect(handleGetContentTool({ file_path: 123 })).rejects.toThrow('file_path is required');
    });
  });

  describe('handleGetChunkTool', () => {
    it('should handle valid get chunk request', async () => {
      const { getChunkContent } = await import('../src/search.js');
      vi.mocked(getChunkContent).mockResolvedValue('chunk content here');

      const args = { file_path: '/test.md', chunk_id: 'chunk_1' };

      const result = await handleGetChunkTool(args);

      expect(getChunkContent).toHaveBeenCalledWith('/test.md', 'chunk_1');
      expect(result).toEqual({
        content: [{
          type: 'text',
          text: 'chunk content here'
        }]
      });
    });

    it('should throw error for missing file_path', async () => {
      await expect(handleGetChunkTool({ chunk_id: 'chunk_1' })).rejects.toThrow('file_path and chunk_id are required');
    });

    it('should throw error for missing chunk_id', async () => {
      await expect(handleGetChunkTool({ file_path: '/test.md' })).rejects.toThrow('file_path and chunk_id are required');
    });

    it('should throw error for missing both parameters', async () => {
      await expect(handleGetChunkTool({})).rejects.toThrow('file_path and chunk_id are required');
      await expect(handleGetChunkTool(null)).rejects.toThrow('file_path and chunk_id are required');
    });
  });

  describe('handleServerInfoTool', () => {
    it('should handle server info request', async () => {
      const { getIndexStatus } = await import('../src/storage.js');
      const mockStatus = {
        directoriesIndexed: 2,
        filesIndexed: 10,
        chunksIndexed: 45,
        databaseSize: '1.2 MB',
        lastIndexed: '2025-01-01T00:00:00Z',
        errors: [],
        directories: [],
        workspaces: [],
        qdrantConsistency: { isConsistent: true, issues: [] }
      };
      vi.mocked(getIndexStatus).mockResolvedValue(mockStatus);

      const result = await handleServerInfoTool('1.0.0');

      expect(getIndexStatus).toHaveBeenCalled();
      expect(result).toEqual({
        content: [{
          type: 'text',
          text: JSON.stringify({
            name: 'directory-indexer',
            version: '1.0.0',
            status: mockStatus
          }, null, 2)
        }]
      });
    });
  });

  describe('formatErrorResponse', () => {
    it('should format Error instance correctly', () => {
      const error = new Error('Test error message');

      const result = formatErrorResponse(error);

      expect(result).toEqual({
        content: [{
          type: 'text',
          text: 'Error: Test error message'
        }],
        isError: true
      });
    });

    it('should handle non-Error values', () => {
      const result = formatErrorResponse('string error');

      expect(result).toEqual({
        content: [{
          type: 'text',
          text: 'Error: Unknown error'
        }],
        isError: true
      });
    });

    it('should handle null/undefined errors', () => {
      expect(formatErrorResponse(null)).toEqual({
        content: [{
          type: 'text',
          text: 'Error: Unknown error'
        }],
        isError: true
      });

      expect(formatErrorResponse(undefined)).toEqual({
        content: [{
          type: 'text',
          text: 'Error: Unknown error'
        }],
        isError: true
      });
    });
  });
});