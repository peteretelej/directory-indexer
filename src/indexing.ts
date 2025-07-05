import { promises as fs } from 'fs';
import { join } from 'path';
import { Config } from './config.js';
import { 
  FileInfo, 
  ChunkInfo, 
  normalizePath, 
  getFileInfo, 
  shouldIgnoreFile, 
  isSupportedFileType,
  isDirectory,
  isFile
} from './utils.js';

export interface ScanOptions {
  ignorePatterns: string[];
  maxFileSize: number;
}

export interface IndexResult {
  indexed: number;
  skipped: number;
  errors: string[];
}

export class IndexingError extends Error {
  constructor(message: string, public override cause?: Error) {
    super(message);
    this.name = 'IndexingError';
  }
}

export function chunkText(content: string, chunkSize: number, overlap: number): ChunkInfo[] {
  if (content.length <= chunkSize) {
    return [{
      id: '0',
      content,
      startIndex: 0,
      endIndex: content.length
    }];
  }
  
  const chunks: ChunkInfo[] = [];
  let startIndex = 0;
  let chunkId = 0;
  
  while (startIndex < content.length) {
    const endIndex = Math.min(startIndex + chunkSize, content.length);
    const chunkContent = content.slice(startIndex, endIndex);
    
    chunks.push({
      id: chunkId.toString(),
      content: chunkContent,
      startIndex,
      endIndex
    });
    
    chunkId++;
    const nextStart = endIndex - overlap;
    
    if (nextStart <= startIndex) {
      startIndex = startIndex + Math.max(1, chunkSize - overlap);
    } else {
      startIndex = nextStart;
    }
    
    if (startIndex >= content.length) break;
  }
  
  return chunks;
}

export async function scanDirectory(dirPath: string, options: ScanOptions): Promise<FileInfo[]> {
  const files: FileInfo[] = [];
  const visited = new Set<string>();
  
  async function walkDirectory(currentPath: string): Promise<void> {
    const normalizedPath = normalizePath(currentPath);
    
    if (visited.has(normalizedPath)) {
      return;
    }
    visited.add(normalizedPath);
    
    try {
      if (shouldIgnoreFile(normalizedPath, options.ignorePatterns)) {
        return;
      }
      
      if (await isDirectory(normalizedPath)) {
        const entries = await fs.readdir(normalizedPath);
        
        for (const entry of entries) {
          const fullPath = join(normalizedPath, entry);
          await walkDirectory(fullPath);
        }
      } else if (await isFile(normalizedPath)) {
        if (!isSupportedFileType(normalizedPath)) {
          return;
        }
        
        const stats = await fs.stat(normalizedPath);
        if (stats.size > options.maxFileSize) {
          return;
        }
        
        const fileInfo = await getFileInfo(normalizedPath);
        files.push(fileInfo);
      }
    } catch (error) {
      throw new IndexingError(`Failed to scan directory: ${normalizedPath}`, error as Error);
    }
  }
  
  await walkDirectory(dirPath);
  return files;
}

export async function getFileMetadata(filePath: string): Promise<FileInfo> {
  try {
    return await getFileInfo(filePath);
  } catch (error) {
    throw new IndexingError(`Failed to get file metadata`, error as Error);
  }
}

export async function indexDirectories(paths: string[], config: Config): Promise<IndexResult> {
  let indexed = 0;
  let skipped = 0;
  const errors: string[] = [];
  
  const scanOptions: ScanOptions = {
    ignorePatterns: config.indexing.ignorePatterns,
    maxFileSize: config.indexing.maxFileSize
  };
  
  for (const path of paths) {
    try {
      const files = await scanDirectory(path, scanOptions);
      
      for (const file of files) {
        try {
          const content = await fs.readFile(file.path, 'utf-8');
          chunkText(content, config.indexing.chunkSize, config.indexing.chunkOverlap);
          
          indexed++;
        } catch (error) {
          skipped++;
          errors.push(`Failed to process ${file.path}: ${(error as Error).message}`);
        }
      }
    } catch (error) {
      errors.push(`Failed to scan directory ${path}: ${(error as Error).message}`);
    }
  }
  
  return { indexed, skipped, errors };
}