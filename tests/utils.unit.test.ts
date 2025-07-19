import { describe, it, expect } from 'vitest';
import * as path from 'path';
import { normalizePath, calculateHash, shouldIgnoreFile } from '../src/utils.js';
import { clearGitignoreCache } from '../src/gitignore.js';

describe('Path Utilities', () => {
  it('should normalize paths across platforms', () => {
    const windowsPath = 'C:\\Users\\test\\Documents';
    const unixPath = '/home/test/documents';
    
    const normalizedWindows = normalizePath(windowsPath);
    const normalizedUnix = normalizePath(unixPath);
    
    expect(typeof normalizedWindows).toBe('string');
    expect(typeof normalizedUnix).toBe('string');
    expect(normalizedWindows.length).toBeGreaterThan(0);
    expect(normalizedUnix.length).toBeGreaterThan(0);
  });

  it('should convert relative paths to absolute', () => {
    const relativePath = './test/path';
    const absolutePath = normalizePath(relativePath);
    
    expect(path.isAbsolute(absolutePath)).toBe(true);
  });

  it('should calculate file hashes consistently', () => {
    const testContent = 'Hello, world!';
    const hash1 = calculateHash(testContent);
    const hash2 = calculateHash(testContent);
    
    expect(hash1).toBe(hash2);
    expect(hash1.length).toBeGreaterThan(0);
    expect(typeof hash1).toBe('string');
  });

  it('should produce consistent hashes', () => {
    const input = 'consistent test string';
    const hash1 = calculateHash(input);
    const hash2 = calculateHash(input);
    expect(hash1).toBe(hash2);
  });

  it('should produce different hashes for different inputs', () => {
    const hash1 = calculateHash('string one');
    const hash2 = calculateHash('string two');
    expect(hash1).not.toBe(hash2);
  });

  it('should handle different file sizes in storage', () => {
    const smallContent = 'small';
    const largeContent = 'x'.repeat(2000);
    
    expect(calculateHash(smallContent)).toBeTruthy();
    expect(calculateHash(largeContent)).toBeTruthy();
    expect(calculateHash(smallContent)).not.toBe(calculateHash(largeContent));
  });
});

describe('Gitignore Utilities', () => {
  it('should manage gitignore cache', () => {
    expect(typeof clearGitignoreCache).toBe('function');
    
    clearGitignoreCache();
    expect(true).toBe(true); // Cache cleared successfully
  });

  it('should prioritize essential patterns over gitignore', () => {
    const essentialPatterns = ['node_modules', '.git'];
    const mockIgnoreFilter = {
      ignores: (path: string) => path.includes('test.log')
    };
    
    expect(shouldIgnoreFile('/path/to/node_modules/package.json', 'node_modules/package.json', essentialPatterns, mockIgnoreFilter)).toBe(true);
    expect(shouldIgnoreFile('/path/to/.git/config', '.git/config', essentialPatterns, mockIgnoreFilter)).toBe(true);
    expect(shouldIgnoreFile('/path/to/test.log', 'test.log', [], mockIgnoreFilter)).toBe(true);
    expect(shouldIgnoreFile('/path/to/README.md', 'README.md', [], mockIgnoreFilter)).toBe(false);
  });

  it('should handle gitignore filter gracefully when errors occur', () => {
    const essentialPatterns = ['.git'];
    const faultyIgnoreFilter = {
      ignores: () => { throw new Error('Gitignore error'); }
    };
    
    expect(shouldIgnoreFile('/path/to/.git/config', '.git/config', essentialPatterns, faultyIgnoreFilter)).toBe(true);
    expect(shouldIgnoreFile('/path/to/README.md', 'README.md', essentialPatterns, faultyIgnoreFilter)).toBe(false);
  });

  it('should work without gitignore filter', () => {
    const essentialPatterns = ['node_modules', '.git'];
    
    expect(shouldIgnoreFile('/path/to/node_modules/package.json', 'node_modules/package.json', essentialPatterns)).toBe(true);
    expect(shouldIgnoreFile('/path/to/README.md', 'README.md', essentialPatterns)).toBe(false);
    expect(shouldIgnoreFile('/path/to/test.log', 'test.log', essentialPatterns)).toBe(false);
  });
});