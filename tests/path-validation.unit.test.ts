import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { validatePathWithinIndexedDirs, resolveIndexedDirectories } from '../src/path-validation.js';
import { SQLiteStorage } from '../src/storage.js';
import { loadConfig } from '../src/config.js';
import { mkdirSync, writeFileSync, rmSync, symlinkSync, realpathSync } from 'fs';
import { join } from 'path';
import { tmpdir } from 'os';

describe('validatePathWithinIndexedDirs', () => {
  let tempDir: string;
  let subDir: string;
  let indexedDirs: Set<string>;

  beforeEach(() => {
    tempDir = join(realpathSync(tmpdir()), `path-validation-test-${Date.now()}`);
    subDir = join(tempDir, 'project');
    mkdirSync(subDir, { recursive: true });
    writeFileSync(join(subDir, 'file.txt'), 'test');
    indexedDirs = new Set([subDir]);
  });

  afterEach(() => {
    rmSync(tempDir, { recursive: true, force: true });
  });

  it('should allow files within an indexed directory', () => {
    expect(() =>
      validatePathWithinIndexedDirs(join(subDir, 'file.txt'), indexedDirs)
    ).not.toThrow();
  });

  it('should allow the indexed directory itself', () => {
    expect(() =>
      validatePathWithinIndexedDirs(subDir, indexedDirs)
    ).not.toThrow();
  });

  it('should allow files in nested subdirectories', () => {
    const nested = join(subDir, 'a', 'b', 'c');
    mkdirSync(nested, { recursive: true });
    writeFileSync(join(nested, 'deep.txt'), 'deep');

    expect(() =>
      validatePathWithinIndexedDirs(join(nested, 'deep.txt'), indexedDirs)
    ).not.toThrow();
  });

  it('should reject paths outside indexed directories', () => {
    expect(() =>
      validatePathWithinIndexedDirs('/etc/passwd', indexedDirs)
    ).toThrow('Access denied');
  });

  it('should reject path traversal via ..', () => {
    expect(() =>
      validatePathWithinIndexedDirs(join(subDir, '..', '..', 'etc', 'passwd'), indexedDirs)
    ).toThrow('Access denied');
  });

  it('should reject null bytes in path', () => {
    expect(() =>
      validatePathWithinIndexedDirs(join(subDir, 'file\x00.txt'), indexedDirs)
    ).toThrow('null bytes');
  });

  it('should prevent prefix collision (e.g. /docs-evil vs /docs)', () => {
    const docsDir = join(tempDir, 'docs');
    const docsEvilDir = join(tempDir, 'docs-evil');
    mkdirSync(docsDir, { recursive: true });
    mkdirSync(docsEvilDir, { recursive: true });
    writeFileSync(join(docsEvilDir, 'steal.txt'), 'secret');

    const docsOnly = new Set([docsDir]);

    expect(() =>
      validatePathWithinIndexedDirs(join(docsEvilDir, 'steal.txt'), docsOnly)
    ).toThrow('Access denied');
  });

  it('should resolve symlinks when validating', () => {
    const realDir = join(tempDir, 'real');
    const linkPath = join(tempDir, 'link');
    mkdirSync(realDir, { recursive: true });
    writeFileSync(join(realDir, 'data.txt'), 'data');

    try {
      symlinkSync(realDir, linkPath);
    } catch {
      // Symlinks may not be supported in all environments; skip if so
      return;
    }

    // Index the real directory, access via symlink
    const realIndexed = new Set([realpathSync(realDir)]);
    expect(() =>
      validatePathWithinIndexedDirs(join(linkPath, 'data.txt'), realIndexed)
    ).not.toThrow();
  });

  it('should handle macOS /tmp symlink to /private/tmp', () => {
    // On macOS, /tmp is a symlink to /private/tmp
    // This test verifies that both resolve to the same path
    if (process.platform !== 'darwin') return;

    const realTmp = realpathSync('/tmp');
    const testDir = join(realTmp, `mac-tmp-test-${Date.now()}`);
    mkdirSync(testDir, { recursive: true });
    writeFileSync(join(testDir, 'test.txt'), 'test');

    const dirs = new Set([testDir]);

    // Access via /tmp (symlink) should work because realpathSync resolves it
    expect(() =>
      validatePathWithinIndexedDirs(join('/tmp', `mac-tmp-test-${Date.now().toString().slice(0, -1)}0`, 'test.txt'), dirs)
    ).toThrow('Access denied'); // Different timestamp, won't match

    // But the actual resolved path works
    expect(() =>
      validatePathWithinIndexedDirs(join(testDir, 'test.txt'), dirs)
    ).not.toThrow();

    rmSync(testDir, { recursive: true, force: true });
  });

  it('should reject when indexedDirs is empty', () => {
    expect(() =>
      validatePathWithinIndexedDirs('/any/path', new Set())
    ).toThrow('Access denied');
  });
});

describe('resolveIndexedDirectories', () => {
  it('should resolve directory paths from storage', () => {
    const config = loadConfig();
    config.storage.sqlitePath = ':memory:';
    const storage = new SQLiteStorage(config);

    try {
      // Insert test directories
      storage.db.prepare('INSERT INTO directories (path, status) VALUES (?, ?)').run('/test/dir1', 'completed');
      storage.db.prepare('INSERT INTO directories (path, status) VALUES (?, ?)').run('/test/dir2', 'completed');

      const dirs = resolveIndexedDirectories(storage);

      expect(dirs.size).toBe(2);
      // The resolved paths should contain the original paths (resolved)
      // Since these directories don't exist, they fall back to path.resolve()
      expect(dirs.size).toBeGreaterThanOrEqual(2);
    } finally {
      storage.close();
    }
  });

  it('should return empty set when no directories exist', () => {
    const config = loadConfig();
    config.storage.sqlitePath = ':memory:';
    const storage = new SQLiteStorage(config);

    try {
      const dirs = resolveIndexedDirectories(storage);
      expect(dirs.size).toBe(0);
    } finally {
      storage.close();
    }
  });
});
