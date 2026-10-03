import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { sep } from 'path';
import { handleDeleteIndexTool } from '../src/mcp-handlers.js';
import { normalizePath } from '../src/utils.js';
import { getMcpTools } from '../src/mcp.js';

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
  getIndexStatus: vi.fn(),
  SQLiteStorage: vi.fn().mockImplementation(() => ({
    getDirectories: vi.fn().mockReturnValue([]),
    close: vi.fn(),
    db: {}
  })),
  initializeStorage: vi.fn(),
  closeAllStorage: vi.fn()
}));

vi.mock('../src/config.js', () => ({
  ...vi.importActual('../src/config.js'),
  loadConfig: vi.fn(),
  getAvailableWorkspaces: vi.fn()
}));

vi.mock('../src/prerequisites.js', () => ({
  validateIndexPrerequisites: vi.fn(),
  validateSearchPrerequisites: vi.fn()
}));

vi.mock('../src/path-validation.js', () => ({
  validatePathWithinIndexedDirs: vi.fn(),
  resolveIndexedDirectories: vi.fn().mockReturnValue(new Set())
}));

vi.mock('../src/logger.js', () => ({
  log: vi.fn(),
  initLogLevel: vi.fn()
}));

vi.mock('@modelcontextprotocol/sdk/server/index.js', () => ({
  Server: vi.fn().mockImplementation(() => ({
    setRequestHandler: vi.fn(),
    connect: vi.fn().mockResolvedValue(undefined),
    sendLoggingMessage: vi.fn()
  }))
}));

vi.mock('@modelcontextprotocol/sdk/server/stdio.js', () => ({
  StdioServerTransport: vi.fn()
}));

describe('handleDeleteIndexTool', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should delete index for an indexed directory', async () => {
    const { initializeStorage } = await import('../src/storage.js');
    const dirPath = normalizePath('/test/dir');
    const mockSqlite = {
      getDirectory: vi.fn().mockResolvedValue({ path: dirPath, status: 'completed' }),
      getDirectoryByCaseInsensitive: vi.fn(),
      getFilesByDirectory: vi.fn().mockResolvedValue([
        { path: `${dirPath}${sep}file1.txt`, chunks: [{ id: '0' }, { id: '1' }] },
        { path: `${dirPath}${sep}file2.txt`, chunks: [{ id: '0' }] }
      ]),
      deleteFilesByDirectory: vi.fn().mockReturnValue(2),
      deleteDirectory: vi.fn(),
      getDirectories: vi.fn().mockReturnValue([]),
      close: vi.fn()
    };
    const mockQdrant = {
      deletePointsByFilePath: vi.fn().mockResolvedValue(undefined)
    };
    vi.mocked(initializeStorage).mockResolvedValue({
      sqlite: mockSqlite as any,
      qdrant: mockQdrant as any
    });

    const { loadConfig } = await import('../src/config.js');
    const config = loadConfig();

    const result = await handleDeleteIndexTool({ directory_path: '/test/dir' }, config);

    expect(mockSqlite.getDirectory).toHaveBeenCalledWith(dirPath);
    expect(mockSqlite.deleteFilesByDirectory).toHaveBeenCalledWith(dirPath);
    expect(mockSqlite.deleteDirectory).toHaveBeenCalledWith(dirPath);
    expect(mockQdrant.deletePointsByFilePath).toHaveBeenCalledTimes(2);

    const text = (result.content[0] as { type: 'text'; text: string }).text;
    expect(text).toContain(`Deleted index for ${dirPath}`);
    expect(text).toContain('removed 2 files');
    expect(text).toContain('3 chunks');
  });

  it('should return error for non-indexed directory', async () => {
    const { initializeStorage } = await import('../src/storage.js');
    const dirPath = normalizePath('/not/indexed');
    const mockSqlite = {
      getDirectory: vi.fn().mockResolvedValue(null),
      getDirectoryByCaseInsensitive: vi.fn().mockResolvedValue(null),
      close: vi.fn()
    };
    vi.mocked(initializeStorage).mockResolvedValue({
      sqlite: mockSqlite as any,
      qdrant: {} as any
    });

    const { loadConfig } = await import('../src/config.js');
    const config = loadConfig();

    await expect(handleDeleteIndexTool({ directory_path: '/not/indexed' }, config))
      .rejects.toThrow(`Directory '${dirPath}' is not indexed`);
  });

  it('should release the directory mutex when storage initialization fails', async () => {
    const { initializeStorage } = await import('../src/storage.js');
    const dirPath = normalizePath('/test/dir');
    const mockSqlite = {
      getDirectory: vi.fn().mockResolvedValue({ path: dirPath, status: 'completed' }),
      getDirectoryByCaseInsensitive: vi.fn(),
      getFilesByDirectory: vi.fn().mockResolvedValue([]),
      deleteFilesByDirectory: vi.fn().mockReturnValue(0),
      deleteDirectory: vi.fn(),
      getDirectories: vi.fn().mockReturnValue([]),
      close: vi.fn()
    };
    vi.mocked(initializeStorage)
      .mockRejectedValueOnce(new Error('storage unavailable'))
      .mockResolvedValue({
        sqlite: mockSqlite as any,
        qdrant: { deletePointsByFilePath: vi.fn().mockResolvedValue(undefined) } as any
      });

    const { loadConfig } = await import('../src/config.js');
    const config = loadConfig();

    await expect(handleDeleteIndexTool({ directory_path: '/test/dir' }, config))
      .rejects.toThrow('storage unavailable');

    let wedgeTimer: ReturnType<typeof setTimeout> | undefined;
    try {
      const result = await Promise.race([
        handleDeleteIndexTool({ directory_path: '/test/dir' }, config),
        new Promise<never>((_, reject) => {
          wedgeTimer = setTimeout(() => reject(new Error('delete_index wedged the directory mutex')), 1000);
        })
      ]);
      expect(result).toBeDefined();
    } finally {
      clearTimeout(wedgeTimer);
    }
  });

  describe('delete_index path normalization', () => {
    beforeEach(() => {
      vi.clearAllMocks();
    });

    function createMockSqlite(storedPath: string): Record<string, ReturnType<typeof vi.fn>> {
      return {
        getDirectory: vi.fn().mockResolvedValue(null),
        getDirectoryByCaseInsensitive: vi.fn().mockResolvedValue({ path: storedPath, status: 'completed' }),
        getFilesByDirectory: vi.fn().mockResolvedValue([
          { path: `${storedPath}${sep}file1.txt`, chunks: [{ id: '0' }] }
        ]),
        deleteFilesByDirectory: vi.fn().mockReturnValue(1),
        deleteDirectory: vi.fn(),
        getDirectories: vi.fn().mockReturnValue([]),
        close: vi.fn()
      };
    }

    it('should delete the indexed directory when given a trailing-slash spelling', async () => {
      const { initializeStorage } = await import('../src/storage.js');
      const storedPath = normalizePath('/test/dir');
      const inputPath = storedPath + sep;
      const mockSqlite = createMockSqlite(storedPath);
      mockSqlite.getDirectory.mockResolvedValue({ path: storedPath, status: 'completed' });
      vi.mocked(initializeStorage).mockResolvedValue({
        sqlite: mockSqlite as any,
        qdrant: { deletePointsByFilePath: vi.fn().mockResolvedValue(undefined) } as any
      });

      const { loadConfig } = await import('../src/config.js');
      const config = loadConfig();

      const result = await handleDeleteIndexTool({ directory_path: inputPath }, config);

      expect(mockSqlite.getDirectory).toHaveBeenCalledWith(storedPath);
      expect(mockSqlite.getFilesByDirectory).toHaveBeenCalledWith(storedPath);
      expect(mockSqlite.deleteFilesByDirectory).toHaveBeenCalledWith(storedPath);
      expect(mockSqlite.deleteDirectory).toHaveBeenCalledWith(storedPath);
      const text = (result.content[0] as { type: 'text'; text: string }).text;
      expect(text).toContain(`Deleted index for ${storedPath}`);
    });

    it.runIf(process.platform === 'win32')('should resolve a case-different spelling to the stored path on windows', async () => {
      const { initializeStorage } = await import('../src/storage.js');
      const storedPath = normalizePath('/test/dir');
      const caseDifferent = storedPath.toUpperCase();
      expect(caseDifferent).not.toBe(storedPath);

      const mockSqlite = createMockSqlite(storedPath);
      vi.mocked(initializeStorage).mockResolvedValue({
        sqlite: mockSqlite as any,
        qdrant: { deletePointsByFilePath: vi.fn().mockResolvedValue(undefined) } as any
      });

      const { loadConfig } = await import('../src/config.js');
      const config = loadConfig();

      const result = await handleDeleteIndexTool({ directory_path: caseDifferent }, config);

      expect(mockSqlite.getDirectory).toHaveBeenCalledWith(caseDifferent);
      expect(mockSqlite.getDirectoryByCaseInsensitive).toHaveBeenCalled();
      expect(mockSqlite.getFilesByDirectory).toHaveBeenCalledWith(storedPath);
      expect(mockSqlite.deleteFilesByDirectory).toHaveBeenCalledWith(storedPath);
      expect(mockSqlite.deleteDirectory).toHaveBeenCalledWith(storedPath);
      const text = (result.content[0] as { type: 'text'; text: string }).text;
      expect(text).toContain(`Deleted index for ${storedPath}`);
    });
  });

  it('should throw error for missing directory_path', async () => {
    const { loadConfig } = await import('../src/config.js');
    const config = loadConfig();

    await expect(handleDeleteIndexTool({}, config)).rejects.toThrow('directory_path is required');
    await expect(handleDeleteIndexTool(null, config)).rejects.toThrow('directory_path is required');
  });
});

describe('getMcpTools DISABLE_DESTRUCTIVE gating', () => {
  const originalEnv = process.env.DISABLE_DESTRUCTIVE;

  afterEach(() => {
    if (originalEnv !== undefined) {
      process.env.DISABLE_DESTRUCTIVE = originalEnv;
    } else {
      delete process.env.DISABLE_DESTRUCTIVE;
    }
  });

  it('should include delete_index by default', () => {
    delete process.env.DISABLE_DESTRUCTIVE;
    const tools = getMcpTools();
    expect(tools.some(t => t.name === 'delete_index')).toBe(true);
  });

  it('should exclude delete_index when DISABLE_DESTRUCTIVE=true', () => {
    process.env.DISABLE_DESTRUCTIVE = 'true';
    const tools = getMcpTools();
    expect(tools.some(t => t.name === 'delete_index')).toBe(false);
  });

  it('should include delete_index when DISABLE_DESTRUCTIVE has other values', () => {
    process.env.DISABLE_DESTRUCTIVE = 'false';
    const tools = getMcpTools();
    expect(tools.some(t => t.name === 'delete_index')).toBe(true);
  });
});
