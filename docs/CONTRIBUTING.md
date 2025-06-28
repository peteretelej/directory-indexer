# Contributing to Directory Indexer

## Development Environment Setup

### Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/) (latest stable version)
- **Node.js**: Version 16+ for npm packaging
- **Qdrant**: Local instance for vector storage

  ```bash
  # Using Docker
  docker run -d --name qdrant \
    -p 127.0.0.1:6333:6333 \
    -v qdrant_storage:/qdrant/storage \
    qdrant/qdrant

  # Or install locally from https://qdrant.tech/
  ```

- **Embedding Provider**: Choose one:
  - **Ollama** (recommended for development): Install natively for GPU support
    ```bash
    # Native installation (GPU acceleration)
    # Visit https://ollama.ai for installation instructions
    # Linux/macOS: curl -fsSL https://ollama.ai/install.sh | sh
    ollama pull nomic-embed-text
    ```
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
   # If using native installation (recommended for GPU)
   ollama pull nomic-embed-text
   
   # If using Docker (development only)
   docker exec ollama-dev ollama pull nomic-embed-text
   ```

4. **Run tests**:
   ```bash
   cargo test
   npm test
   ```

### Development Services

Use the development script to start Qdrant and Ollama services:

```bash
# Start dev services on standard ports
./scripts/start-dev-services.sh

# Run integration tests  
./scripts/test-integration-local.sh

# Stop services
./scripts/stop-dev-services.sh
```

**Note**: The development script runs services on standard ports (6333, 11434) and sets `QDRANT_ENDPOINT` and `OLLAMA_ENDPOINT` environment variables for you. Your tests and development workflows will automatically use these when available.

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

### Local CI Testing with Act

[Act](https://github.com/nektos/act) lets you run GitHub Actions workflows locally for faster feedback:

```bash
# Install act (if not already available)
# See: https://github.com/nektos/act#installation

# Run all CI jobs
act

# Run specific jobs
act -j lint              # Fast linting checks
act -j test-unit         # Unit tests only

# For integration tests, use the local script instead
./scripts/test-integration-local.sh  # Requires services running on standard ports

# List available workflows
act -l

# Run with custom event
act pull_request
```

**Benefits:**
- Test CI changes before pushing
- Debug workflow issues locally  
- Faster iteration than waiting for GitHub Actions
- Works offline with cached Docker images

**⚠️ Important - Act Cleanup:**
Act doesn't clean up containers/networks after runs, which can cause port conflicts:

```bash
# Clean up act containers and networks
docker stop $(docker ps -q --filter "name=act-") 2>/dev/null || true
docker rm $(docker ps -aq --filter "name=act-") 2>/dev/null || true
docker network ls | grep act | awk '{print $1}' | xargs -r docker network rm

# Or use the helper script
./scripts/cleanup-act.sh
```

**Note:** Integration tests require Docker services (Qdrant/Ollama) to be available.

### CI Strategy

To keep CI fast, integration tests are **conditional**:

- **Always run**: Lint, unit tests, build, smoke tests
- **Integration tests run when**:
  - Pushing to `main` branch
  - Opening PR to `main` branch  
  - Including `[integration]` in commit message or PR title

**For most development**: Fast feedback from unit tests and smoke tests  
**For releases**: Full integration test coverage

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
         "args": ["run", "--", "serve"],
         "env": {
           "QDRANT_ENDPOINT": "http://localhost:6333",
           "OLLAMA_ENDPOINT": "http://localhost:11434",
           "DIRECTORY_INDEXER_DB": "/path/to/dev/database.db"
         }
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

### Development Environment

Directory Indexer uses environment variables for configuration. The development scripts automatically set these for you:

```bash
# Set by ./scripts/start-dev-services.sh
export QDRANT_ENDPOINT="http://localhost:6333"
export OLLAMA_ENDPOINT="http://localhost:11434"

# Optional database path (default: ~/.directory-indexer/data.db)
export DIRECTORY_INDEXER_DB="/path/to/your/database.db"

# Optional API keys (if needed)
export QDRANT_API_KEY="your-key"
export OLLAMA_API_KEY="your-key"
```

### Manual Configuration

If running services on different ports or using hosted services:

```bash
# Custom ports
export QDRANT_ENDPOINT="http://localhost:6334"
export OLLAMA_ENDPOINT="http://localhost:11435"

# Custom database location
export DIRECTORY_INDEXER_DB="/custom/path/to/database.db"

# Qdrant Cloud
export QDRANT_ENDPOINT="https://your-cluster.qdrant.io"
export QDRANT_API_KEY="your-qdrant-cloud-key"

# Hosted Ollama
export OLLAMA_ENDPOINT="https://your-ollama-host.com"
export OLLAMA_API_KEY="your-ollama-key"
```

## Testing

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration_tests

# All tests
./scripts/pre-push

# Tests with custom database location (useful for CI/testing)
DIRECTORY_INDEXER_DB=/tmp/test.db cargo test --test error_scenarios_tests
```

## Publishing

### Pre-publish Steps

```bash
# 1. Update versions
# Edit Cargo.toml: version = "0.1.0"
# Edit package.json: "version": "0.1.0"

# 2. Run quality checks
./scripts/pre-push

# 3. Check what will be included
cargo package --list
npm pack --dry-run

# 4. Test publish without uploading
cargo publish --dry-run
npm publish --dry-run
```

### Publishing to crates.io

```bash
# Account setup at https://crates.io, then:
cargo login
cargo publish
cargo install directory-indexer --force

cargo search directory-indexer
directory-indexer --version
```

### Publishing to npm

```bash
npm login
npm run build-all
npm publish
npm install -g directory-indexer

npm view directory-indexer
directory-indexer --version
```

### Post-publish

```bash
git add Cargo.toml package.json
git commit -m "Release v0.1.0"
git tag v0.1.0
git push origin main --tags
```

## Release Process

### Automated Releases

Releases are automated via GitHub Actions:

1. **PR Checks**: Run tests, linting, and cross-platform builds
2. **Release Build**: Triggered on version tags
3. **npm Publishing**: Automated upload to npm registry
4. **GitHub Release**: Create release with binaries

## Troubleshooting

### Common Issues

```bash
# Build issues
rustup update

# Test failures
docker run -d --name qdrant \
  -p 127.0.0.1:6333:6333 \
  -v qdrant_storage:/qdrant/storage \
  qdrant/qdrant
ollama pull nomic-embed-text

# Debug logging
RUST_LOG=debug cargo run -- serve
```

### Debug Logging

```bash
# Enable debug logging
RUST_LOG=debug cargo run -- serve

# Trace level logging
RUST_LOG=trace cargo run -- index ./docs
```

## Community

- Issues: GitHub Issues

## Security

- No API keys in commits
- Use env vars for secrets
- Report security issues privately
