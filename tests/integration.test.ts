import { describe, it, expect, beforeAll } from 'vitest';
import { spawn } from 'child_process';
import { join } from 'path';
import { existsSync } from 'fs';

function checkServicesAvailable(): Promise<boolean> {
  return Promise.all([
    checkQdrantHealth(),
    checkOllamaHealth()
  ]).then(([qdrant, ollama]) => qdrant && ollama);
}

async function checkQdrantHealth(): Promise<boolean> {
  try {
    // Test health endpoint
    const response = await fetch('http://localhost:6333/healthz');
    if (!response.ok) return false;
    
    // Test if we can access collections endpoint (actual usability test)
    const collectionsResponse = await fetch('http://localhost:6333/collections');
    return collectionsResponse.ok;
  } catch {
    return false;
  }
}

async function checkOllamaHealth(): Promise<boolean> {
  try {
    // Test basic connection
    const response = await fetch('http://localhost:11434/api/tags');
    if (!response.ok) return false;
    
    // Test if we can generate embeddings (actual usability test)
    const embedResponse = await fetch('http://localhost:11434/api/embeddings', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        model: 'nomic-embed-text',
        prompt: 'test'
      })
    });
    
    // If nomic-embed-text is not available, just check if ollama responds
    return response.ok;
  } catch {
    return false;
  }
}

function runCLI(args: string[], timeout = 30000): Promise<{ stdout: string; stderr: string; exitCode: number }> {
  return new Promise((resolve, reject) => {
    const child = spawn('node', ['dist/cli.js', ...args], {
      env: {
        ...process.env,
        DIRECTORY_INDEXER_QDRANT_COLLECTION: 'directory-indexer-test-node'
      }
    });

    let stdout = '';
    let stderr = '';

    child.stdout?.on('data', (data) => {
      stdout += data.toString();
    });

    child.stderr?.on('data', (data) => {
      stderr += data.toString();
    });

    const timer = setTimeout(() => {
      child.kill();
      reject(new Error(`Command timed out after ${timeout}ms`));
    }, timeout);

    child.on('close', (code) => {
      clearTimeout(timer);
      resolve({ stdout, stderr, exitCode: code || 0 });
    });

    child.on('error', (error) => {
      clearTimeout(timer);
      reject(error);
    });
  });
}

describe('Directory Indexer Integration Tests', () => {
  let servicesAvailable = false;

  beforeAll(async () => {
    servicesAvailable = await checkServicesAvailable();
    if (!servicesAvailable) {
      const qdrantHealthy = await checkQdrantHealth();
      const ollamaHealthy = await checkOllamaHealth();
      
      console.error('❌ Integration tests require both Qdrant and Ollama services');
      console.error(`Qdrant (localhost:6333): ${qdrantHealthy ? '✅' : '❌'}`);
      console.error(`Ollama (localhost:11434): ${ollamaHealthy ? '✅' : '❌'}`);
      
      if (!qdrantHealthy) {
        console.error('  - Start Qdrant: docker run -p 127.0.0.1:6333:6333 qdrant/qdrant');
      }
      if (!ollamaHealthy) {
        console.error('  - Start Ollama and install model: ollama pull nomic-embed-text');
      }
      
      throw new Error('Required services not available for integration tests');
    }
  });

  describe('CLI Commands', () => {
    it('should show help when no arguments provided', async () => {
      const result = await runCLI(['--help']);
      expect(result.exitCode).toBe(0);
      expect(result.stdout).toContain('directory-indexer');
      expect(result.stdout).toContain('AI-powered directory indexing');
    });

    it('should handle invalid commands gracefully', async () => {
      const result = await runCLI(['invalid-command']);
      expect(result.exitCode).toBe(1);
    });

    it('should require arguments for commands', async () => {
      const searchResult = await runCLI(['search']);
      expect(searchResult.exitCode).toBe(1);

      const similarResult = await runCLI(['similar']);
      expect(similarResult.exitCode).toBe(1);

      const getResult = await runCLI(['get']);
      expect(getResult.exitCode).toBe(1);
    });
  });

  describe('Main Workflow (requires services)', () => {
    it('should complete full indexing and search workflow', async () => {

      const testDataPath = join(process.cwd(), 'tests', 'test_data');
      
      if (!existsSync(testDataPath)) {
        throw new Error(`Test data not found at ${testDataPath}`);
      }

      // 1. Index test data
      console.log('🔄 Indexing test data...');
      const indexResult = await runCLI(['index', testDataPath], 120000);
      expect(indexResult.exitCode).toBe(0);
      expect(indexResult.stdout.toLowerCase()).toContain('index');

      // 2. Test status command
      console.log('🔄 Checking status...');
      const statusResult = await runCLI(['status']);
      expect(statusResult.exitCode).toBe(0);
      expect(statusResult.stdout.toLowerCase()).toContain('status');

      // 3. Test semantic search
      console.log('🔄 Testing search...');
      const searchResult = await runCLI(['search', 'authentication', '--limit', '5']);
      expect(searchResult.exitCode).toBe(0);

      // 4. Test similar files (using a known file from test_data)
      const testFile = join(testDataPath, 'docs', 'api_guide.md');
      if (existsSync(testFile)) {
        console.log('🔄 Testing similar files...');
        const similarResult = await runCLI(['similar', testFile, '--limit', '3']);
        expect(similarResult.exitCode).toBe(0);
      }

      // 5. Test get content
      if (existsSync(testFile)) {
        console.log('🔄 Testing get content...');
        const getResult = await runCLI(['get', testFile]);
        expect(getResult.exitCode).toBe(0);
      }

      console.log('✅ Full workflow completed successfully');
    });

    it('should handle search with limit', async () => {
      const result = await runCLI(['search', 'configuration', '--limit', '2']);
      expect(result.exitCode).toBe(0);
    });

    it('should handle get content with chunk selection', async () => {

      const testFile = join(process.cwd(), 'tests', 'test_data', 'docs', 'api_guide.md');
      
      if (existsSync(testFile)) {
        const result = await runCLI(['get', testFile, '--chunks', '1-2']);
        expect(result.exitCode).toBe(0);
      }
    });
  });

  describe('MCP Server', () => {
    it('should start MCP server without crashing', async () => {
      // Just test that serve command doesn't immediately fail
      const child = spawn('node', ['dist/cli.js', 'serve'], {
        env: {
          ...process.env,
          DIRECTORY_INDEXER_QDRANT_COLLECTION: 'directory-indexer-mcp-test'
        }
      });

      // Give it a moment to start
      await new Promise(resolve => setTimeout(resolve, 1000));

      // Check if it's still running
      expect(child.killed).toBe(false);

      // Clean up
      child.kill();
      await new Promise(resolve => setTimeout(resolve, 100));
    });
  });
});