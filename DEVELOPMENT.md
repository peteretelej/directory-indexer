# Development Setup

## Quick Start

```bash
# Start dev services (Qdrant on 6335, Ollama on 11435)
./scripts/start-dev-services.sh

# Run tests
cargo test --test connectivity_tests

# Stop services  
./scripts/stop-dev-services.sh
```

## Manual Service Management

### Qdrant
```bash
docker run -d --name qdrant-dev -p 6335:6333 -v qdrant_dev_storage:/qdrant/storage qdrant/qdrant
```

### Ollama
```bash
docker run -d --name ollama-dev -p 11435:11434 -v ollama_dev_data:/root/.ollama ollama/ollama
docker exec ollama-dev ollama pull nomic-embed-text
```

## Health Checks

```bash
curl http://localhost:6335/health    # Qdrant
curl http://localhost:11435/api/tags # Ollama
```