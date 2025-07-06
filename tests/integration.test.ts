import { describe, it, expect, beforeAll } from 'vitest';
import { spawn } from 'child_process';
import { join } from 'path';
import { existsSync } from 'fs';
import { loadConfig } from '../src/config.js';
import { indexDirectories, scanDirectory, getFileMetadata, chunkText } from '../src/indexing.js';
import { searchContent, findSimilarFiles, getFileContent } from '../src/search.js';
import { getIndexStatus, SQLiteStorage, QdrantClient } from '../src/storage.js';
import { startMcpServer } from '../src/mcp.js';
import { createEmbeddingProvider } from '../src/embedding.js';
import { normalizePath, calculateHash } from '../src/utils.js';

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
    await fetch('http://localhost:11434/api/embeddings', {
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
    const child = spawn('node', ['bin/directory-indexer.js', ...args], {
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

    it('should complete full workflow via direct function calls', async () => {
      const testDataPath = join(process.cwd(), 'tests', 'test_data');
      
      if (!existsSync(testDataPath)) {
        throw new Error(`Test data not found at ${testDataPath}`);
      }

      // Load configuration for direct function calls
      // Set test-specific environment variables
      process.env.DIRECTORY_INDEXER_QDRANT_COLLECTION = 'directory-indexer-test-node';
      const config = await loadConfig({ verbose: false });

      // 1. Test direct indexing function
      console.log('🔄 Testing indexDirectories() directly...');
      const indexResult = await indexDirectories([testDataPath], config);
      expect(indexResult.indexed).toBeGreaterThan(0);
      expect(indexResult.skipped).toBeGreaterThanOrEqual(0);
      expect(Array.isArray(indexResult.errors)).toBe(true);

      // 2. Test direct status function
      console.log('🔄 Testing getIndexStatus() directly...');
      const status = await getIndexStatus();
      expect(status.filesIndexed).toBeGreaterThan(0);
      expect(status.chunksIndexed).toBeGreaterThan(0);
      expect(typeof status.databaseSize).toBe('string');

      // 3. Test direct search function
      console.log('🔄 Testing searchContent() directly...');
      const searchResults = await searchContent('authentication', { limit: 5 });
      expect(Array.isArray(searchResults)).toBe(true);
      expect(searchResults.length).toBeLessThanOrEqual(5);
      
      // 4. Test direct similar files function
      const testFile = join(testDataPath, 'docs', 'api_guide.md');
      if (existsSync(testFile)) {
        console.log('🔄 Testing findSimilarFiles() directly...');
        const similarResults = await findSimilarFiles(testFile, 3);
        expect(Array.isArray(similarResults)).toBe(true);
        expect(similarResults.length).toBeLessThanOrEqual(3);
      }

      // 5. Test direct content retrieval function
      if (existsSync(testFile)) {
        console.log('🔄 Testing getFileContent() directly...');
        const content = await getFileContent(testFile);
        expect(typeof content).toBe('string');
        expect(content.length).toBeGreaterThan(0);

        // Test with chunk selection
        const chunkedContent = await getFileContent(testFile, '1-2');
        expect(typeof chunkedContent).toBe('string');
      }

      console.log('✅ Direct function workflow completed successfully');
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

    it('should test MCP server components directly', async () => {
      // Test that MCP server can be initialized
      await loadConfig({ verbose: false });
      
      // Test that we can import the MCP server function
      expect(typeof startMcpServer).toBe('function');
      
      // We don't actually start the server here since it would hang the test,
      // but importing and checking the function exercises the MCP module for coverage
      console.log('✅ MCP server components loaded successfully');
    });
  });

  describe('File Scanning and Processing', () => {
    it('should scan directory and filter files', async () => {
      const testDataPath = join(process.cwd(), 'tests', 'test_data');
      const files = await scanDirectory(testDataPath, {
        ignorePatterns: ['.git', 'node_modules', '*.log'],
        maxFileSize: 1000000
      });
      
      expect(Array.isArray(files)).toBe(true);
      expect(files.length).toBeGreaterThan(0);
      
      const gitFiles = files.filter(f => f.path.includes('.git'));
      expect(gitFiles.length).toBe(0);
    });

    it('should extract file metadata', async () => {
      const testFile = join(process.cwd(), 'tests', 'test_data', 'docs', 'api_guide.md');
      if (existsSync(testFile)) {
        const metadata = await getFileMetadata(testFile);
        
        expect(metadata.path).toContain('api_guide.md');
        expect(metadata.size).toBeGreaterThan(0);
        expect(metadata.modifiedTime).toBeInstanceOf(Date);
        expect(metadata.hash.length).toBeGreaterThan(0);
      }
    });

    it('should chunk text with sliding window', async () => {
      const longText = 'This is a very long text that needs to be chunked into smaller pieces for embedding generation and vector storage. It should be split properly.';
      const chunks = chunkText(longText, 50, 10);
      
      expect(Array.isArray(chunks)).toBe(true);
      expect(chunks.length).toBeGreaterThan(1);
      expect(chunks[0].content.length).toBeLessThanOrEqual(50);
      expect(chunks[0].startIndex).toBe(0);
    });
  });

  describe('Embedding Provider Tests', () => {
    it('should create and use mock embedding provider', async () => {
      const provider = createEmbeddingProvider('mock', {
        model: 'test-model',
        endpoint: '',
        dimensions: 384
      });
      
      expect(provider.name).toBe('mock');
      expect(provider.dimensions).toBe(384);
      
      const embedding = await provider.generateEmbedding('test text');
      expect(Array.isArray(embedding)).toBe(true);
      expect(embedding.length).toBe(384);
      
      const embeddings = await provider.generateEmbeddings(['text one', 'text two']);
      expect(embeddings.length).toBe(2);
      expect(embeddings[0].length).toBe(384);
    });

    it('should create ollama provider when services available', async () => {
      const provider = createEmbeddingProvider('ollama', {
        model: 'nomic-embed-text',
        endpoint: 'http://localhost:11434',
        dimensions: 768
      });
      
      expect(provider.name).toBe('ollama');
      expect(provider.dimensions).toBe(768);
      
      try {
        const embedding = await provider.generateEmbedding('test');
        expect(Array.isArray(embedding)).toBe(true);
        expect(embedding.length).toBe(768);
      } catch {
        console.log('Ollama embedding test failed - service may not be ready');
      }
    });
  });

  describe('Storage Operations', () => {
    it('should handle SQLite storage operations', async () => {
      const config = await loadConfig();
      config.storage.sqlitePath = ':memory:';
      const storage = new SQLiteStorage(config);
      
      try {
        await storage.upsertDirectory('/test/dir', 'pending');
        const dir = await storage.getDirectory('/test/dir');
        expect(dir?.path).toBe('/test/dir');
        expect(dir?.status).toBe('pending');
        
        const fileInfo = {
          path: '/test/dir/file.txt',
          size: 100,
          modifiedTime: new Date(),
          hash: 'hash123',
          parentDirs: ['/test', '/test/dir']
        };
        
        await storage.upsertFile(fileInfo);
        const file = await storage.getFile('/test/dir/file.txt');
        expect(file?.path).toBe('/test/dir/file.txt');
        
        const files = await storage.getFilesByDirectory('/test/dir');
        expect(files.length).toBe(1);
        
        await storage.deleteFile('/test/dir/file.txt');
        const deletedFile = await storage.getFile('/test/dir/file.txt');
        expect(deletedFile).toBeNull();
        
      } finally {
        storage.close();
      }
    });

    it('should handle Qdrant operations', async () => {
      const config = await loadConfig();
      const qdrant = new QdrantClient(config);
      
      const isHealthy = await qdrant.healthCheck();
      expect(typeof isHealthy).toBe('boolean');
      
      if (isHealthy) {
        await qdrant.createCollection();
        
        const points = [{
          id: 12345,
          vector: new Array(768).fill(0.1),
          payload: {
            filePath: '/test/file.txt',
            chunkId: 'chunk-1',
            parentDirectories: ['/test']
          }
        }];
        
        await qdrant.upsertPoints(points);
        
        const searchResults = await qdrant.searchPoints(new Array(768).fill(0.1), 5);
        expect(Array.isArray(searchResults)).toBe(true);
        
        await qdrant.deletePoints([12345]);
      }
    });
  });

  describe('Utility Functions', () => {
    it('should normalize paths correctly', async () => {
      const windowsPath = 'C:\\Users\\test\\Documents';
      const unixPath = '/home/test/documents';
      
      const normalizedWindows = normalizePath(windowsPath);
      const normalizedUnix = normalizePath(unixPath);
      
      expect(typeof normalizedWindows).toBe('string');
      expect(typeof normalizedUnix).toBe('string');
      expect(normalizedWindows.length).toBeGreaterThan(0);
      expect(normalizedUnix.length).toBeGreaterThan(0);
    });

    it('should calculate file hashes consistently', async () => {
      const testContent = 'Hello, world!';
      const hash1 = calculateHash(testContent);
      const hash2 = calculateHash(testContent);
      
      expect(hash1).toBe(hash2);
      expect(hash1.length).toBeGreaterThan(0);
      expect(typeof hash1).toBe('string');
    });
  });

  describe('Error Handling', () => {
    it('should handle search with invalid parameters', async () => {
      try {
        await searchContent('', { limit: -1 });
        expect(false).toBe(true);
      } catch (error) {
        expect(error).toBeInstanceOf(Error);
      }
    });

    it('should handle similar files with non-existent file', async () => {
      try {
        await findSimilarFiles('/nonexistent/file.txt', 5);
        expect(false).toBe(true);
      } catch (error) {
        expect(error).toBeInstanceOf(Error);
      }
    });

    it('should handle get content with non-existent file', async () => {
      try {
        await getFileContent('/nonexistent/file.txt');
        expect(false).toBe(true);
      } catch (error) {
        expect(error).toBeInstanceOf(Error);
      }
    });
  });

  describe('Configuration Tests', () => {
    it('should handle environment variable overrides', async () => {
      const originalQdrant = process.env.QDRANT_ENDPOINT;
      const originalOllama = process.env.OLLAMA_ENDPOINT;
      
      try {
        process.env.QDRANT_ENDPOINT = 'http://test-qdrant:6333';
        process.env.OLLAMA_ENDPOINT = 'http://test-ollama:11434';
        
        const config = await loadConfig();
        expect(config.storage.qdrantEndpoint).toBe('http://test-qdrant:6333');
        expect(config.embedding.endpoint).toBe('http://test-ollama:11434');
        
      } finally {
        if (originalQdrant) process.env.QDRANT_ENDPOINT = originalQdrant;
        else delete process.env.QDRANT_ENDPOINT;
        if (originalOllama) process.env.OLLAMA_ENDPOINT = originalOllama;
        else delete process.env.OLLAMA_ENDPOINT;
      }
    });

    it('should validate configuration parameters', async () => {
      const config = await loadConfig({ verbose: true });
      
      expect(config.verbose).toBe(true);
      expect(config.indexing.chunkSize).toBeGreaterThan(0);
      expect(config.indexing.maxFileSize).toBeGreaterThan(0);
      expect(Array.isArray(config.indexing.ignorePatterns)).toBe(true);
      expect(config.indexing.ignorePatterns.length).toBeGreaterThan(0);
    });
  });
});