# Reset Command Implementation Plan

## Overview

Add `reset` command to completely clear directory-indexer's stored data, allowing users to start fresh. Uses custom configuration paths and collection names when specified.

## CLI Interface

```bash
npx directory-indexer reset [options]
```

**Options:**
- `--force` - Skip confirmation prompt
- `-v, --verbose` - Show detailed operations

## Configuration Awareness

The reset command must respect all custom configuration:

- **Custom data directory**: `DIRECTORY_INDEXER_DATA_DIR`
- **Custom SQLite path**: Derived from data directory
- **Custom Qdrant endpoint**: `QDRANT_ENDPOINT` 
- **Custom collection name**: `DIRECTORY_INDEXER_QDRANT_COLLECTION`
- **Qdrant API key**: `QDRANT_API_KEY` (for authentication)

## Interactive Flow

```bash
$ npx directory-indexer reset

The following directory-indexer data will be reset:
  • SQLite database: /custom/path/database.db (1.2 MB)
  • Qdrant collection: my-custom-collection (3,891 vectors)
  • Qdrant endpoint: http://remote-qdrant:6333

Your original files will not be touched. Continue? (y/N): 
```

## Implementation Steps

### 1. Add CLI Command

**File**: `src/cli.ts`

Add reset subcommand to commander.js:

```typescript
program
  .command('reset')
  .description('Reset directory-indexer data (database and vector collection)')
  .option('--force', 'Skip confirmation prompt')
  .option('-v, --verbose', 'Show detailed operations')
  .action(async (options) => {
    await handleReset(options);
  });
```

### 2. Reset Implementation

**File**: `src/reset.ts` (new)

```typescript
export async function resetEnvironment(config: Config, options: ResetOptions): Promise<void> {
  const stats = await gatherResetStats(config);
  
  if (!options.force) {
    await showConfirmation(stats, config);
  }
  
  await performReset(config, options);
}

async function gatherResetStats(config: Config): Promise<ResetStats> {
  // Check SQLite file size
  // Query Qdrant collection info
  // Return stats for display
}

async function showConfirmation(stats: ResetStats, config: Config): Promise<void> {
  // Display what will be reset with custom paths
  // Prompt for confirmation
  // Throw error if user cancels
}

async function performReset(config: Config, options: ResetOptions): Promise<void> {
  // Delete SQLite database file
  // Delete Qdrant collection
  // Show success message
}
```

### 3. Reset Operations

**SQLite Reset:**
- Delete the database file at `config.storage.sqlitePath`
- Handle file not found gracefully

**Qdrant Reset:**
- Delete collection at `config.storage.qdrantCollection`
- Use `config.storage.qdrantEndpoint` and API key
- Handle collection not found gracefully

### 4. Error Handling

**Service Unavailable:**
- Qdrant unreachable: Warn but continue with SQLite cleanup
- SQLite file locked: Clear error message

**Partial Failures:**
- Track what was successfully reset
- Report partial success clearly

**Network Issues:**
- Timeout handling for Qdrant operations
- Retry logic for transient failures

### 5. Verbose Output

When `--verbose` specified:

```bash
$ npx directory-indexer reset --verbose --force

Checking SQLite database...
  ✓ Found: /custom/path/database.db (1.2 MB)
  
Checking Qdrant collection...  
  ✓ Connected to: http://remote-qdrant:6333
  ✓ Found collection: my-custom-collection (3,891 vectors)

Resetting data...
  ✓ Deleted SQLite database
  ✓ Deleted Qdrant collection
  
Reset complete. Directory-indexer is ready for fresh indexing.
```

### 6. Integration Points

**Config Loading:**
- Reuse existing `loadConfig()` from `config.ts`
- Respect all environment variables and CLI args

**Storage Operations:**
- Reuse Qdrant client from `storage.ts`  
- Use same connection logic as other commands

**Error Types:**
- Use existing `AppError`, `StorageError` classes
- Add `ResetError` if needed

### 7. API Documentation

**Update**: `docs/API.md`

Add reset command documentation with examples showing custom configuration usage.

## Testing

**Integration Test:**
- Test with default configuration
- Test with custom data directory and collection
- Test with Qdrant unavailable
- Test confirmation flow and force flag

**Edge Cases:**
- Empty database/collection
- Permission issues
- Network timeouts

## Success Criteria

- ✅ Respects all custom configuration paths
- ✅ Safe confirmation flow with clear preview
- ✅ Graceful handling of missing resources
- ✅ Consistent with existing CLI patterns
- ✅ Clear success/error messaging