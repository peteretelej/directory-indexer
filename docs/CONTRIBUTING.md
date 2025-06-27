# Contributing to Directory Indexer

## Development Environment Setup

### Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/) (latest stable version)
- **Node.js**: Version 16+ for npm packaging
- **Qdrant**: Local instance for vector storage
  ```bash
  # Using Docker
  docker run -p 6333:6333 qdrant/qdrant
  
  # Or install locally from https://qdrant.tech/
  ```
- **Embedding Provider**: Choose one:
  - **Ollama** (recommended for development): Install from [ollama.ai](https://ollama.ai/)
  - **OpenAI API**: Requires API key

### Initial Setup

1. **Clone and build**:
   ```bash
   git clone https://github.com/peteretelej/directory-indexer.git
   cd directory-indexer
   cargo build
   ```

2. **Install npm dependencies**:
   ```bash
   npm install
   ```

3. **Set up Ollama** (if using local embeddings):
   ```bash
   ollama pull nomic-embed-text
   ```

4. **Run tests**:
   ```bash
   cargo test
   npm test
   ```

## Project Structure

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Library interface
├── cli/                 # Command-line interface
├── config/              # Configuration handling
├── storage/             # SQLite + Qdrant storage
├── indexing/            # File processing and indexing
├── embedding/           # Embedding providers (Ollama, OpenAI)
├── search/              # Search engine logic
├── mcp/                 # MCP server implementation
├── error.rs             # Error types
└── utils.rs             # Utilities
```

## Development Workflow

### Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Cross-platform builds
npm run build-all
```

### Testing

```bash
# Run Rust unit tests
cargo test

# Run integration tests
cargo test --test integration_tests

# Test CLI commands
cargo run -- index ./test-docs
cargo run -- search "test query"
cargo run -- serve
```

### Linting and Formatting

```bash
# Format code
cargo fmt

# Check linting
cargo clippy

# Fix common issues
cargo clippy --fix
```

### Pre-Push Quality Checks

Before pushing code, run the pre-push script to ensure code quality:

```bash
# Run all quality checks
./scripts/pre-push
```

**Setting up Git Hook (recommended):**
```bash
# Copy the script as a git pre-push hook
cp scripts/pre-push .git/hooks/pre-push
chmod +x .git/hooks/pre-push
```

The script runs:
- `cargo clippy` - Rust linter with strict warnings
- `cargo fmt --check` - Code formatting validation  
- `cargo test` - All tests
- `cargo audit` - Security vulnerability scan (auto-installs if missing)

## MCP Server Development

### Testing MCP Integration

1. **Start the server**:
   ```bash
   cargo run -- serve
   ```

2. **Test with MCP client**:
   ```json
   {
     "mcpServers": {
       "directory-indexer": {
         "command": "cargo",
         "args": ["run", "--", "serve"]
       }
     }
   }
   ```

3. **Test tools manually**:
   ```bash
   # Test indexing
   cargo run -- index ~/Documents/test

   # Test search
   cargo run -- search "test query"

   # Test similar files
   cargo run -- similar ~/Documents/test/file.md
   ```

## Cross-Platform Support

### Building for All Platforms

```bash
# Install targets
rustup target add x86_64-pc-windows-gnu
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
rustup target add x86_64-unknown-linux-gnu

# Build all platforms
npm run build-all
```

### Testing Platform-Specific Features

- **Windows**: Path handling, file permissions
- **macOS**: ARM64 vs x64, file system events
- **Linux**: Various distributions, permissions

## Configuration

### Development Config

Create `~/.directory-indexer/config.json`:

```json
{
  "embedding": {
    "provider": "ollama",
    "model": "nomic-embed-text",
    "endpoint": "http://localhost:11434"
  },
  "storage": {
    "sqlite_path": "./dev-data.db",
    "qdrant": {
      "endpoint": "http://localhost:6333",
      "collection": "dev-documents"
    }
  },
  "indexing": {
    "chunk_size": 256,
    "overlap": 25,
    "max_file_size": 1048576,
    "concurrency": 2
  }
}
```

## Code Quality Guidelines

### Rust Best Practices

- **Error Handling**: Use `Result<T>` and custom error types
- **Async/Await**: Use tokio for async operations
- **Memory Safety**: Leverage Rust's ownership system
- **Documentation**: Document public APIs with `///`

### Code Style

- Follow `cargo fmt` formatting
- Use descriptive variable names
- Keep functions focused and small
- Add tests for new functionality

### Performance

- Use `cargo build --release` for benchmarking
- Profile with `cargo flamegraph` if available
- Monitor memory usage with large directories
- Test with realistic file volumes

## Testing Strategy

### Unit Tests

- Test individual functions and modules
- Mock external dependencies (Qdrant, embedding APIs)
- Use `tempfile` for filesystem tests

### Integration Tests

- Test complete workflows end-to-end
- Use real file systems and test directories
- Verify MCP protocol compliance
- Test cross-platform compatibility

### Performance Tests

- Benchmark indexing speed with large directories
- Test memory usage with many files
- Verify search response times

## Release Process

### Version Management

1. Update version in `Cargo.toml` and `package.json`
2. Update `CHANGELOG.md` with release notes
3. Create git tag: `git tag v0.1.0`
4. Push tag: `git push origin v0.1.0`

### Automated Releases

Releases are automated via GitHub Actions:

1. **PR Checks**: Run tests, linting, and cross-platform builds
2. **Release Build**: Triggered on version tags
3. **npm Publishing**: Automated upload to npm registry
4. **GitHub Release**: Create release with binaries

### Manual Release Testing

Before tagging a release:

```bash
# Test installation
npm pack
npm install -g ./directory-indexer-*.tgz

# Test CLI
directory-indexer --help
directory-indexer index ./test-docs
directory-indexer search "test"

# Test MCP integration
# (Configure with Claude Desktop/Cline and test tools)
```

## Troubleshooting

### Common Issues

**Build Failures:**
- Ensure Rust toolchain is up to date: `rustup update`
- Check for platform-specific dependencies

**Test Failures:**
- Verify Qdrant is running on port 6333
- Check Ollama is running and has required models
- Ensure test directories have proper permissions

**Runtime Issues:**
- Check configuration file location and format
- Verify network connectivity to embedding providers
- Monitor disk space for SQLite database

### Debug Logging

```bash
# Enable debug logging
RUST_LOG=debug cargo run -- serve

# Trace level logging
RUST_LOG=trace cargo run -- index ./docs
```

## Community

- **Issues**: Report bugs and feature requests on GitHub
- **Discussions**: Use GitHub Discussions for questions
- **Pull Requests**: Follow the PR template and guidelines

## Security

- Never commit API keys or sensitive configuration
- Use environment variables for secrets in CI/CD
- Report security issues privately to maintainers