import { describe, it, expect, vi } from 'vitest';
import { 
  handleIndex, 
  handleSearch, 
  handleGet 
} from '../src/cli-handlers.js';

vi.mock('../src/indexing.js');
vi.mock('../src/search.js');
vi.mock('../src/config.js');
vi.mock('../src/prerequisites.js');

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
});