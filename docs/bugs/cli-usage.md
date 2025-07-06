# CLI Usage Issue

## Problem Statement

The directory-indexer CLI commands fail to execute properly when installed via npm or used with npx. Commands hang indefinitely and produce no output, including basic help and version commands.

## Symptoms

- `directory-indexer --help` hangs with no output
- `directory-indexer --version` hangs with no output  
- `npx directory-indexer --help` hangs with no output
- All CLI commands hang indefinitely (tested with 3-5 second timeouts)
- Commands exit with code 0 but produce no stdout or stderr
- Issue occurs both with global npm install and npx usage

## Expected Behavior

CLI commands should execute normally and display output:
```bash
$ directory-indexer --help
Usage: directory-indexer [options] [command]

AI-powered directory indexing with semantic search
...
```

## Current Status - RESOLVED ✅

- ✅ **Package structure**: Correct `bin` configuration in package.json
- ✅ **Shebang**: Proper `#!/usr/bin/env node` in CLI script
- ✅ **Symlink**: npm creates `/usr/bin/directory-indexer` symlink correctly
- ✅ **Permissions**: CLI script has execute permissions
- ✅ **Dependencies**: Qdrant and Ollama services running
- ✅ **Execution**: Commands now work properly with CLI launcher pattern

## Resolution Implemented (v0.0.13)

### Root Cause Identified
The CLI script called `main()` unconditionally at module load time, causing hanging when the module was imported/executed via npm symlinks.

### Solution Applied
Implemented industry-standard CLI launcher pattern used by Vite ecosystem:

1. **CLI Launcher**: Created `bin/directory-indexer.js` that dynamically imports the CLI module
2. **Clean Module Structure**: Removed unconditional execution from CLI module
3. **Dynamic Version**: Both CLI and MCP server now read version from package.json
4. **Updated Package Config**: Changed bin path and added proper file includes

### New Package Configuration
```json
{
  "bin": {
    "directory-indexer": "bin/directory-indexer.js"
  },
  "files": ["dist/", "bin/"],
  "type": "module"
}
```

### CLI Launcher (`bin/directory-indexer.js`)
```javascript
#!/usr/bin/env node
import('../dist/cli.js').then(({ main }) => {
  main().catch((error) => {
    console.error('CLI Error:', error);
    process.exit(1);
  });
}).catch((error) => {
  console.error('Failed to load CLI:', error);
  process.exit(1);
});
```

### CLI Module (`src/cli.ts`)
```typescript
// No unconditional execution - safely importable
export async function main() {
  // CLI logic here
}
// Main function is exported for launcher to use
```

## Reproduction Steps

### Prerequisites
1. Start development services:
   ```bash
   ./scripts/start-dev-services.sh
   ```

2. Set up debug container:
   ```bash
   ./scripts/docker-debug/setup-debug-container.sh
   ```

### Test Commands
```bash
# Test direct CLI usage (hangs)
docker exec test-directory-indexer directory-indexer --help

# Test npx usage (hangs)  
docker exec test-directory-indexer timeout 5 npx directory-indexer --help

# Test with timeout to confirm hanging
docker exec test-directory-indexer timeout 3 directory-indexer --version
```

### Debug Information
```bash
# Check symlink
docker exec test-directory-indexer ls -la /usr/bin/directory-indexer

# Check package structure
docker exec test-directory-indexer ls -la /usr/lib/node_modules/directory-indexer/

# Check mounted dist
docker exec test-directory-indexer ls -la /usr/lib/node_modules/directory-indexer/dist/
```

## Development Workflow

The debug container allows live code iteration:

1. **Make changes** to `src/cli.ts`
2. **Rebuild**: `npm run build`  
3. **Test immediately**: `docker exec test-directory-indexer directory-indexer --help`

No container restart needed due to mounted dist directory.

## Environment

- **Container**: Ubuntu 22.04 with Node.js 20.19.3
- **Package**: directory-indexer@0.0.12 installed globally
- **Services**: Qdrant (localhost:6333) and Ollama (localhost:11434) running
- **Test Data**: Available at `/test-data` in container

## Testing After v0.0.13 Publication

Once version 0.0.13 is published to npm:

1. **Test with Docker container**:
   ```bash
   ./scripts/docker-debug/setup-debug-container.sh
   docker exec test-directory-indexer directory-indexer --help
   ```

2. **Test npx usage**:
   ```bash
   npx directory-indexer@0.0.13 --help
   npx directory-indexer@0.0.13 --version
   ```

3. **Test global install**:
   ```bash
   npm install -g directory-indexer@0.0.13
   directory-indexer --help
   ```

## Known Issues Post-Fix

- Integration tests need updates to work with new CLI launcher pattern
- Test setup may need adjustment for CLI execution testing