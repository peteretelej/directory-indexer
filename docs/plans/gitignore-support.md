# Implementation Plan: .gitignore File Support

## Objective

Add support for reading and respecting .gitignore files during directory indexing to automatically exclude files and directories that are already ignored by git.

## Current Implementation

**File:** `src/utils.ts:110-113`
```typescript
export function shouldIgnoreFile(filePath: string, ignorePatterns: string[]): boolean {
  const normalizedPath = normalizePath(filePath);
  return ignorePatterns.some(pattern => normalizedPath.includes(pattern));
}
```

**File:** `src/config.ts:125`
```typescript
ignorePatterns: ['.git', 'node_modules', 'target', '.DS_Store'],
```

The current implementation uses simple string matching against hardcoded patterns.

## Solution Overview

Implement gitignore parsing and pattern matching that:

1. **Discovers .gitignore files** in directory hierarchy during scanning
2. **Parses gitignore syntax** including patterns, negations, and comments
3. **Applies patterns hierarchically** (parent patterns affect subdirectories)
4. **Combines with essential ignore patterns** - hardcoded patterns for directories that should never be indexed (node_modules, .git, etc.)

## Implementation Steps

### Step 1: Add gitignore parsing utility

**File:** `src/gitignore.ts` (new file)

Use `ignore` npm package for proven gitignore compatibility:

```typescript
import ignore from 'ignore';

export function createIgnoreFilter(gitignoreContent: string): (filePath: string) => boolean
export function findGitignoreFiles(directory: string): Promise<string[]>
export function loadGitignoreRules(directory: string): Promise<ReturnType<typeof ignore>>
```

Key features:
- Use `ignore` package for complete gitignore syntax support
- Only traverse within the directory being indexed (no parent directory scanning)
- Cache ignore filters per directory
- Combine with hardcoded essential patterns

### Step 2: Modify directory scanning

**File:** `src/indexing.ts`

Update `scanDirectory` function to:
1. Find .gitignore files within the indexed directory tree only
2. Create ignore filters using `ignore` package
3. Apply both essential hardcoded patterns and gitignore rules
4. Respect directory boundaries - no traversal outside indexed paths

### Step 3: Enhance ignore checking

**File:** `src/utils.ts`

Modify `shouldIgnoreFile` function:
```typescript
export function shouldIgnoreFile(
  filePath: string, 
  ignorePatterns: string[],
  ignoreFilter?: ReturnType<typeof ignore>
): boolean
```

Apply essential hardcoded patterns first, then gitignore rules. Essential patterns (node_modules, .git) cannot be overridden by gitignore negations.

### Step 4: Update configuration

**File:** `src/config.ts`

Add optional configuration:
```typescript
indexing: {
  // existing fields...
  respectGitignore: boolean; // default: true
  gitignoreFiles: string[];  // default: ['.gitignore']
}
```

### Step 5: Add tests

**File:** `tests/gitignore.test.ts` (new file)

Test scenarios:
- Basic gitignore pattern matching
- Negation patterns (`!pattern`)
- Directory-specific patterns (`pattern/`)
- Hierarchical gitignore files
- Integration with existing ignore patterns

## Technical Details

### Gitignore Pattern Matching

Implement proper gitignore semantics:

- **Glob patterns**: `*.log`, `build/`, `src/**/*.tmp`
- **Negations**: `!important.log` (re-include previously ignored files)
- **Directory markers**: `dir/` (only match directories)
- **Anchoring**: `/root-only` vs `anywhere/pattern`

### Performance Considerations

- **Cache parsed rules** per directory to avoid re-parsing
- **Short-circuit evaluation** - check hardcoded patterns first (faster)
- **Minimal file I/O** - read .gitignore files only once per directory

### Essential vs Gitignore Patterns

- **Essential patterns** (node_modules, .git, target, .DS_Store) always ignored - these directories should never be indexed regardless of gitignore
- **Gitignore patterns** provide project-specific ignore rules
- **No parent directory traversal** - only process .gitignore files within the indexed directory boundaries

## Dependencies

**New dependency:** `ignore` npm package

```bash
npm install ignore
npm install --save-dev @types/ignore
```

The `ignore` package provides complete gitignore syntax support and is the most reliable solution for gitignore pattern matching.

## Testing Strategy

### Integration Tests (Primary)

Add to existing `tests/integration.test.ts`:

1. **Add test files to `tests/test_data/`**:
   - `debug.log` - should be ignored by `*.log` pattern
   - `important.log` - should be indexed despite `*.log` (negation test)
   - `temp/cache.tmp` - directory and file to test `temp/` pattern
   - `build/output.js` - test build directory ignoring

2. **Test scenarios**:
   ```typescript
   // Create .gitignore at test runtime
   const gitignorePath = 'tests/test_data/.gitignore';
   const gitignoreContent = '*.log\ntemp/\nbuild/\n!important.log';
   await fs.writeFile(gitignorePath, gitignoreContent);
   
   try {
     // Index and verify
     const result = await indexDirectories(['tests/test_data/']);
     // Assert debug.log, temp/, build/ are ignored
     // Assert important.log is indexed (negation pattern)
   } finally {
     // Cleanup
     await fs.unlink(gitignorePath);
   }
   ```

3. **Essential pattern tests**:
   - Verify node_modules always ignored even if gitignore says `!node_modules`
   - Test .git directory exclusion
   - Confirm essential patterns override gitignore negations

### Unit Tests (Minimal)

Simple tests in `tests/unit.test.ts`:
- `ignore` package integration
- Essential vs gitignore pattern precedence
- Directory boundary enforcement

**Keep tests small, focused, and use real files - no mocking of filesystem operations.**

## Rollout Plan

1. **Phase 1**: Implement core gitignore parsing (Step 1)
2. **Phase 2**: Integrate with directory scanning (Steps 2-3)  
3. **Phase 3**: Add configuration options (Step 4)
4. **Phase 4**: Comprehensive testing and documentation (Step 5)

## Risk Assessment

**Low risk** - Implementation is additive and can be feature-flagged for safe rollout.

**Mitigation strategies**:
- Default to current behavior if gitignore parsing fails
- Extensive logging in verbose mode for debugging
- Configuration option to disable gitignore support