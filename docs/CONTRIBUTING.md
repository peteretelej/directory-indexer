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

### Isolated Development Environment

In case you don't want to use the Ollama and Qdrant instances on the default ports, you can use docker to run them in differen ports using the script below:

```bash
# Start dev services (isolated ports for development)
./scripts/start-dev-services.sh

# Run tests
cargo test --test connectivity_tests

# Stop services
./scripts/stop-dev-services.sh
```

The script runs them on different ports to avoid conflicts with existing instances and sets up environment variables for seamless integration.

**Note**: The development script runs services on isolated ports (6335, 11435) and sets `QDRANT_URL` and `OLLAMA_ENDPOINT` environment variables for you. Your tests and development workflows will automatically use these when available.

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

## Testing

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration_tests

# All tests
./scripts/pre-push
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
