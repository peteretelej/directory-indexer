# Bug: No .gitignore File Support

## Issue Reference
- GitHub Issue: [#10 - Expand ignore list](https://github.com/peteretelej/directory-indexer/issues/10)
- Priority: Medium
- Status: Open

## Current State

The directory indexer currently uses hardcoded ignore patterns defined in the configuration:

**File:** `src/config.ts:125`
```typescript
ignorePatterns: ['.git', 'node_modules', 'target', '.DS_Store'],
```

These patterns are applied globally during directory scanning to skip specific directories and files. The current implementation:

1. **Hardcoded patterns only** - No support for user-defined ignore patterns
2. **No .gitignore parsing** - Existing .gitignore files in directories are completely ignored
3. **Limited flexibility** - Users cannot customize ignore patterns per project or directory

## Problem Description

When indexing code repositories or project directories, users expect that files and directories already ignored by git (via .gitignore) should also be excluded from indexing. This is standard behavior in most development tools.

Currently, the indexer will process and embed files that are gitignored, leading to:

- **Unnecessary indexing** of build artifacts, temporary files, and vendor dependencies
- **Bloated search results** with irrelevant content
- **Wasted storage** and processing time on files that shouldn't be searchable
- **Inconsistent behavior** with developer expectations

## Expected Behavior

The indexer should automatically respect .gitignore files by:

1. Reading .gitignore files in each directory during scanning
2. Applying gitignore patterns to exclude matching files and directories
3. Respecting gitignore hierarchy (parent directory patterns affect subdirectories)
4. Combining gitignore patterns with existing hardcoded ignore patterns

## Reproduction Steps

1. Create a project directory with a .gitignore file containing patterns like `*.log` or `build/`
2. Add files that match these patterns (e.g., `debug.log`, `build/output.js`)
3. Run `directory-indexer index /path/to/project`
4. Search for content that should be gitignored
5. Observe that gitignored files appear in search results

## Impact

- **Low severity** - Does not break core functionality
- **High user experience impact** - Violates developer expectations
- **Medium effort** - Requires gitignore parsing implementation