import { promises as fs } from 'fs';
import { generateEmbedding } from './embedding.js';
import { initializeStorage } from './storage.js';
import { fileExists } from './utils.js';

export interface SearchOptions {
  limit?: number;
  threshold?: number;
  workspace?: string;
}

export interface ChunkMatch {
  chunkId: string;
  score: number;
}

export interface SearchResult {
  filePath: string;
  score: number;
  fileSizeBytes: number;
  matchingChunks: number;
  chunks: ChunkMatch[];
}

export interface SimilarFile {
  filePath: string;
  score: number;
  fileSizeBytes: number;
}

export class SearchError extends Error {
  constructor(message: string, public override cause?: Error) {
    super(message);
    this.name = 'SearchError';
  }
}


function buildWorkspaceFilter(workspacePaths: string[]): Record<string, unknown> {
  return {
    must: [
      {
        key: "parentDirectories",
        match: { any: workspacePaths }
      }
    ]
  };
}

export async function searchContent(query: string, options: SearchOptions = {}): Promise<SearchResult[]> {
  const { limit = 10, threshold = 0.0, workspace } = options;
  
  try {
    const config = (await import('./config.js')).loadConfig();
    const { getWorkspacePaths } = await import('./config.js');
    const { sqlite, qdrant } = await initializeStorage(config);
    
    const queryEmbedding = await generateEmbedding(query, config);
    
    // Get workspace paths if workspace is specified
    const workspacePaths = workspace ? getWorkspacePaths(config, workspace) : [];
    
    // Use Qdrant filtering for workspace searches
    const filter = workspace && workspacePaths.length > 0 ? buildWorkspaceFilter(workspacePaths) : undefined;
    const points = await qdrant.searchPoints(queryEmbedding, limit, filter);
    
    // Group points by file path
    const fileGroups = new Map<string, Array<{ score: number; chunkId: string }>>();
    
    for (const point of points) {
      const score = point.score ?? 0;
      if (score < threshold) continue;
      
      const filePath = point.payload.filePath;
      
      if (!fileGroups.has(filePath)) {
        fileGroups.set(filePath, []);
      }
      
      fileGroups.get(filePath)!.push({
        score,
        chunkId: point.payload.chunkId
      });
    }
    
    // Calculate average score per file and sort
    const results: SearchResult[] = [];
    for (const [filePath, chunks] of fileGroups.entries()) {
      const avgScore = chunks.reduce((sum, chunk) => sum + chunk.score, 0) / chunks.length;
      
      // Sort chunks by score (best first) and create chunk matches
      const sortedChunks = chunks.sort((a, b) => b.score - a.score);
      const chunkMatches: ChunkMatch[] = sortedChunks.map(chunk => ({
        chunkId: chunk.chunkId,
        score: chunk.score
      }));
      
      // Get file size from SQLite database
      const fileRecord = await sqlite.getFile(filePath);
      const fileSizeBytes = fileRecord?.size ?? 0;
      
      results.push({
        filePath,
        score: avgScore,
        fileSizeBytes,
        matchingChunks: chunks.length,
        chunks: chunkMatches
      });
    }
    
    // Sort by average score and return top results
    return results
      .sort((a, b) => b.score - a.score)
      .slice(0, limit);
  } catch (error) {
    throw new SearchError(`Failed to search content`, error as Error);
  }
}

export async function findSimilarFiles(filePath: string, limit: number = 5, workspace?: string): Promise<SimilarFile[]> {
  try {
    if (!await fileExists(filePath)) {
      throw new Error(`File not found: ${filePath}`);
    }
    
    const config = (await import('./config.js')).loadConfig();
    const { getWorkspacePaths } = await import('./config.js');
    const { sqlite, qdrant } = await initializeStorage(config);
    
    // Get workspace paths if workspace is specified
    const workspacePaths = workspace ? getWorkspacePaths(config, workspace) : [];
    const filter = workspace && workspacePaths.length > 0 ? buildWorkspaceFilter(workspacePaths) : undefined;
    
    // Get embedding from file record or read file directly
    let embedding: number[];
    const fileRecord = await sqlite.getFile(filePath);
    if (fileRecord && fileRecord.chunks.length > 0) {
      embedding = await generateEmbedding(fileRecord.chunks[0].content, config);
    } else {
      const content = await fs.readFile(filePath, 'utf-8');
      embedding = await generateEmbedding(content, config);
    }
    
    const points = await qdrant.searchPoints(embedding, limit + 1, filter);
    
    const filteredPoints = points
      .filter(point => point.payload.filePath !== filePath)
      .slice(0, limit);
    
    const results: SimilarFile[] = [];
    for (const point of filteredPoints) {
      const pointFileRecord = await sqlite.getFile(point.payload.filePath);
      const fileSizeBytes = pointFileRecord?.size ?? 0;
      
      results.push({
        filePath: point.payload.filePath,
        score: point.score ?? 0,
        fileSizeBytes
      });
    }
    
    return results;
  } catch (error) {
    throw new SearchError(`Failed to find similar files`, error as Error);
  }
}

export async function getChunkContent(filePath: string, chunkId: string): Promise<string> {
  try {
    if (!await fileExists(filePath)) {
      throw new Error(`File not found: ${filePath}`);
    }
    
    const config = (await import('./config.js')).loadConfig();
    const { sqlite } = await initializeStorage(config);
    
    const fileRecord = await sqlite.getFile(filePath);
    if (!fileRecord || fileRecord.chunks.length === 0) {
      throw new Error(`File not indexed: ${filePath}`);
    }
    
    const chunk = fileRecord.chunks.find(c => c.id === chunkId);
    if (!chunk) {
      throw new Error(`Chunk ${chunkId} not found in file: ${filePath}`);
    }
    
    return chunk.content;
  } catch (error) {
    throw new SearchError(`Failed to get chunk content`, error as Error);
  }
}

export async function getFileContent(filePath: string, chunks?: string): Promise<string> {
  try {
    if (!await fileExists(filePath)) {
      throw new Error(`File not found: ${filePath}`);
    }
    
    const config = (await import('./config.js')).loadConfig();
    const { sqlite } = await initializeStorage(config);
    
    const fileRecord = await sqlite.getFile(filePath);
    
    if (!chunks) {
      return await fs.readFile(filePath, 'utf-8');
    }
    
    if (!fileRecord || fileRecord.chunks.length === 0) {
      return await fs.readFile(filePath, 'utf-8');
    }
    
    const chunkRange = parseChunkRange(chunks);
    const selectedChunks = fileRecord.chunks.filter(chunk => {
      const chunkNum = parseInt(chunk.id);
      return chunkNum >= chunkRange.start && chunkNum <= chunkRange.end;
    });
    
    return selectedChunks.map(chunk => chunk.content).join('');
  } catch (error) {
    throw new SearchError(`Failed to get file content`, error as Error);
  }
}

function parseChunkRange(chunks: string): { start: number; end: number } {
  if (chunks.includes('-')) {
    const [start, end] = chunks.split('-').map(num => parseInt(num.trim()));
    return { start: start || 0, end: end || start || 0 };
  }
  
  const num = parseInt(chunks);
  return { start: num, end: num };
}