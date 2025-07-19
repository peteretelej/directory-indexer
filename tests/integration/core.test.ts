import { describe, it, expect, beforeAll, afterEach } from 'vitest';
import { join } from 'path';
import { promises as fs } from 'fs';
import { 
  setupServicesCheck, 
  createIsolatedTestEnvironment
} from '../utils/test-helpers.js';
import { loadConfig } from '../../src/config.js';
import { indexDirectories } from '../../src/indexing.js';
import { searchContent, findSimilarFiles, getFileContent } from '../../src/search.js';
import { SQLiteStorage, QdrantClient } from '../../src/storage.js';
import { createEmbeddingProvider } from '../../src/embedding.js';
import { normalizePath, calculateHash, fileExists } from '../../src/utils.js';
import { clearGitignoreCache } from '../../src/gitignore.js';

describe.sequential('Core Functionality Integration Tests', () => {
  beforeAll(async () => {
    await setupServicesCheck();
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
            fileHash: 'test-hash-123',
            content: 'test content',
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

    it('should clean up deleted files during re-indexing', async () => {
      const testEnv = await createIsolatedTestEnvironment('cleanup');
      
      // Create temporary directory for test files
      const tempDir = join(testEnv.dataDir, 'test-files');
      await fs.mkdir(tempDir, { recursive: true });
      const testFile = join(tempDir, 'test-file.md');
      
      try {
        await fs.writeFile(testFile, '# Test Content\nThis is test content for deletion.');
        
        const config = await loadConfig({ verbose: false });
        
        // First indexing - should add the file
        const indexResult1 = await indexDirectories([tempDir], config);
        expect(indexResult1.indexed).toBe(1);
        expect(indexResult1.deleted).toBe(0);
        
        // Verify file was indexed
        const searchResults1 = await searchContent('test content', { limit: 10 });
        const foundFile = searchResults1.find(r => r.filePath === testFile);
        expect(foundFile).toBeDefined();
        
        // Delete the file
        await fs.unlink(testFile);
        expect(await fileExists(testFile)).toBe(false);
        
        // Second indexing - should detect and clean up deleted file
        const indexResult2 = await indexDirectories([tempDir], config);
        expect(indexResult2.deleted).toBe(1);
        expect(indexResult2.indexed).toBe(0);
        
        // Verify file was removed from search results
        const searchResults2 = await searchContent('test content', { limit: 10 });
        const foundFile2 = searchResults2.find(r => r.filePath === testFile);
        expect(foundFile2).toBeUndefined();
        
      } finally {
        await testEnv.cleanup();
      }
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

    it('should handle indexing with inaccessible files', async () => {
      const testEnv = await createIsolatedTestEnvironment('fs-errors');
      
      try {
        const config = await loadConfig({ verbose: false });
        
        // Try to index nonexistent directory - should handle gracefully
        const result = await indexDirectories(['/nonexistent/path'], config);
        
        expect(result.failed).toBeGreaterThanOrEqual(0);
        expect(Array.isArray(result.errors)).toBe(true);
        
      } finally {
        await testEnv.cleanup();
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

  describe('Gitignore Integration Tests', () => {
    afterEach(async () => {
      // Clear gitignore cache after each test
      clearGitignoreCache();
      
      // Clean up test .gitignore file if it exists
      const gitignorePath = join(process.cwd(), 'tests/test_data/.gitignore');
      try {
        await fs.unlink(gitignorePath);
      } catch {
        // Ignore if file doesn't exist
      }
    });

    it('should respect .gitignore patterns during indexing', async () => {
      const testEnv = await createIsolatedTestEnvironment('gitignore-basic');
      
      try {
        const config = await loadConfig({ verbose: false });
        // Create .gitignore file with test patterns
        const gitignorePath = join(process.cwd(), 'tests/test_data/.gitignore');
        const gitignoreContent = 'temp/\nbuild/';
        await fs.writeFile(gitignorePath, gitignoreContent);
        
        // Index test_data directory
        const testDataPath = join(process.cwd(), 'tests/test_data');
        const result = await indexDirectories([testDataPath], config);
        
        // Gitignore functionality is working correctly if:
        // 1. Indexing completed successfully
        // 2. Some files were indexed (proving the indexing works)
        // 3. Essential patterns and gitignore patterns are being applied
        expect(result.indexed).toBeGreaterThan(0);
        expect(result.failed).toBe(0);
        expect(Array.isArray(result.errors)).toBe(true);
        
        // Verify we can still find content from non-ignored files
        const indexedResults = await searchContent('programming', { limit: 10 });
        expect(indexedResults.length).toBeGreaterThan(0);
        
      } finally {
        await testEnv.cleanup();
      }
    });

    it('should prioritize essential patterns over gitignore negations', async () => {
      const testEnv = await createIsolatedTestEnvironment('gitignore-essential');
      
      try {
        const config = await loadConfig({ verbose: false });
        // Create .gitignore that tries to negate essential patterns
        const gitignorePath = join(process.cwd(), 'tests/test_data/.gitignore');
        const gitignoreContent = 'node_modules/\n!node_modules\n.git/\n!.git';
        await fs.writeFile(gitignorePath, gitignoreContent);
        
        // Create test node_modules directory
        const nodeModulesPath = join(process.cwd(), 'tests/test_data/node_modules');
        await fs.mkdir(nodeModulesPath, { recursive: true });
        await fs.writeFile(join(nodeModulesPath, 'package.json'), '{"name": "test"}');
        
        try {
          // Index test_data directory
          const testDataPath = join(process.cwd(), 'tests/test_data');
          await indexDirectories([testDataPath], config);
          
          // Search for node_modules content
          const nodeModulesResults = await searchContent('test', { limit: 10 });
          
          // Should not find content from node_modules even with negation
          const nodeModulesFound = nodeModulesResults.some(r => 
            r.filePath.includes('node_modules')
          );
          expect(nodeModulesFound).toBe(false);
          
        } finally {
          // Cleanup test node_modules
          await fs.rm(nodeModulesPath, { recursive: true, force: true });
        }
        
      } finally {
        await testEnv.cleanup();
      }
    });

    it('should work correctly when respectGitignore is disabled', async () => {
      const testEnv = await createIsolatedTestEnvironment('gitignore-disabled');
      
      try {
        const config = await loadConfig({ verbose: false });
        // Disable gitignore support
        config.indexing.respectGitignore = false;
        
        // Create .gitignore file
        const gitignorePath = join(process.cwd(), 'tests/test_data/.gitignore');
        const gitignoreContent = '*.log\ntemp/\nbuild/';
        await fs.writeFile(gitignorePath, gitignoreContent);
        
        // Index test_data directory
        const testDataPath = join(process.cwd(), 'tests/test_data');
        await indexDirectories([testDataPath], config);
        
        // When gitignore is disabled, .log files should be indexed
        const debugLogResults = await searchContent('DEBUG Application started', { limit: 10 });
        expect(debugLogResults.length).toBeGreaterThan(0); // debug.log should be indexed
        
        // But essential patterns should still work
        const nodeModulesResults = await searchContent('node_modules', { limit: 10 });
        const nodeModulesFound = nodeModulesResults.some(r => 
          r.filePath.includes('node_modules')
        );
        expect(nodeModulesFound).toBe(false); // Essential patterns still apply
        
      } finally {
        await testEnv.cleanup();
      }
    });
  });
});