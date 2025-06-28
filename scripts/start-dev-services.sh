#!/bin/bash

# Development services startup script
# Uses non-standard ports to avoid conflicts with existing services

set -e

echo "Starting development services for directory-indexer..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
    echo -e "${RED}Docker is not running. Please start Docker first.${NC}"
    exit 1
fi

# Start Qdrant on port 6335
echo -e "${YELLOW}Starting Qdrant on port 6335...${NC}"
if docker ps --format 'table {{.Names}}' | grep -q "^qdrant-dev$"; then
    echo -e "${YELLOW}Qdrant development container already running${NC}"
else
    docker run -d \
        --name qdrant-dev \
        -p 127.0.0.1:6335:6333 \
        -v qdrant_dev_storage:/qdrant/storage \
        qdrant/qdrant
    
    # Wait for Qdrant to be ready
    echo -e "${YELLOW}Waiting for Qdrant to be ready...${NC}"
    for i in {1..30}; do
        if curl -s http://localhost:6335/health >/dev/null 2>&1; then
            echo -e "${GREEN}Qdrant is ready on http://localhost:6335${NC}"
            break
        fi
        if [ $i -eq 30 ]; then
            echo -e "${RED}Qdrant failed to start after 30 seconds${NC}"
            exit 1
        fi
        sleep 1
    done
fi

# Start Ollama on port 11435
echo -e "${YELLOW}Starting Ollama on port 11435...${NC}"
if docker ps --format 'table {{.Names}}' | grep -q "^ollama-dev$"; then
    echo -e "${YELLOW}Ollama development container already running${NC}"
else
    docker run -d \
        --name ollama-dev \
        -p 127.0.0.1:11435:11434 \
        -v ollama_dev_data:/root/.ollama \
        ollama/ollama
    
    # Wait for Ollama to be ready
    echo -e "${YELLOW}Waiting for Ollama to be ready...${NC}"
    for i in {1..30}; do
        if curl -s http://localhost:11435/api/tags >/dev/null 2>&1; then
            echo -e "${GREEN}Ollama is ready on http://localhost:11435${NC}"
            break
        fi
        if [ $i -eq 30 ]; then
            echo -e "${RED}Ollama failed to start after 30 seconds${NC}"
            exit 1
        fi
        sleep 1
    done
fi

# Pull required embedding model
echo -e "${YELLOW}Ensuring nomic-embed-text model is available...${NC}"
if docker exec ollama-dev ollama list | grep -q "nomic-embed-text"; then
    echo -e "${GREEN}nomic-embed-text model already available${NC}"
else
    echo -e "${YELLOW}Pulling nomic-embed-text model (this may take a while)...${NC}"
    docker exec ollama-dev ollama pull nomic-embed-text
    echo -e "${GREEN}nomic-embed-text model ready${NC}"
fi

echo -e "${GREEN}Development services are ready!${NC}"
echo
echo -e "${YELLOW}Services:${NC}"
echo "  Qdrant: http://localhost:6335"
echo "  Ollama: http://localhost:11435"
echo
echo -e "${YELLOW}To stop services:${NC}"
echo "  ./scripts/stop-dev-services.sh"
echo
echo -e "${YELLOW}To run tests:${NC}"
echo "  cargo test --test connectivity_tests"