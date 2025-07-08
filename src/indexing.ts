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
import { generateEmbedding } from './embedding.js';
import { initializeStorage, FileRecord } from './storage.js';

export interface ScanOptions {
  ignorePatterns: string[];
  maxFileSize: number;
}

export interface IndexResult {
  indexed: number;
  skipped: number;
  failed: number;
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

async function shouldReprocessFile(filePath: string, existingRecord: FileRecord, config: Config): Promise<boolean> {
  try {
    const fs = await import('fs/promises');
    
    // Try modtime check first (fast path)
    const currentStats = await fs.stat(filePath);
    const existingModTime = new Date(existingRecord.modifiedTime);
    
    // If modtime is clearly older, likely unchanged
    if (currentStats.mtime <= existingModTime) {
      return false; // Skip processing
    }
    
    // If modtime suggests change, verify with hash
    const currentFileInfo = await getFileInfo(filePath);
    return currentFileInfo.hash !== existingRecord.hash;
    
  } catch (modtimeError) {
    // Graceful fallback: skip modtime, use hash only
    if (config.verbose) {
      console.log(`Warning: Could not check modification time for ${filePath}:`, modtimeError);
    }
    try {
      const currentFileInfo = await getFileInfo(filePath);
      return currentFileInfo.hash !== existingRecord.hash;
    } catch (hashError) {
      // If we can't hash either, assume changed to be safe
      if (config.verbose) {
        console.log(`Warning: Could not compute hash for ${filePath}:`, hashError);
      }
      return true;
    }
  }
}

export async function indexDirectories(paths: string[], config: Config): Promise<IndexResult> {
  let indexed = 0;
  let skipped = 0;
  let failed = 0;
  const errors: string[] = [];
  
  const scanOptions: ScanOptions = {
    ignorePatterns: config.indexing.ignorePatterns,
    maxFileSize: config.indexing.maxFileSize
  };
  
  // Initialize storage
  const { sqlite, qdrant } = await initializeStorage(config);
  
  // First pass: scan all directories to get total file count
  let totalFiles = 0;
  for (const path of paths) {
    try {
      if (config.verbose) {
        console.log(`Scanning directory: ${path}`);
      }
      const files = await scanDirectory(path, scanOptions);
      totalFiles += files.length;
      if (config.verbose) {
        console.log(`Found ${files.length} files to process in ${path}`);
      }
    } catch {
      // Continue with other directories even if one fails to scan
    }
  }
  
  if (!config.verbose && totalFiles > 0) {
    console.log(`Processing ${totalFiles} files...`);
  }
  
  for (const path of paths) {
    try {
      // Mark directory as indexing
      const normalizedPath = normalizePath(path);
      await sqlite.upsertDirectory(normalizedPath, 'indexing');
      
      const files = await scanDirectory(path, scanOptions);
      
      for (const file of files) {
        try {
          // Check if file already exists and needs reprocessing
          const existingFile = await sqlite.getFile(file.path);
          
          if (existingFile) {
            const needsReprocessing = await shouldReprocessFile(file.path, existingFile, config);
            if (!needsReprocessing) {
              skipped++;
              continue; // Skip unchanged file
            }
            
            // File changed - clean up old vectors first
            await qdrant.deletePointsByFileHash(existingFile.hash);
          }
          
          const content = await fs.readFile(file.path, 'utf-8');
          const chunks = chunkText(content, config.indexing.chunkSize, config.indexing.chunkOverlap);
          
          // Store file metadata in SQLite
          await sqlite.upsertFile(file, chunks);
          
          // Generate embeddings and store in Qdrant
          for (let i = 0; i < chunks.length; i++) {
            const chunk = chunks[i];
            const embedding = await generateEmbedding(chunk.content, config);
            // Generate a unique integer ID by combining hash and chunk index
            const hashNum = parseInt(file.hash.slice(0, 8), 16);
            const pointId = (hashNum % 1000000) * 1000 + parseInt(chunk.id);
            const point = {
              id: pointId,
              vector: embedding,
              payload: {
                filePath: file.path,
                chunkId: chunk.id,
                fileHash: file.hash,
                content: chunk.content,
                parentDirectories: file.parentDirs
              }
            };
            await qdrant.upsertPoints([point]);
          }
          
          indexed++;
          if (config.verbose) {
            console.log(`  Indexed: ${file.path} (${chunks.length} chunks)`);
          }
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : String(error);
          const causeMessage = error instanceof Error && error.cause ? `: ${(error.cause as Error).message}` : '';
          const fullError = `Failed to process ${file.path}: ${errorMessage}${causeMessage}`;
          errors.push(fullError);
          failed++;
          
          // Print error immediately during processing (not just in verbose mode)
          console.error(`❌ ${fullError}`);
        }
      }
      
      // Mark directory as completed if no errors for this directory
      const directoryErrors = errors.filter(err => err.includes(path));
      const directoryStatus = directoryErrors.length > 0 ? 'failed' : 'completed';
      await sqlite.upsertDirectory(normalizedPath, directoryStatus);
      
    } catch (error) {
      const normalizedPath = normalizePath(path);
      await sqlite.upsertDirectory(normalizedPath, 'failed');
      errors.push(`Failed to scan directory ${path}: ${(error as Error).message}`);
    }
  }
  
  return { indexed, skipped, failed, errors };
}