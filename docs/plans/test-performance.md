# Test Performance Optimization Plan

## Implementation Instructions

**IMPORTANT: Follow these guidelines when implementing this plan:**

### Work Organization
- **Always work in phases** - Complete one phase fully before moving to the next
- **Use TodoWrite tool** to track progress for each phase
- **Run tests after each change** to validate correctness
- **Keep code simple, clear and concise** - avoid over-engineering

### Phase Workflow
1. **Start phase**: Create todos for all tasks in the current phase
2. **Work incrementally**: Complete one todo at a time
3. **Validate continuously**: Run `npm test` after each significant change
4. **Mark complete**: Only mark todos complete when tests pass
5. **Phase completion**: Run full test suite and verify coverage before next phase

### Code Quality Standards
- **Simplicity first**: Choose the simplest solution that works
- **Clear intent**: Test names and structure should be self-documenting
- **Concise implementation**: Remove redundancy, keep only essential tests
- **Maintainable**: Future developers should easily understand the test organization

### Validation Requirements
- All tests must pass after each change
- Code coverage must be maintained or improved
- Test execution time should show measurable improvement
- No functionality should be lost during consolidation

## Current State Analysis

### Performance Issues

Our test suite currently suffers from significant performance bottlenecks:

- **Total test execution time**: ~2+ minutes (often times out)
- **Integration tests**: Major bottleneck due to repeated indexing operations
- **Test count**: 100+ tests across 13 files with significant redundancy
- **Service dependencies**: Heavy reliance on external Qdrant/Ollama services

### Root Causes

1. **Over-isolation**: Each integration test creates complete new environments
2. **Redundant operations**: Multiple tests re-index identical test data  
3. **Scattered organization**: Related functionality tested across multiple files
4. **Poor fixture management**: No shared test fixtures or cached results

## Test File Audit

### Unit Tests (9 files, ~80 tests)

| File | Tests | Purpose | Redundancy Level | Performance |
|------|-------|---------|------------------|-------------|
| `cli-handlers.test.ts` | 16 | CLI command handlers | Medium (overlap with cli.unit.test.ts) | ✅ Fast |
| `cli.unit.test.ts` | 8 | CLI argument parsing | Medium (overlap with cli-handlers) | ✅ Fast |
| `edge-cases.unit.test.ts` | 16 | Edge case testing | High (overlap with unit.test.ts) | ✅ Fast |
| `error-handling.unit.test.ts` | 12 | Error scenarios | Low | ✅ Fast |
| `mcp-handlers.test.ts` | 25 | MCP protocol handlers | Low | ✅ Fast |
| `providers.unit.test.ts` | 15 | Embedding providers | Medium (overlap with unit.test.ts) | ✅ Fast |
| `reset.unit.test.ts` | 4 | Reset functionality | Low | ✅ Fast |
| `unit.test.ts` | 22 | General utilities | **High (spreads across multiple concerns)** | ✅ Fast |
| `prerequisites.test.ts` | 6 | Service validation | Low | ⚠️ **Requires live services** |

### Integration Tests (4 files, ~30 tests)

| File | Tests | Purpose | Performance Impact |
|------|-------|---------|-------------------|
| `integration/cli.test.ts` | 15 | Full CLI workflows | 🔴 **MAJOR BOTTLENECK** - Full re-indexing per test |
| `integration/core.test.ts` | 12 | Core functionality | 🔴 **MAJOR BOTTLENECK** - Database + file operations |
| `integration/mcp.test.ts` | 4 | MCP server integration | 🟡 Moderate - Service dependent |
| `integration.test.ts` | - | Test orchestration | ✅ Fast |

## Optimization Strategy

### Phase 1: Consolidation (Immediate - 40% time reduction)

#### 1.1 Merge Redundant Unit Tests

**Target files for consolidation:**

```
Current                          →  Proposed
├── unit.test.ts (scattered)     →  ├── config.unit.test.ts
├── edge-cases.unit.test.ts      →  ├── storage.unit.test.ts  
├── providers.unit.test.ts       →  ├── providers.unit.test.ts
└── cli-handlers.test.ts         →  ├── cli.unit.test.ts (merged)
    cli.unit.test.ts             →  └── utils.unit.test.ts
```

**Benefits:**
- Eliminate ~30% of redundant tests
- Clear separation of concerns
- Easier maintenance

#### 1.2 Delete Low-Value Tests

**Candidates for removal:**
- Duplicate configuration loading tests
- Redundant CLI argument validation
- Overlapping edge case scenarios that don't add coverage

**Estimated reduction:** 15-20 tests

### Phase 2: Integration Test Optimization (Major - 70% time reduction)

#### 2.1 Shared Test Fixtures

**Current problem:**
```typescript
// Each test does this expensive operation
describe('Test 1', () => {
  const testEnv = await createIsolatedTestEnvironment('test1');
  await indexDirectories(['/test/data'], config); // 30-60 seconds
});

describe('Test 2', () => {
  const testEnv = await createIsolatedTestEnvironment('test2');  
  await indexDirectories(['/test/data'], config); // 30-60 seconds
});
```

**Proposed solution:**
```typescript
// Shared setup
beforeAll(async () => {
  sharedTestEnv = await createIsolatedTestEnvironment('shared-integration');
  await indexDirectories([getTestDataPath()], config); // Once: 30-60 seconds
}, 120000);

// Lightweight test isolation
beforeEach(() => {
  // Only isolate search/query state, not full re-indexing
});
```

#### 2.2 Test Categories

**Fast Integration Suite (~5-10 seconds):**
- Use shared indexed test data
- Test search, retrieval, and analysis operations
- Mock external service calls where possible

**Slow Integration Suite (~30-60 seconds):**
- Full indexing workflows
- Database creation/destruction
- Service integration testing
- Run only on CI or when specifically requested

#### 2.3 Smart Test Data Management

**Current:** Each test indexes full test directory
**Proposed:** Layered test data approach

```typescript
// Minimal dataset for basic operations (5-10 files)
const MINIMAL_TEST_DATA = '/tests/data/minimal';

// Full dataset for comprehensive testing (50+ files)  
const FULL_TEST_DATA = '/tests/data/full';

// Use minimal for most tests, full only when needed
```

### Phase 3: Architecture Improvements (Long-term - 90% time reduction)

#### 3.1 Mock Integration Layer

Create a "mock-integration" testing approach:

```typescript
// Mock external services but use real data structures
class MockQdrantClient implements QdrantClient {
  // In-memory vector store for testing
  // Instant operations, real API compatibility
}

class MockEmbeddingProvider implements EmbeddingProvider {
  // Deterministic embeddings for consistent testing
  // No network calls, instant results
}
```

#### 3.2 Parallel Test Execution

**Current issue:** Tests run sequentially due to `.sequential()`
**Solution:** 
- Separate unit tests (can run in parallel)
- Use proper test isolation for parallel integration tests
- Utilize Vitest's native parallelization

#### 3.3 Lazy Service Dependencies

**Current:** All integration tests require Qdrant + Ollama
**Proposed:** 
- Mock services by default
- Real services only for explicit integration tests
- Skip integration tests if services unavailable (rather than fail)

## Implementation Plan

### Week 1: Unit Test Consolidation
- [ ] Merge `cli.unit.test.ts` and `cli-handlers.test.ts`
- [ ] Extract configuration tests into `config.unit.test.ts`
- [ ] Extract storage tests into `storage.unit.test.ts`
- [ ] Extract utilities tests into `utils.unit.test.ts`
- [ ] Delete redundant tests from `unit.test.ts` and `edge-cases.unit.test.ts`

### Week 2: Integration Test Shared Fixtures
- [ ] Implement shared test environment setup
- [ ] Create minimal and full test datasets
- [ ] Refactor integration tests to use shared fixtures
- [ ] Implement proper cleanup strategies

### Week 3: Test Categories and Optimization
- [ ] Separate fast and slow test suites
- [ ] Implement mock integration layer
- [ ] Enable parallel execution for unit tests
- [ ] Add test performance monitoring

### Week 4: Validation and Fine-tuning
- [ ] Verify coverage maintained or improved
- [ ] Performance benchmarking
- [ ] Documentation updates
- [ ] CI/CD pipeline optimization

## Expected Outcomes

### Performance Improvements
- **Unit tests**: 5-10 seconds (currently 20-30 seconds)
- **Fast integration**: 10-20 seconds (currently 60-120 seconds)  
- **Full integration**: 30-60 seconds (currently 120-180 seconds)
- **Total time**: 45-90 seconds (currently 180-300+ seconds)

### Maintenance Benefits
- **Clearer test organization**: Related tests grouped together
- **Reduced duplication**: ~30% fewer tests overall
- **Better failure isolation**: Clear separation between unit and integration failures
- **Easier debugging**: Less noise from redundant test failures

### Coverage Maintenance
- **Line coverage**: Maintain current 80-90% coverage
- **Branch coverage**: Improve by removing redundant edge cases and adding targeted tests
- **Integration coverage**: Better coverage of real-world scenarios with shared fixtures

## Risks and Mitigation

### Risk: Shared fixtures hide issues
**Mitigation**: Maintain critical isolation tests for database operations

### Risk: Reduced test coverage
**Mitigation**: Coverage reporting and targeted test addition where gaps found

### Risk: Mock divergence from real services
**Mitigation**: Regular integration test runs against real services in CI

### Risk: Test data management complexity
**Mitigation**: Clear documentation and helper utilities for test data lifecycle

## Success Metrics

- [ ] Total test execution time < 90 seconds
- [ ] Unit test execution time < 10 seconds  
- [ ] Integration test failure isolation (unit failures don't block integration)
- [ ] Maintained or improved code coverage
- [ ] Developer feedback on test clarity and maintainability
- [ ] CI/CD pipeline reliability improvement

## Next Steps

1. **Get stakeholder approval** for this optimization plan
2. **Set up performance baseline** measurements
3. **Begin Phase 1** unit test consolidation
4. **Monitor progress** with regular time measurements
5. **Iterate and adjust** based on results and feedback

---

*This plan aims to reduce test execution time by 70-80% while maintaining comprehensive coverage and improving test organization. The focus is on eliminating redundancy and optimizing expensive operations rather than reducing test quality.*