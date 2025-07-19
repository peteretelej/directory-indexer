import { createHash } from 'crypto';
import { promises as fs } from 'fs';
import { resolve, normalize, sep } from 'path';
import { createInterface } from 'readline';

export interface FileInfo {
  path: string;
  size: number;
  modifiedTime: Date;
  hash: string;
  parentDirs: string[];
}

export interface ChunkInfo {
  id: string;
  content: string;
  startIndex: number;
  endIndex: number;
}

export class FileError extends Error {
  constructor(message: string, public filePath: string, public override cause?: Error) {
    super(message);
    this.name = 'FileError';
  }
}

export function normalizePath(path: string): string {
  return normalize(resolve(path));
}

export function getParentDirectories(filePath: string): string[] {
  const normalizedPath = normalizePath(filePath);
  const parts = normalizedPath.split(sep);
  const parents: string[] = [];
  
  for (let i = 1; i < parts.length; i++) {
    parents.push(parts.slice(0, i + 1).join(sep));
  }
  
  return parents;
}

export async function getFileHash(filePath: string): Promise<string> {
  try {
    const content = await fs.readFile(filePath);
    return createHash('sha256').update(content).digest('hex');
  } catch (error) {
    throw new FileError(`Failed to hash file`, filePath, error as Error);
  }
}

export function calculateHash(content: string): string {
  return createHash('sha256').update(content).digest('hex');
}

export async function getFileInfo(filePath: string): Promise<FileInfo> {
  try {
    const stats = await fs.stat(filePath);
    const hash = await getFileHash(filePath);
    const parentDirs = getParentDirectories(filePath);
    
    return {
      path: normalizePath(filePath),
      size: stats.size,
      modifiedTime: stats.mtime,
      hash,
      parentDirs,
    };
  } catch (error) {
    throw new FileError(`Failed to get file info`, filePath, error as Error);
  }
}

export async function isDirectory(path: string): Promise<boolean> {
  try {
    const stats = await fs.stat(path);
    return stats.isDirectory();
  } catch {
    return false;
  }
}

export async function isFile(path: string): Promise<boolean> {
  try {
    const stats = await fs.stat(path);
    return stats.isFile();
  } catch {
    return false;
  }
}

export async function fileExists(path: string): Promise<boolean> {
  try {
    await fs.access(path);
    return true;
  } catch {
    return false;
  }
}

export async function ensureDirectory(dirPath: string): Promise<void> {
  try {
    await fs.mkdir(dirPath, { recursive: true });
  } catch (error) {
    throw new FileError(`Failed to create directory`, dirPath, error as Error);
  }
}

export function shouldIgnoreFile(
  filePath: string, 
  relativePath: string,
  ignorePatterns: string[],
  gitignoreFilter?: { ignores: (path: string) => boolean } | null
): boolean {
  const normalizedPath = normalizePath(filePath);
  
  // Essential patterns always take precedence
  if (ignorePatterns.some(pattern => normalizedPath.includes(pattern))) {
    return true;
  }
  
  // Check gitignore patterns using relative path
  if (gitignoreFilter && relativePath) {
    try {
      return gitignoreFilter.ignores(relativePath);
    } catch {
      // Ignore errors in gitignore matching
    }
  }
  
  return false;
}

export function isSupportedFileType(filePath: string): boolean {
  const supportedExtensions = [
    '.md', '.txt', '.rst',
    '.rs', '.py', '.js', '.ts', '.go', '.java', '.cpp', '.c',
    '.json', '.yaml', '.yml', '.toml', '.csv',
    '.env', '.conf', '.ini',
    '.html', '.xml'
  ];
  
  return supportedExtensions.some(ext => filePath.toLowerCase().endsWith(ext));
}

export async function readlineSync(prompt: string): Promise<string> {
  const rl = createInterface({
    input: process.stdin,
    output: process.stdout
  });

  return new Promise((resolve) => {
    rl.question(prompt, (answer) => {
      rl.close();
      resolve(answer);
    });
  });
}