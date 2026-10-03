import { describe, it, expect, vi, beforeEach } from 'vitest';
import { searchContent } from '../src/search.js';
import { initializeStorage } from '../src/storage.js';

vi.mock('../src/embedding.js', () => ({
  generateEmbedding: vi.fn().mockResolvedValue([0.1, 0.2, 0.3])
}));

vi.mock('../src/storage.js', () => ({
  initializeStorage: vi.fn()
}));

type ScoredPoint = { payload: { filePath: string; chunkId: string }; score: number };

function stubStorage(scoredPoints: ScoredPoint[]): ReturnType<typeof vi.fn> {
  const searchPoints = vi.fn().mockResolvedValue(scoredPoints);
  vi.mocked(initializeStorage).mockResolvedValue({
    sqlite: { getFile: vi.fn().mockResolvedValue(null), close: vi.fn() },
    qdrant: { searchPoints }
  } as unknown as Awaited<ReturnType<typeof initializeStorage>>);
  return searchPoints;
}

describe('searchContent threshold', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should over-fetch and filter by threshold before slicing to the limit', async () => {
    const searchPoints = stubStorage([
      { payload: { filePath: '/high.md', chunkId: '0' }, score: 0.9 },
      { payload: { filePath: '/mid.md', chunkId: '0' }, score: 0.5 },
      { payload: { filePath: '/low.md', chunkId: '0' }, score: 0.2 }
    ]);

    const results = await searchContent('query', { limit: 2, threshold: 0.3 });

    expect(searchPoints).toHaveBeenCalledWith(expect.anything(), 100, undefined);
    expect(results.map(r => r.filePath)).toEqual(['/high.md', '/mid.md']);
  });

  it('should fetch exactly the limit when no threshold is set', async () => {
    const searchPoints = stubStorage([
      { payload: { filePath: '/a.md', chunkId: '0' }, score: 0.9 },
      { payload: { filePath: '/b.md', chunkId: '0' }, score: 0.8 }
    ]);

    const results = await searchContent('query', { limit: 3 });

    expect(searchPoints).toHaveBeenCalledWith(expect.anything(), 3, undefined);
    expect(results.map(r => r.filePath)).toEqual(['/a.md', '/b.md']);
  });
});
