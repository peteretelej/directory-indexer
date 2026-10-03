import { promises as fs } from 'fs';
import { createHash } from 'node:crypto';
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
import { loadGitignoreRules } from './gitignore.js';
import { generateEmbedding } from './embedding.js';
import { initializeStorage, ensureCollectionDimensions, FileRecord } from './storage.js';
import { log } from './logger.js';

export interface ScanOptions {
  ignorePatterns: string[];
  maxFileSize: number;
  respectGitignore: boolean;
}

export interface IndexResult {
  indexed: number;
  skipped: number;
  failed: number;
  deleted: number;
  errors: string[];
}

export interface IndexReporters {
  progress?: (message: string) => void;
  error?: (message: string) => void;
}

export class IndexingError extends Error {
  constructor(message: string, public override cause?: Error) {
    super(message);
    this.name = 'IndexingError';
  }
}

export function pointIdFor(filePath: string, chunkId: string): string {
  const hex = createHash('sha256').update(`${filePath}:${chunkId}`).digest('hex').slice(0, 32);
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20, 32)}`;
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
  const basePath = normalizePath(dirPath);
  
  // Load gitignore rules for the root directory if enabled
  const gitignoreFilter = options.respectGitignore ? await loadGitignoreRules(dirPath) : null;

  async function walkDirectory(currentPath: string): Promise<void> {
    const normalizedPath = normalizePath(currentPath);

    let realPath = normalizedPath;
    try {
      realPath = normalizePath(await fs.realpath(normalizedPath));
    } catch {
      realPath = normalizedPath;
    }

    if (visited.has(realPath)) {
      return;
    }
    visited.add(realPath);

    try {
      // Convert to relative path for gitignore matching
      const relativePath = normalizedPath.startsWith(basePath) 
        ? normalizedPath.slice(basePath.length + 1)
        : normalizedPath;
        
      if (shouldIgnoreFile(normalizedPath, relativePath, options.ignorePatterns, gitignoreFilter)) {
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

    // An exactly-unchanged mtime skips; older or newer mtimes verify via hash
    // so restored or backed-up files with stale timestamps are re-examined
    if (currentStats.mtime.getTime() === existingModTime.getTime()) {
      return false;
    }

    // If modtime suggests change, verify with hash
    const currentFileInfo = await getFileInfo(filePath);
    return currentFileInfo.hash !== existingRecord.hash;

  } catch (modtimeError) {
    // Graceful fallback: skip modtime, use hash only
    if (config.verbose) {
      log('warning', `Could not check modification time for ${filePath}`, {
        error: modtimeError instanceof Error ? modtimeError.message : String(modtimeError)
      });
    }
    try {
      const currentFileInfo = await getFileInfo(filePath);
      return currentFileInfo.hash !== existingRecord.hash;
    } catch (hashError) {
      // If we can't hash either, assume changed to be safe
      if (config.verbose) {
        log('warning', `Could not compute hash for ${filePath}`, {
          error: hashError instanceof Error ? hashError.message : String(hashError)
        });
      }
      return true;
    }
  }
}

export async function indexDirectories(paths: string[], config: Config, reporters: IndexReporters = {}): Promise<IndexResult> {
  const progress = reporters.progress ?? ((message: string) => log('info', message));
  const reportError = reporters.error ?? ((message: string) => log('error', message));
  let indexed = 0;
  let skipped = 0;
  let failed = 0;
  let deleted = 0;
  let collectionEnsured = false;
  const errors: string[] = [];

  const scanOptions: ScanOptions = {
    ignorePatterns: config.indexing.ignorePatterns,
    maxFileSize: config.indexing.maxFileSize,
    respectGitignore: config.indexing.respectGitignore
  };

  // Initialize storage
  const { sqlite, qdrant } = await initializeStorage(config);

  // First pass: scan all directories to get total file count
  let totalFiles = 0;
  for (const path of paths) {
    try {
      if (config.verbose) {
        progress(`Scanning directory: ${path}`);
      }
      const files = await scanDirectory(path, scanOptions);
      totalFiles += files.length;
      if (config.verbose) {
        progress(`Found ${files.length} files to process in ${path}`);
      }
    } catch {
      // Continue with other directories even if one fails to scan
    }
  }

  if (!config.verbose && totalFiles > 0) {
    // Add a blank line for spacing in the console output
    progress('');
    progress(`Found ${totalFiles} files to process (checking for changes...)`);
  }

  for (const path of paths) {
    try {
      // Mark directory as indexing
      const normalizedPath = normalizePath(path);
      await sqlite.upsertDirectory(normalizedPath, 'indexing');

      const files = await scanDirectory(path, scanOptions);

      // Track directory-specific counters
      const dirStartIndexed = indexed;
      const dirStartSkipped = skipped;
      let dirHadFailure = false;

      // Calculate progress interval for non-verbose updates
      const progressInterval = totalFiles > 1000 ? 50 : 10;

      for (const file of files) {
        let chunks: ChunkInfo[] = [];
        try {
          // Check if file already exists and needs reprocessing
          const existingFile = await sqlite.getFile(file.path);

          if (existingFile) {
            const hasStoredFailure =
              existingFile.errors?.some(error => error.startsWith('Failed to process')) ?? false;
            const needsReprocessing =
              hasStoredFailure || (await shouldReprocessFile(file.path, existingFile, config));
            if (!needsReprocessing) {
              skipped++;
              if (config.verbose) {
                progress(`  Skipped: ${file.path} (unchanged)`);
              }
              // Show periodic progress in non-verbose mode
              if (!config.verbose && (indexed + skipped) % progressInterval === 0) {
                progress(`Progress: ${indexed + skipped}/${totalFiles} files (${skipped} skipped as unchanged)...`);
              }
              continue; // Skip unchanged file
            }

            // File changed - clean up old vectors first
            await qdrant.deletePointsByFilePath(file.path);
          }

          const rawContent = await fs.readFile(file.path, 'utf-8');

          // Skip non-UTF-8 files (Node.js inserts U+FFFD for invalid byte sequences)
          if (rawContent.includes('\uFFFD')) {
            log('warning', 'Skipping non-UTF-8 file', { path: file.path });
            await sqlite.upsertFile(file, [], ['Skipped: file appears to be non-UTF-8 encoded']);
            skipped++;
            continue;
          }

          // Normalize CRLF to LF for consistent chunk boundaries
          const content = rawContent.replace(/\r\n/g, '\n');
          chunks = chunkText(content, config.indexing.chunkSize, config.indexing.chunkOverlap);

          // Store file metadata in SQLite
          await sqlite.upsertFile(file, chunks);

          // Generate embeddings and store in Qdrant
          for (let i = 0; i < chunks.length; i++) {
            const chunk = chunks[i];
            const embedding = await generateEmbedding(chunk.content, config);
            if (!collectionEnsured) {
              await ensureCollectionDimensions(sqlite, qdrant, config, embedding.length);
              collectionEnsured = true;
            }
            const point = {
              id: pointIdFor(file.path, chunk.id),
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
            progress(`  Indexed: ${file.path} (${chunks.length} chunks)`);
          }
          // Show periodic progress in non-verbose mode
          if (!config.verbose && (indexed + skipped) % progressInterval === 0) {
            progress(`Progress: ${indexed + skipped}/${totalFiles} files (${skipped} skipped as unchanged)...`);
          }
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : String(error);
          const causeMessage = error instanceof Error && error.cause ? `: ${(error.cause as Error).message}` : '';
          const fullError = `Failed to process ${file.path}: ${errorMessage}${causeMessage}`;
          errors.push(fullError);
          failed++;
          dirHadFailure = true;

          // Print error immediately during processing (not just in verbose mode)
          reportError(`❌ ${fullError}`);

          // Record the failure on the file row so a later run reprocesses it
          // instead of skipping it as unchanged
          await sqlite.upsertFile(file, chunks, [fullError]);
        }
      }

      // Clean up deleted files from this directory
      const indexedFiles = await sqlite.getFilesByDirectory(normalizedPath);
      const existingFilePaths = new Set(files.map(f => f.path));
      const deletedFiles = indexedFiles.filter(f => !existingFilePaths.has(f.path));

      for (const deletedFile of deletedFiles) {
        try {
          // Remove from Qdrant
          await qdrant.deletePointsByFilePath(deletedFile.path);
          
          // Remove from SQLite
          await sqlite.deleteFile(deletedFile.path);
          
          deleted++;
          if (config.verbose) {
            progress(`  Cleaned up deleted file: ${deletedFile.path}`);
          }
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : String(error);
          const fullError = `Failed to clean up deleted file ${deletedFile.path}: ${errorMessage}`;
          errors.push(fullError);
          failed++;
          dirHadFailure = true;

          reportError(`❌ ${fullError}`);
        }
      }

      // Mark directory as completed unless a file or cleanup failure occurred
      const directoryStatus = dirHadFailure ? 'failed' : 'completed';
      await sqlite.upsertDirectory(normalizedPath, directoryStatus);

      // Show directory completion
      const dirFiles = files.length;
      const dirIndexed = indexed - dirStartIndexed;
      const dirSkipped = skipped - dirStartSkipped;
      if (config.verbose) {
        progress(`Directory ${path} completed: ${dirIndexed} indexed, ${dirSkipped} skipped`);
      } else {
        progress(`Directory ${path} completed: ${dirFiles} files processed`);
      }

    } catch (error) {
      const normalizedPath = normalizePath(path);
      await sqlite.upsertDirectory(normalizedPath, 'failed');
      errors.push(`Failed to scan directory ${path}: ${(error as Error).message}`);
    }
  }

  return { indexed, skipped, failed, deleted, errors };
}