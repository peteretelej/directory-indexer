#!/bin/bash

# Cleanup script for act containers and networks
# Act doesn't clean up after itself, causing port conflicts

echo "🧹 Cleaning up act containers and networks..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Count act containers
ACT_CONTAINERS=$(docker ps -aq --filter "name=act-" | wc -l)
ACT_NETWORKS=$(docker network ls | grep act | wc -l)

if [ "$ACT_CONTAINERS" -eq 0 ] && [ "$ACT_NETWORKS" -eq 0 ]; then
    echo -e "${GREEN}✅ No act containers or networks to clean up${NC}"
    exit 0
fi

echo -e "${YELLOW}Found $ACT_CONTAINERS act containers and $ACT_NETWORKS act networks${NC}"

# Stop act containers
if [ "$ACT_CONTAINERS" -gt 0 ]; then
    echo -e "${YELLOW}Stopping act containers...${NC}"
    docker stop $(docker ps -q --filter "name=act-") 2>/dev/null || true
    
    echo -e "${YELLOW}Removing act containers...${NC}"
    docker rm $(docker ps -aq --filter "name=act-") 2>/dev/null || true
fi

# Remove act networks
if [ "$ACT_NETWORKS" -gt 0 ]; then
    echo -e "${YELLOW}Removing act networks...${NC}"
    docker network ls | grep act | awk '{print $1}' | xargs -r docker network rm 2>/dev/null || true
fi

echo -e "${GREEN}🎉 Act cleanup completed!${NC}"

# Show what's still running
echo -e "${YELLOW}Currently running containers:${NC}"
docker ps --format "table {{.Names}}\t{{.Ports}}" | head -10