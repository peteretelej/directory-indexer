#!/bin/bash

# Setup debug container for directory-indexer CLI testing
set -e

echo "Setting up debug container for directory-indexer CLI testing..."

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$PROJECT_ROOT"

# Clean up any existing container gracefully
echo "Cleaning up existing debug container..."
if docker ps -a --format '{{.Names}}' | grep -q "^test-directory-indexer$"; then
    echo "  Stopping existing container..."
    docker stop test-directory-indexer 2>/dev/null || true
    echo "  Removing existing container..."
    docker rm test-directory-indexer 2>/dev/null || true
fi

# Build the test container if it doesn't exist
if ! docker images | grep -q test-directory-indexer; then
    echo "Building test container..."
    docker build -f scripts/docker-debug/Dockerfile.test -t test-directory-indexer .
fi

# Run container with both mounts from the start
echo "Starting container with live dist mount..."
docker run -d --name test-directory-indexer --network host \
    -v "$(pwd)/dist:/usr/lib/node_modules/directory-indexer/dist" \
    -v "$(pwd)/tests/test_data:/test-data" \
    test-directory-indexer

# Install the package - dist mount will overlay the package's dist files
echo "Installing package with mounted dist overlay..."
docker exec test-directory-indexer npm install -g directory-indexer@latest

# Fix permissions on mounted CLI
docker exec test-directory-indexer chmod +x /usr/lib/node_modules/directory-indexer/dist/cli.js

echo ""
echo "✅ Debug container ready!"
echo ""
echo "Test commands:"
echo "  docker exec test-directory-indexer directory-indexer --help"
echo "  docker exec test-directory-indexer directory-indexer status"
echo "  docker exec test-directory-indexer directory-indexer index /test-data"
echo ""
echo "After making changes to src/cli.ts:"
echo "  npm run build"
echo "  docker exec test-directory-indexer directory-indexer --help"
echo ""