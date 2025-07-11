# Plan: Deleted Files Cleanup Implementation

## Overview

Implement automatic cleanup of deleted files from both SQLite metadata and Qdrant vectors during directory re-indexing. Currently, when files are deleted from the filesystem, their data remains in the storage systems causing data accumulation and search inconsistencies.

## Problem Analysis

### Current Indexing Flow Gap

**Location**: `src/indexing.ts:267` - TODO comment already exists

**Current behavior**:
```typescript
const files = await scanDirectory(path, scanOptions);  // Only existing files
for (const file of files) {
  // Process existing files only - no comparison with previously indexed
}
// TODO: Clean up deleted files from this directory
```

**Missing logic**: Compare filesystem scan results with SQLite records to identify deleted files.

### Available Infrastructure

**SQLite operations**:
- `getFilesByDirectory(directoryPath)` - get all indexed files for directory (`storage.ts:374`)
- `deleteFile(path)` - remove file record (`storage.ts:365`)

**Qdrant operations**:
- `deletePointsByFilePath(filePath)` - remove file vectors (`storage.ts:158`)

## Solution Design

### Approach: Incremental Cleanup During Indexing

Add deleted file detection to the existing indexing workflow in `indexDirectories()` function.

#### Implementation Location
- **File**: `src/indexing.ts`
- **Function**: `indexDirectories()` 
- **Line**: After line 266 (current file processing loop)

#### Logic Flow
```typescript
// After processing existing files, detect and clean up deleted files
const existingFiles = await scanDirectory(path, scanOptions);
const indexedFiles = await sqlite.getFilesByDirectory(normalizedPath);

const existingFilePaths = new Set(existingFiles.map(f => f.path));
const deletedFiles = indexedFiles.filter(f => !existingFilePaths.has(f.path));

for (const deletedFile of deletedFiles) {
  // Remove from Qdrant
  await qdrant.deletePointsByFilePath(deletedFile.path);
  
  // Remove from SQLite
  await sqlite.deleteFile(deletedFile.path);
  
  if (config.verbose) {
    console.log(`  Cleaned up deleted file: ${deletedFile.path}`);
  }
}
```

### Return Value Enhancement

Extend `IndexResult` interface to include cleanup statistics:

```typescript
export interface IndexResult {
  indexed: number;
  skipped: number;
  failed: number;
  deleted: number;  // NEW: number of deleted files cleaned up
  errors: string[];
}
```

## Implementation Phases

### Phase 1: Core Implementation
- **File**: `src/indexing.ts`
- **Changes**:
  - Add `deleted` field to `IndexResult` interface (line 22-27)
  - Add cleanup logic after existing file processing (after line 266)
  - Update return statement to include `deleted` count

### Phase 2: Integration Test
- **File**: `tests/integration.test.ts`
- **Test name**: `should clean up deleted files during re-indexing`
- **Test flow**:
  1. Create temporary test directory with files
  2. Index the directory
  3. Verify files are indexed (search/status)
  4. Delete one file from filesystem
  5. Re-index the same directory
  6. Verify file is no longer in search results
  7. Verify database counts are reduced
  8. Verify status report reflects cleanup

### Phase 3: Error Handling & Output Enhancement
- Continue processing if individual file cleanup fails
- Log cleanup errors but don't fail entire indexing
- Add cleanup errors to `IndexResult.errors`
- Update CLI output to include cleanup information:
  ```
  Indexed 5 files, skipped 3 files, cleaned up 2 deleted files, 0 errors
  ```

## Test Design

### Integration Test Implementation

```typescript
it('should clean up deleted files during re-indexing', async () => {
  const tempDir = await createTempTestDirectory();
  const testFile = join(tempDir, 'test-file.md');
  
  try {
    // 1. Create and index a test file
    await fs.writeFile(testFile, '# Test Content\nThis is test content for deletion.');
    const indexResult1 = await indexDirectories([tempDir], config);
    expect(indexResult1.indexed).toBe(1);
    
    // 2. Verify file is searchable
    const searchResults1 = await searchContent('test content', { limit: 10 });
    const foundFile = searchResults1.find(r => r.filePath === testFile);
    expect(foundFile).toBeDefined();
    
    // 3. Delete file from filesystem
    await fs.unlink(testFile);
    expect(await fileExists(testFile)).toBe(false);
    
    // 4. Re-index directory
    const indexResult2 = await indexDirectories([tempDir], config);
    expect(indexResult2.deleted).toBe(1);
    expect(indexResult2.indexed).toBe(0);
    
    // 5. Verify file is no longer searchable
    const searchResults2 = await searchContent('test content', { limit: 10 });
    const foundFile2 = searchResults2.find(r => r.filePath === testFile);
    expect(foundFile2).toBeUndefined();
    
    // 6. Verify database cleanup
    const status = await getIndexStatus();
    expect(status.filesIndexed).toBe(0); // Assuming temp dir was only directory
    
  } finally {
    await cleanupTempDirectory(tempDir);
  }
});
```

### Test Utilities Needed

```typescript
async function createTempTestDirectory(): Promise<string> {
  const tempDir = join(process.cwd(), 'tests', 'temp-' + Date.now());
  await fs.mkdir(tempDir, { recursive: true });
  return tempDir;
}

async function cleanupTempDirectory(dir: string): Promise<void> {
  await fs.rm(dir, { recursive: true, force: true });
}
```

## Edge Cases & Considerations

### File Path Matching
- Ensure path normalization consistency between scan and database lookup
- Handle case sensitivity differences across platforms
- Verify symlink behavior

### Performance Considerations
- Cleanup runs per directory, not globally
- Use Set for O(1) path lookups
- Consider batch deletion for large numbers of deleted files

### Error Recovery
- If Qdrant cleanup fails but SQLite succeeds, log warning
- If SQLite cleanup fails, don't retry Qdrant deletion
- Partial failures should not block processing of other files

### Concurrent Access
- No special handling needed - SQLite handles file-level locking
- Qdrant operations are atomic at collection level

## Validation Criteria

### Success Metrics
- Integration test passes: deleted files are cleaned up during re-indexing
- Search results exclude deleted files after re-indexing
- Database size decreases after file deletion and re-indexing
- Status reports accurate file counts
- No performance regression in indexing speed

### Manual Testing
1. Index a directory with known files
2. Delete several files from filesystem
3. Re-index the directory
4. Verify search no longer returns deleted files
5. Check status command shows reduced counts

## Files to Modify

### Primary Changes
- `src/indexing.ts` - Core cleanup implementation
- `tests/integration.test.ts` - Integration test

### Documentation Updates
- `docs/API.md` - Update CLI output examples
- `docs/design.md` - Update data flow documentation

---

**Status**: Ready for implementation
**Priority**: High - fixes progressive data accumulation bug