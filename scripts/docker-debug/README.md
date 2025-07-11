# Docker Debug Container

Container setup for testing and debugging directory-indexer CLI functionality with live code iteration.

## Purpose

This container allows testing the CLI in a clean Ubuntu environment with Node.js while mounting the local `dist/` folder for live code updates. Useful for:

- Testing npm package installation and CLI functionality
- Debugging CLI entry point and symlink issues
- Validating commands work as expected for end users
- Testing both `directory-indexer` and `npx directory-indexer` usage

## Prerequisites

Start the development services first:

```bash
./scripts/start-dev-services.sh
```

This starts Ollama and Qdrant via Docker, which the CLI needs for indexing and search operations.

## Setup

**Quick setup with helper script:**
```bash
./scripts/docker-debug/setup-debug-container.sh
```

**Manual setup:**

1. **Build the test container:**
   ```bash
   docker build -f scripts/docker-debug/Dockerfile.test -t test-directory-indexer .
   ```

2. **Set up debug container:**
   ```bash
   # Clean up any existing container
   docker stop test-directory-indexer 2>/dev/null || true
   docker rm test-directory-indexer 2>/dev/null || true
   
   # Run container without dist mount first
   docker run -d --name test-directory-indexer --network host \
     -v $(pwd)/tests/test_data:/test-data \
     test-directory-indexer
   
   # Install the package to create proper structure
   docker exec test-directory-indexer npm install -g directory-indexer@latest
   
   # Stop and recreate with dist mounted for live development
   docker stop test-directory-indexer
   docker rm test-directory-indexer
   docker run -d --name test-directory-indexer --network host \
     -v $(pwd)/dist:/usr/lib/node_modules/directory-indexer/dist \
     -v $(pwd)/tests/test_data:/test-data \
     test-directory-indexer
   
   # Fix permissions on mounted CLI
   docker exec test-directory-indexer chmod +x /usr/lib/node_modules/directory-indexer/dist/cli.js
   ```

## Testing

Test CLI commands:

```bash
# Help and version
docker exec test-directory-indexer directory-indexer --help
docker exec test-directory-indexer directory-indexer --version

# Status and indexing
docker exec test-directory-indexer directory-indexer status
docker exec test-directory-indexer directory-indexer index /test-data
docker exec test-directory-indexer directory-indexer search "test query"

# Test with npx
docker exec test-directory-indexer npx directory-indexer --help
docker exec test-directory-indexer npx directory-indexer status
```

## Live Development

After making changes to source code:

1. **Rebuild locally:**
   ```bash
   npm run build
   ```

2. **Test immediately in container** (no restart needed):
   ```bash
   docker exec test-directory-indexer directory-indexer --help
   ```

The mounted `dist/` folder allows immediate testing of changes without rebuilding the container.

## Cleanup

```bash
docker stop test-directory-indexer
docker rm test-directory-indexer
```

## Network Setup

The container uses `--network host` to access:
- **Qdrant**: `localhost:6333`
- **Ollama**: `localhost:11434`

Both services must be running on the host before testing CLI commands that require them.

## Workspace Configuration

Workspaces are configured via environment variables:
```bash
export WORKSPACE_<NAME>="/path/to/directory"
export WORKSPACE_MULTIPATH="/path1,/path2"  # Multiple paths
```