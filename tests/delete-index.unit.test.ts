import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { handleDeleteIndexTool } from '../src/mcp-handlers.js';
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
    const mockSqlite = {
      getDirectory: vi.fn().mockResolvedValue({ path: '/test/dir', status: 'completed' }),
      getFilesByDirectory: vi.fn().mockResolvedValue([
        { path: '/test/dir/file1.txt', chunks: [{ id: '0' }, { id: '1' }] },
        { path: '/test/dir/file2.txt', chunks: [{ id: '0' }] }
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

    expect(mockSqlite.getDirectory).toHaveBeenCalledWith('/test/dir');
    expect(mockSqlite.deleteFilesByDirectory).toHaveBeenCalledWith('/test/dir');
    expect(mockSqlite.deleteDirectory).toHaveBeenCalledWith('/test/dir');
    expect(mockQdrant.deletePointsByFilePath).toHaveBeenCalledTimes(2);

    const text = (result.content[0] as { type: 'text'; text: string }).text;
    expect(text).toContain('Deleted index for /test/dir');
    expect(text).toContain('removed 2 files');
    expect(text).toContain('3 chunks');
  });

  it('should return error for non-indexed directory', async () => {
    const { initializeStorage } = await import('../src/storage.js');
    const mockSqlite = {
      getDirectory: vi.fn().mockResolvedValue(null),
      close: vi.fn()
    };
    vi.mocked(initializeStorage).mockResolvedValue({
      sqlite: mockSqlite as any,
      qdrant: {} as any
    });

    const { loadConfig } = await import('../src/config.js');
    const config = loadConfig();

    await expect(handleDeleteIndexTool({ directory_path: '/not/indexed' }, config))
      .rejects.toThrow("Directory '/not/indexed' is not indexed");
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
