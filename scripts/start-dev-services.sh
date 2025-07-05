#!/bin/bash

# Development services startup script
# Sets up isolated development environment using environment variables

set -e

echo "Starting development services for directory-indexer..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Standard ports for development
DEV_QDRANT_PORT=6333
DEV_OLLAMA_PORT=11434

# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
    echo -e "${RED}Docker is not running. Please start Docker first.${NC}"
    exit 1
fi

# Start Qdrant on standard port
echo -e "${YELLOW}Starting Qdrant on port $DEV_QDRANT_PORT...${NC}"
docker stop qdrant-dev || true
docker rm qdrant-dev || true

docker run -d \
    --name qdrant-dev \
    -p $DEV_QDRANT_PORT:6333 \
    -v qdrant_dev_storage:/qdrant/storage \
    qdrant/qdrant

# Wait for Qdrant to be ready
echo -e "${YELLOW}Waiting for Qdrant to be ready...${NC}"
for i in {1..30}; do
    if curl -s http://localhost:$DEV_QDRANT_PORT/ >/dev/null 2>&1; then
        echo -e "${GREEN}Qdrant is ready on http://localhost:$DEV_QDRANT_PORT${NC}"
        break
    fi
    if [ $i -eq 30 ]; then
        echo -e "${RED}Qdrant failed to start after 30 seconds${NC}"
        exit 1
    fi
    sleep 1
done

# Start Ollama on standard port
echo -e "${YELLOW}Starting Ollama on port $DEV_OLLAMA_PORT...${NC}"
docker stop ollama-dev || true
docker rm ollama-dev || true

docker run -d \
    --name ollama-dev \
    -p $DEV_OLLAMA_PORT:11434 \
    -v ollama_dev_data:/root/.ollama \
    ollama/ollama

# Wait for Ollama to be ready
echo -e "${YELLOW}Waiting for Ollama to be ready...${NC}"
for i in {1..30}; do
    if curl -s http://localhost:$DEV_OLLAMA_PORT/api/tags >/dev/null 2>&1; then
        echo -e "${GREEN}Ollama is ready on http://localhost:$DEV_OLLAMA_PORT${NC}"
        break
    fi
    if [ $i -eq 30 ]; then
        echo -e "${RED}Ollama failed to start after 30 seconds${NC}"
        exit 1
    fi
    sleep 1
done

# Pull required embedding model
echo -e "${YELLOW}Ensuring nomic-embed-text model is available...${NC}"
if docker exec ollama-dev ollama list | grep -q "nomic-embed-text"; then
    echo -e "${GREEN}nomic-embed-text model already available${NC}"
else
    echo -e "${YELLOW}Pulling nomic-embed-text model (this may take a while)...${NC}"
    docker exec ollama-dev ollama pull nomic-embed-text
    echo -e "${GREEN}nomic-embed-text model ready${NC}"
fi

# Set environment variables for development
export QDRANT_ENDPOINT="http://localhost:$DEV_QDRANT_PORT"
export OLLAMA_ENDPOINT="http://localhost:$DEV_OLLAMA_PORT"

echo -e "${GREEN}Development services are ready!${NC}"
echo
echo -e "${YELLOW}Services:${NC}"
echo "  Qdrant: http://localhost:$DEV_QDRANT_PORT"
echo "  Ollama: http://localhost:$DEV_OLLAMA_PORT"
echo
echo -e "${YELLOW}Environment variables set:${NC}"
echo "  QDRANT_ENDPOINT=$QDRANT_ENDPOINT"
echo "  OLLAMA_ENDPOINT=$OLLAMA_ENDPOINT"
echo
echo -e "${YELLOW}To use these services in your current shell:${NC}"
echo "  export QDRANT_ENDPOINT=$QDRANT_ENDPOINT"
echo "  export OLLAMA_ENDPOINT=$OLLAMA_ENDPOINT"
echo
echo -e "${YELLOW}To stop services:${NC}"
echo "  ./scripts/stop-dev-services.sh"
echo
echo -e "${YELLOW}To run tests with dev services:${NC}"
echo "  QDRANT_ENDPOINT=$QDRANT_ENDPOINT OLLAMA_ENDPOINT=$OLLAMA_ENDPOINT npm test"