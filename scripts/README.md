# Scripts

## Development Scripts

### `start-dev-services.sh`
Starts development services (Ollama and Qdrant) via Docker containers.

### `stop-dev-services.sh`
Stops development services (Ollama and Qdrant) Docker containers.

## Git Hooks

### `pre-push`
Git pre-push hook that runs before pushing changes to the repository.

## Testing

### `docker-debug/`
Docker-based testing environment for debugging directory-indexer CLI functionality. See [docker-debug/README.md](./docker-debug/README.md) for complete setup and usage instructions.

Quick start:
```bash
./scripts/docker-debug/setup-debug-container.sh
```