# Bug: Orphaned Data from Deleted Files

## Issue Summary
When files are deleted from the filesystem, their corresponding vectors remain in Qdrant and metadata remains in SQLite. This causes indefinite data accumulation and search inconsistencies.

## Current System Behavior

### Indexing Flow (`src/indexing.ts`)
```typescript
export async function indexDirectories(paths: string[], config: Config): Promise<IndexResult> {
  // ...
  for (const path of paths) {
    const files = await scanDirectory(path, scanOptions);  // Only finds existing files
    
    for (const file of files) {
      // Process existing files: add new, update changed, skip unchanged
      const existingFile = await sqlite.getFile(file.path);
      if (existingFile) {
        const needsReprocessing = await shouldReprocessFile(file.path, existingFile, config);
        if (!needsReprocessing) {
          skipped++;
          continue; // Skip unchanged file
        }
        // File changed - clean up old vectors first
        await qdrant.deletePointsByFilePath(file.path);
      }
      // ... index the file
    }
    // Directory marked as completed
  }
}
```

### The Gap
The indexing process only handles **existing files**:
- ✅ **New files:** Added to SQLite + Qdrant
- ✅ **Changed files:** Updated in SQLite + Qdrant (old vectors cleaned up)
- ✅ **Unchanged files:** Skipped
- ❌ **Deleted files:** No detection or cleanup logic

## Problem Demonstration

### Scenario
```bash
# Initial indexing
$ indexer index /project/src
# Filesystem: [file1.js, file2.js, file3.js]
# SQLite: 3 file records
# Qdrant: 15 vectors (5 chunks each)

# User deletes a file
$ rm /project/src/file2.js

# Re-indexing same directory
$ indexer index /project/src  
# Filesystem: [file1.js, file3.js] (file2.js gone)
# scanDirectory() only finds: [file1.js, file3.js]
# Processing: file1.js (unchanged, skipped), file3.js (unchanged, skipped)
# SQLite: Still 3 file records (file2.js record remains)
# Qdrant: Still 15 vectors (file2.js vectors remain)
```

### Impact
- **Data Growth:** SQLite and Qdrant grow indefinitely with orphaned records
- **Search Pollution:** Results may include content from deleted files
- **Inconsistent Counts:** System status shows mismatched vector/chunk counts
- **Storage Waste:** Unused vector embeddings consume memory
- **Workspace Health:** Incorrect file counts in workspace reporting

## Code Context

### Relevant Functions
- **`scanDirectory()`** (`src/utils.ts`): Only returns existing filesystem files
- **`sqlite.getFilesByDirectory()`** (`src/storage.ts`): Returns all indexed files for a directory
- **`qdrant.deletePointsByFilePath()`** (`src/storage.ts`): Can remove vectors by file path

### Current Cleanup Mechanisms
- ✅ **File changes:** `deletePointsByFilePath()` called before re-indexing
- ✅ **Manual reset:** User can delete database/collection and re-index
- ❌ **File deletion:** No automatic detection or cleanup

## Possible Solution Approaches

### 1. Incremental Cleanup During Indexing
Compare current filesystem scan with SQLite records to identify and remove deleted files.

### 2. Reset Flag for Full Re-indexing  
Add `--reset` flag to wipe all data for a directory and rebuild from scratch.

### 3. Garbage Collection Process
Separate background process to periodically identify and clean orphaned data.

### 4. File Watching
Monitor filesystem changes to detect deletions in real-time.

## Current Workaround
```bash
# Manual full reset
rm ~/.directory-indexer/data.db
curl -X DELETE http://localhost:6333/collections/directory-indexer  
indexer index /path/to/directory
```

## Priority
**High** - Causes progressive data accumulation and functional inconsistencies.

## Status  
**Open** - Requires design decision on cleanup approach.