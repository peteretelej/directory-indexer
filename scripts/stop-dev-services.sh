#!/bin/bash

# Development services stop script

set -e

echo "Stopping development services..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Stop and remove Qdrant dev container
if docker ps -q --filter "name=qdrant-dev" | grep -q .; then
    echo -e "${YELLOW}Stopping Qdrant development container...${NC}"
    docker stop qdrant-dev
    docker rm qdrant-dev
    echo -e "${GREEN}Qdrant development container stopped${NC}"
else
    echo -e "${YELLOW}Qdrant development container not running${NC}"
fi

# Stop and remove Ollama dev container
if docker ps -q --filter "name=ollama-dev" | grep -q .; then
    echo -e "${YELLOW}Stopping Ollama development container...${NC}"
    docker stop ollama-dev
    docker rm ollama-dev
    echo -e "${GREEN}Ollama development container stopped${NC}"
else
    echo -e "${YELLOW}Ollama development container not running${NC}"
fi

echo -e "${GREEN}Development services stopped!${NC}"
echo
echo -e "${YELLOW}Note:${NC} Data volumes are preserved:"
echo "  qdrant_dev_storage"
echo "  ollama_dev_data"
echo
echo -e "${YELLOW}To completely remove data volumes:${NC}"
echo "  docker volume rm qdrant_dev_storage ollama_dev_data"