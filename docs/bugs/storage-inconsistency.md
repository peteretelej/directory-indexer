# Bug: Storage Inconsistency After Manual Database Deletion

## Issue
When users manually delete SQLite database or Qdrant collection, the system doesn't detect the inconsistency and behaves incorrectly.

## Scenarios

### SQLite Deleted
- **Result**: Empty SQLite, full Qdrant
- **Behavior**: Next indexing creates duplicate vectors in Qdrant
- **Impact**: Inflated vector counts, potential search duplication

### Qdrant Deleted  
- **Result**: Full SQLite, empty Qdrant
- **Behavior**: Files appear indexed but have no searchable vectors
- **Impact**: Search returns no results despite showing indexed files in status

## Current Detection
None. System assumes storage consistency.

## Priority
Medium - affects users who manually manage storage systems.