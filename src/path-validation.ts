import { realpathSync } from 'fs';
import { resolve, sep } from 'path';
import { SQLiteStorage } from './storage.js';

/**
 * Validates that a file path falls within one of the indexed directories.
 * Prevents path traversal attacks by resolving symlinks and checking prefixes.
 */
export function validatePathWithinIndexedDirs(filePath: string, indexedDirs: Set<string>): void {
  // Reject null bytes
  if (filePath.includes('\x00')) {
    throw new Error('Access denied: path contains null bytes');
  }

  // Validate Windows UNC path format if it starts with \\
  if (filePath.startsWith('\\\\')) {
    const uncParts = filePath.split('\\').filter(Boolean);
    if (uncParts.length < 2) {
      throw new Error('Invalid UNC path format: expected \\\\server\\share\\... pattern');
    }
  }

  // Resolve the path, following symlinks if the file exists
  let resolved: string;
  try {
    resolved = realpathSync(resolve(filePath));
  } catch {
    // File may not exist yet (ENOENT), fall back to resolve only
    resolved = resolve(filePath);
  }

  for (const dir of indexedDirs) {
    // Exact match (the directory itself)
    if (resolved === dir) {
      return;
    }
    // Prefix match with separator to prevent /docs-evil matching /docs
    if (resolved.startsWith(dir + sep)) {
      return;
    }
  }

  throw new Error(
    `Access denied: ${filePath} is outside indexed directories. Only files within indexed directories can be accessed.`
  );
}

/**
 * Resolves all indexed directory paths from storage, following symlinks where possible.
 * Returns a Set of resolved absolute paths.
 */
export function resolveIndexedDirectories(storage: SQLiteStorage): Set<string> {
  const dirs = storage.getDirectories();
  const resolved = new Set<string>();

  for (const dir of dirs) {
    try {
      resolved.add(realpathSync(resolve(dir)));
    } catch {
      // Directory may have been removed; keep the raw resolved path
      resolved.add(resolve(dir));
    }
  }

  return resolved;
}
