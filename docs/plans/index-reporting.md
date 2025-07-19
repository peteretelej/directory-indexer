# Index Reporting Implementation Plan

## Problem
Large directory indexing lacks adequate progress feedback. Users experience long periods of silence between "Processing X files..." and completion summary.

## Solution Overview
Improve existing verbose/non-verbose reporting modes with better user guidance and periodic progress updates.

## Implementation Changes

### 1. Enhanced Non-Verbose Mode (Default)

**Location**: `src/cli.ts:38-39` (after indexing starts)

Add initial user guidance:
```typescript
if (!options.verbose) {
  console.log('Run with --verbose for detailed per-file indexing reports');
  console.log('Indexing can be safely stopped and resumed - progress is automatically saved');
  console.log('You can start using the MCP server while indexing continues');
}
```

**Location**: `src/indexing.ts:210-267` (file processing loop)

Add periodic progress updates:
```typescript
const progressInterval = Math.max(10, Math.floor(totalFiles / 20));
if (!config.verbose && (indexed + skipped) % progressInterval === 0) {
  console.log(`Progress: ${indexed + skipped}/${totalFiles} files (${skipped} skipped as unchanged)...`);
}
```

### 2. Directory Completion Reporting

**Location**: `src/indexing.ts:296-299` (after each directory)

Add completion summary for both modes:
```typescript
if (config.verbose) {
  console.log(`Directory ${path} completed: ${dirIndexed} indexed, ${dirSkipped} skipped`);
} else {
  console.log(`Directory ${path} completed: ${dirTotal} files processed`);
}
```

### 3. Improved Skip Messaging

**Location**: `src/indexing.ts:254-256` (verbose mode file skipping)

Clarify skip reasons:
```typescript
if (config.verbose) {
  console.log(`  Skipped: ${file.path} (unchanged)`);
}
```

## Expected Output

### Non-Verbose Mode (Default)
```
directory-indexer indexing 1 directory: ./project-files
Run with --verbose for detailed per-file indexing reports
Indexing can be safely stopped and resumed - progress is automatically saved
You can start using the MCP server while indexing continues

Found 1,250 files to process (checking for changes...)
Progress: 50/1,250 files (23 skipped as unchanged)...
Progress: 100/1,250 files (67 skipped as unchanged)...
Directory ./project-files completed: 1,250 files processed
Indexed 183 files, skipped 1,067 unchanged files, 0 failed
```

### Verbose Mode
```
directory-indexer indexing 1 directory: ./project-files
Run with --verbose for detailed per-file indexing reports
Indexing can be safely stopped and resumed - progress is automatically saved
You can start using the MCP server while indexing continues

Scanning directory: ./project-files
Found 1,250 files to process in ./project-files
Progress: 50/1,250 files (23 skipped as unchanged)...
  Skipped: /path/to/unchanged-file.md (unchanged)
  Indexed: /path/to/modified-file.md (45 chunks)
  Indexed: /path/to/new-file.md (12 chunks)
Progress: 100/1,250 files (67 skipped as unchanged)...
Directory ./project-files completed: 183 indexed, 1,067 skipped
Indexed 183 files, skipped 1,067 unchanged files, 0 failed
```

## Files to Modify

1. `src/cli.ts` - Add initial user guidance messages
2. `src/indexing.ts` - Add progress updates and directory completion reporting

## Technical Notes

- No progress bars (avoid terminal compatibility issues)
- No new dependencies required
- Leverages existing progress counters
- Maintains backward compatibility
- Simple implementation with minimal code changes

## Testing

Add integration tests to `tests/integration/cli.test.ts`:

```typescript
it('should show progress messages during indexing', async () => {
  const indexResult = await runCLIWithLogging(['index', testDataPath], testEnv.env);
  expect(indexResult.stdout).toMatch(/Processing \d+ files\.\.\./);
  expect(indexResult.stdout).toMatch(/Indexed \d+ files, skipped \d+ files/);
});

it('should show detailed progress in verbose mode', async () => {
  const indexResult = await runCLIWithLogging(['index', testDataPath, '--verbose'], testEnv.env);
  expect(indexResult.stdout).toMatch(/Indexed: .*\.md \(\d+ chunks\)/);
  expect(indexResult.stdout).toMatch(/Found \d+ files to process in/);
});
```

## Benefits

- Better user experience for large directory operations
- Clear guidance on available options and behavior
- Periodic feedback prevents "hanging" concerns
- Educates users about resume capability and concurrent MCP usage