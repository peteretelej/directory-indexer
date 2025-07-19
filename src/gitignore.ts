import { promises as fs } from 'fs';
import { join } from 'path';
import ignore from 'ignore';
import { normalizePath, isFile } from './utils.js';

export interface GitignoreCache {
  [directory: string]: ReturnType<typeof ignore> | null;
}

const gitignoreCache: GitignoreCache = {};

export async function findGitignoreFiles(directory: string): Promise<string[]> {
  const gitignoreFiles: string[] = [];
  const normalizedDir = normalizePath(directory);
  
  try {
    const gitignorePath = join(normalizedDir, '.gitignore');
    if (await isFile(gitignorePath)) {
      gitignoreFiles.push(gitignorePath);
    }
  } catch {
    // Ignore errors when checking for .gitignore files
  }
  
  return gitignoreFiles;
}

export async function loadGitignoreRules(directory: string): Promise<ReturnType<typeof ignore> | null> {
  const normalizedDir = normalizePath(directory);
  
  // Return cached result if available
  if (normalizedDir in gitignoreCache) {
    return gitignoreCache[normalizedDir];
  }
  
  try {
    const gitignoreFiles = await findGitignoreFiles(normalizedDir);
    
    if (gitignoreFiles.length === 0) {
      gitignoreCache[normalizedDir] = null;
      return null;
    }
    
    const ig = ignore();
    
    for (const gitignoreFile of gitignoreFiles) {
      try {
        const content = await fs.readFile(gitignoreFile, 'utf-8');
        ig.add(content);
      } catch {
        // Continue if we can't read a specific .gitignore file
      }
    }
    
    gitignoreCache[normalizedDir] = ig;
    return ig;
  } catch {
    gitignoreCache[normalizedDir] = null;
    return null;
  }
}


export function clearGitignoreCache(): void {
  for (const key in gitignoreCache) {
    delete gitignoreCache[key];
  }
}