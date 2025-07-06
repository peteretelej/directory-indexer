import { promises as fs } from 'fs';
import { generateEmbedding } from './embedding.js';
import { initializeStorage } from './storage.js';
import { fileExists } from './utils.js';

export interface SearchOptions {
  limit?: number;
  threshold?: number;
  directoryPath?: string;
}

export interface SearchResult {
  filePath: string;
  chunkId: string;
  content: string;
  score: number;
  parentDirectories: string[];
}

export interface SimilarFile {
  filePath: string;
  score: number;
  parentDirectories: string[];
}

export class SearchError extends Error {
  constructor(message: string, public override cause?: Error) {
    super(message);
    this.name = 'SearchError';
  }
}

export async function searchContent(query: string, options: SearchOptions = {}): Promise<SearchResult[]> {
  const { limit = 10, threshold = 0.0 } = options;
  
  try {
    const config = (await import('./config.js')).loadConfig();
    const { qdrant } = await initializeStorage(config);
    
    const queryEmbedding = await generateEmbedding(query, config);
    const points = await qdrant.searchPoints(queryEmbedding, limit);
    
    return points
      .filter(point => (point.score ?? 0) >= threshold)
      .map(point => ({
        filePath: point.payload.filePath,
        chunkId: point.payload.chunkId,
        content: '',
        score: point.score ?? 0,
        parentDirectories: point.payload.parentDirectories
      }));
  } catch (error) {
    throw new SearchError(`Failed to search content`, error as Error);
  }
}

export async function findSimilarFiles(filePath: string, limit: number = 5): Promise<SimilarFile[]> {
  try {
    if (!await fileExists(filePath)) {
      throw new Error(`File not found: ${filePath}`);
    }
    
    const config = (await import('./config.js')).loadConfig();
    const { sqlite, qdrant } = await initializeStorage(config);
    
    const fileRecord = await sqlite.getFile(filePath);
    if (!fileRecord || fileRecord.chunks.length === 0) {
      const content = await fs.readFile(filePath, 'utf-8');
      const embedding = await generateEmbedding(content, config);
      const points = await qdrant.searchPoints(embedding, limit + 1);
      
      return points
        .filter(point => point.payload.filePath !== filePath)
        .slice(0, limit)
        .map(point => ({
          filePath: point.payload.filePath,
          score: point.score ?? 0,
          parentDirectories: point.payload.parentDirectories
        }));
    }
    
    const firstChunkEmbedding = await generateEmbedding(fileRecord.chunks[0].content, config);
    const points = await qdrant.searchPoints(firstChunkEmbedding, limit + 1);
    
    return points
      .filter(point => point.payload.filePath !== filePath)
      .slice(0, limit)
      .map(point => ({
        filePath: point.payload.filePath,
        score: point.score ?? 0,
        parentDirectories: point.payload.parentDirectories
      }));
  } catch (error) {
    throw new SearchError(`Failed to find similar files`, error as Error);
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