# Testing Coverage Gap Analysis & Implementation Plan

## Progress Tracking Checklist

- [x] **Gap 1: OpenAI Embedding Provider Tests** (Simple - 2-4 hours) ✅ **COMPLETED**
- [x] **Gap 2: Search Engine Direct Tests** (Medium - 4-6 hours) ✅ **COMPLETED**
- [x] **Gap 3: Configuration & Error Handling** (Medium - 4-8 hours) ✅ **COMPLETED**
- [ ] **Gap 4: CLI Arguments & Main Entry** (Medium - 3-5 hours)
- [ ] **Gap 5: File Monitoring & Advanced Features** (Complex - 8-12 hours)

**After each implementation:** Run `./scripts/pre-push` to ensure code quality and measure coverage improvement.

---

## Current Coverage Status

**Overall Coverage: 47.33%**

**Key Coverage Gaps by Module:**
- `main.rs`: 0% (entry point not tested)
- `cli/args.rs`: 0% (argument parsing not tested)
- `embedding/openai.rs`: 0% (OpenAI provider never tested)
- `search/engine.rs`: 0% (search functionality not directly tested)
- `indexing/monitor.rs`: 0% (file monitoring not tested)
- `mcp/server.rs`: 0% (MCP server spawns external process)
- `mcp/tools.rs`: 0% (MCP tools not directly tested)
- `mcp/json_rpc.rs`: 0% (JSON-RPC handling not tested)

---

## Gap 1: OpenAI Embedding Provider Tests
**Complexity: Simple** | **Estimated Time: 2-4 hours** | **Expected Coverage Gain: +5-8%**

### Problem
The OpenAI embedding provider (`src/embedding/openai.rs`) has 0% coverage because:
- Only Ollama provider is tested in integration tests
- No unit tests exist for OpenAI provider functionality
- Error handling for API failures not tested

### Implementation Tasks
1. **Add unit tests for OpenAI provider** in `src/embedding/openai.rs`:
   - Test embedding generation with mock HTTP responses
   - Test batch embedding processing
   - Test API key validation
   - Test error handling (network failures, API errors, rate limiting)
   - Test different response formats

2. **Add integration test variant** in `tests/library_integration_tests.rs`:
   - Test with OpenAI provider when `OPENAI_API_KEY` is set
   - Skip gracefully when API key not available

### Files to Modify
- `src/embedding/openai.rs` (add `#[cfg(test)]` module)
- `tests/library_integration_tests.rs` (add OpenAI variant tests)

---

## Gap 2: Search Engine Direct Tests  
**Complexity: Medium** | **Expected Time: 4-6 hours** | **Expected Coverage Gain: +8-12%**

### Problem
The search engine (`src/search/engine.rs`) has 0% coverage because:
- Search functionality only tested through CLI commands
- Vector similarity algorithms not directly tested
- Result ranking and filtering logic not tested

### Implementation Tasks
1. **Add unit tests for search engine** in `src/search/engine.rs`:
   - Test vector similarity calculations
   - Test result ranking algorithms
   - Test filtering by directory scope
   - Test limit handling
   - Test empty result scenarios

2. **Add integration tests for search functionality**:
   - Test search with different embedding providers
   - Test search performance with large datasets
   - Test search accuracy with known queries

### Files to Modify
- `src/search/engine.rs` (add comprehensive unit tests)
- `tests/search_integration_tests.rs` (new file for search-specific tests)

---

## Gap 3: Configuration & Error Handling
**Complexity: Medium** | **Expected Time: 4-8 hours** | **Expected Coverage Gain: +10-15%**

### Problem
Configuration and error handling have low coverage because:
- Environment validation logic not fully tested
- Config file loading/saving edge cases not tested
- Error type conversions and formatting not tested
- Invalid configuration scenarios not tested

### Implementation Tasks
1. **Enhance config tests** in `src/config/settings.rs`:
   - Test config loading from files vs environment variables
   - Test invalid configuration scenarios
   - Test config validation logic
   - Test default value generation

2. **Add comprehensive error handling tests** in `src/error.rs`:
   - Test all error type variants
   - Test error message formatting
   - Test error conversion chains
   - Test error serialization/deserialization

3. **Test environment validation** in `src/environment.rs`:
   - Test service availability checks
   - Test network timeout scenarios
   - Test invalid endpoint configurations

### Files to Modify
- `src/config/settings.rs` (expand test module)
- `src/error.rs` (add comprehensive test module)
- `src/environment.rs` (add test module)
- `tests/config_integration_tests.rs` (new file for config edge cases)

---

## Gap 4: CLI Arguments & Main Entry Points
**Complexity: Medium** | **Expected Time: 3-5 hours** | **Expected Coverage Gain: +6-10%**

### Problem
CLI and main entry points have 0% coverage because:
- Argument parsing logic not tested
- Main function error handling not tested
- CLI command routing not directly tested
- Help text and version display not tested

### Implementation Tasks
1. **Add CLI argument parsing tests** in `src/cli/args.rs`:
   - Test valid argument combinations
   - Test invalid argument scenarios
   - Test help text generation
   - Test default value handling

2. **Test main function behavior** (challenging - needs careful approach):
   - Test error exit codes
   - Test log level configuration
   - Test graceful shutdown scenarios

3. **Add library-based CLI command tests**:
   - Test command routing logic directly
   - Test command validation without external processes

### Files to Modify
- `src/cli/args.rs` (add comprehensive test module)
- `src/main.rs` (add limited testable components)
- `tests/cli_unit_tests.rs` (new file for CLI logic tests)

---

## Gap 5: File Monitoring & Advanced Features
**Complexity: Complex** | **Expected Time: 8-12 hours** | **Expected Coverage Gain: +15-20%**

### Problem
Advanced features have low/no coverage:
- File monitoring system not tested (`src/indexing/monitor.rs`)
- MCP server functionality tested only via external process
- JSON-RPC handling not directly tested
- Async processing and concurrency not tested

### Implementation Tasks
1. **Add file monitoring tests** in `src/indexing/monitor.rs`:
   - Test file system event detection
   - Test batch processing logic
   - Test file change debouncing
   - Test monitoring startup/shutdown

2. **Add direct MCP functionality tests**:
   - Test JSON-RPC message parsing and generation
   - Test tool registration and discovery
   - Test tool execution without full server
   - Test MCP protocol compliance

3. **Test async processing**:
   - Test concurrent file processing
   - Test backpressure handling
   - Test graceful cancellation
   - Test resource cleanup

### Files to Modify
- `src/indexing/monitor.rs` (add comprehensive test module)
- `src/mcp/json_rpc.rs` (add unit tests)
- `src/mcp/tools.rs` (add direct tool tests)
- `src/mcp/server.rs` (add testable components)
- `tests/monitoring_integration_tests.rs` (new file)
- `tests/mcp_unit_tests.rs` (new file for direct MCP tests)

---

## Implementation Strategy

### Phase 1: Quick Wins (Gaps 1-2)
Focus on simple, high-impact improvements that require minimal external dependencies.

### Phase 2: Core Functionality (Gap 3)
Address configuration and error handling to improve reliability and user experience.

### Phase 3: CLI Coverage (Gap 4)
Ensure command-line interface works correctly in all scenarios.

### Phase 4: Advanced Features (Gap 5)
Tackle complex async and monitoring functionality.

### Success Metrics
- **Target Overall Coverage: 80%+**
- **No module below 60% coverage**
- **All critical paths tested**
- **Error scenarios properly covered**

### Quality Gates
After each gap implementation:
1. Run `./scripts/pre-push` (linting, formatting, tests)
2. Measure coverage improvement
3. Verify no regressions in existing functionality
4. Update this checklist

---

## Expected Final Coverage Distribution

| Module | Current | Target | Gap |
|--------|---------|--------|-----|
| CLI commands | 52% | 80% | Gap 4 |
| Config | 43% | 85% | Gap 3 |
| Embedding | 65%* | 85% | Gap 1 |
| Storage | 60% | 75% | Gap 3 |
| Indexing | 60% | 80% | Gap 5 |
| Search | 0% | 85% | Gap 2 |
| MCP | 0% | 75% | Gap 5 |
| Utils | 77% | 85% | Gap 3 |

*Average of Ollama (high) and OpenAI (0%)

**Total Expected Coverage: 80%+**