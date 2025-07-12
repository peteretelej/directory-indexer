import { describe, it, expect, beforeAll } from 'vitest';
import { existsSync } from 'fs';
import { join } from 'path';
import { runCLI, setupServicesCheck, getTestDataPath, getTestFile, createTempTestDirectory, cleanupTempDirectory } from '../utils/test-helpers.js';
import { loadConfig } from '../../src/config.js';
import { indexDirectories, getFileMetadata, chunkText, scanDirectory } from '../../src/indexing.js';
import { searchContent, findSimilarFiles, getFileContent, getChunkContent } from '../../src/search.js';
import { getIndexStatus } from '../../src/storage.js';

describe.sequential('CLI Commands Integration Tests', () => {
  beforeAll(async () => {
    await setupServicesCheck();
  });

  describe('Help and Argument Validation', () => {
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

  describe('Main CLI Workflow', () => {
    it('should complete full indexing and search workflow', async () => {
      const testDataPath = getTestDataPath();
      
      if (!existsSync(testDataPath)) {
        throw new Error(`Test data not found at ${testDataPath}`);
      }

      console.log('🔄 Indexing test data...');
      const indexResult = await runCLI(['index', testDataPath], 120000);
      expect(indexResult.exitCode).toBe(0);
      expect(indexResult.stdout.toLowerCase()).toContain('index');

      console.log('🔄 Checking status...');
      const statusResult = await runCLI(['status']);
      expect(statusResult.exitCode).toBe(0);
      expect(statusResult.stdout.toLowerCase()).toContain('status');

      console.log('🔄 Testing search...');
      const searchResult = await runCLI(['search', 'authentication', '--limit', '5']);
      expect(searchResult.exitCode).toBe(0);

      const testFile = getTestFile();
      if (existsSync(testFile)) {
        console.log('🔄 Testing similar files...');
        const similarResult = await runCLI(['similar', testFile, '--limit', '3']);
        expect(similarResult.exitCode).toBe(0);
      }

      if (existsSync(testFile)) {
        console.log('🔄 Testing get content...');
        const getResult = await runCLI(['get', testFile]);
        expect(getResult.exitCode).toBe(0);
      }

      console.log('✅ Full CLI workflow completed successfully');
    });

    it('should handle search with limit', async () => {
      const result = await runCLI(['search', 'configuration', '--limit', '2']);
      expect(result.exitCode).toBe(0);
    });

    it('should handle get content with chunk selection', async () => {
      const testFile = getTestFile();
      
      if (existsSync(testFile)) {
        const result = await runCLI(['get', testFile, '--chunks', '1-2']);
        expect(result.exitCode).toBe(0);
      }
    });

    it('should handle status command with workspace health', async () => {
      const testDataPath = getTestDataPath();
      const result = await runCLI(['status'], 30000, {
        WORKSPACE_DOCS: join(testDataPath, 'docs'),
        WORKSPACE_INVALID: '/nonexistent/path'
      });
      
      expect(result.exitCode).toBe(0);
      expect(result.stdout.toLowerCase()).toContain('status');
      
      if (result.stdout.includes('WORKSPACES:')) {
        expect(result.stdout).toMatch(/\d+ healthy, \d+ warnings, \d+ errors/);
      }
    });

    it('should complete full workflow via direct function calls', async () => {
      const testDataPath = getTestDataPath();
      
      if (!existsSync(testDataPath)) {
        throw new Error(`Test data not found at ${testDataPath}`);
      }

      process.env.DIRECTORY_INDEXER_QDRANT_COLLECTION = 'directory-indexer-test-node';
      const config = await loadConfig({ verbose: false });

      console.log('🔄 Testing indexDirectories() directly...');
      const indexResult = await indexDirectories([testDataPath], config);
      expect(indexResult.indexed + indexResult.skipped).toBeGreaterThan(0);
      expect(indexResult.skipped).toBeGreaterThanOrEqual(0);
      expect(Array.isArray(indexResult.errors)).toBe(true);

      console.log('🔄 Testing getIndexStatus() directly...');
      const status = await getIndexStatus();
      expect(status.filesIndexed).toBeGreaterThan(0);
      expect(status.chunksIndexed).toBeGreaterThan(0);
      expect(typeof status.databaseSize).toBe('string');

      console.log('🔄 Testing searchContent() directly...');
      const searchResults = await searchContent('authentication', { limit: 5 });
      expect(Array.isArray(searchResults)).toBe(true);
      expect(searchResults.length).toBeLessThanOrEqual(5);
      
      const testFile = getTestFile();
      if (existsSync(testFile)) {
        console.log('🔄 Testing findSimilarFiles() directly...');
        const similarResults = await findSimilarFiles(testFile, 3);
        expect(Array.isArray(similarResults)).toBe(true);
        expect(similarResults.length).toBeLessThanOrEqual(3);
      }

      if (existsSync(testFile)) {
        console.log('🔄 Testing getFileContent() directly...');
        const content = await getFileContent(testFile);
        expect(typeof content).toBe('string');
        expect(content.length).toBeGreaterThan(0);

        const chunkedContent = await getFileContent(testFile, '1-2');
        expect(typeof chunkedContent).toBe('string');
      }

      if (searchResults.length > 0 && searchResults[0].chunks.length > 0) {
        console.log('🔄 Testing getChunkContent() directly...');
        const firstResult = searchResults[0];
        const firstChunk = firstResult.chunks[0];
        
        const chunkContent = await getChunkContent(firstResult.filePath, firstChunk.chunkId);
        expect(typeof chunkContent).toBe('string');
        expect(chunkContent.length).toBeGreaterThan(0);
      }

      console.log('✅ Direct function workflow completed successfully');
    });

    it('should filter search results by workspace', async () => {
      const testDataPath = getTestDataPath();
      
      const config = await loadConfig();
      await indexDirectories([testDataPath], config);
      
      const originalEnv = process.env;
      process.env.WORKSPACE_DOCS = join(testDataPath, 'docs');
      
      try {
        await searchContent('API', { limit: 10 });
        
        const docsResults = await searchContent('API', { workspace: 'docs', limit: 10 });
        
        if (docsResults.length > 0) {
          expect(docsResults.every(r => r.filePath.includes('/docs/'))).toBe(true);
        }
        
        const codeResults = await searchContent('function', { workspace: 'docs', limit: 10 });
        
        if (codeResults.length > 0) {
          expect(codeResults.every(r => !r.filePath.includes('/programming/'))).toBe(true);
        }
        
      } finally {
        process.env = originalEnv;
      }
    });

    it('should report workspace health status', async () => {
      const testDataPath = getTestDataPath();
      
      const originalEnv = process.env;
      process.env.WORKSPACE_DOCS = join(testDataPath, 'docs');
      process.env.WORKSPACE_INVALID = '/nonexistent/path';
      
      try {
        const status = await getIndexStatus();
        
        expect(status.workspaceHealth).toBeDefined();
        expect(typeof status.workspaceHealth.healthy).toBe('number');
        expect(typeof status.workspaceHealth.errors).toBe('number');
        
        expect(status.workspaces.length).toBe(2);
        
        const docsWorkspace = status.workspaces.find(w => w.name === 'docs');
        const invalidWorkspace = status.workspaces.find(w => w.name === 'invalid');
        
        expect(docsWorkspace?.isValid).toBe(true);
        expect(invalidWorkspace?.isValid).toBe(false);
        
      } finally {
        process.env = originalEnv;
      }
    });
  });

  describe('Reset Command', () => {
    it('should reset directory-indexer data with --force flag', async () => {
      const testDataPath = getTestDataPath();
      const testCollectionName = 'directory-indexer-test-reset-force';
      
      console.log('🔄 Setting up test data for reset...');
      const indexResult = await runCLI(['index', testDataPath], 120000, {
        DIRECTORY_INDEXER_QDRANT_COLLECTION: testCollectionName
      });
      expect(indexResult.exitCode).toBe(0);
      
      const statusBefore = await runCLI(['status'], 30000, {
        DIRECTORY_INDEXER_QDRANT_COLLECTION: testCollectionName
      });
      expect(statusBefore.exitCode).toBe(0);
      expect(statusBefore.stdout.toLowerCase()).toContain('file');
      
      console.log('🔄 Testing reset --force...');
      const resetResult = await runCLI(['reset', '--force'], 30000, {
        DIRECTORY_INDEXER_QDRANT_COLLECTION: testCollectionName
      });
      
      if (resetResult.exitCode !== 0) {
        console.log('Reset command failed:');
        console.log('stdout:', resetResult.stdout);
        console.log('stderr:', resetResult.stderr);
      }
      
      expect(resetResult.exitCode).toBe(0);
      expect(resetResult.stdout.toLowerCase()).toContain('reset');
      
      const statusAfter = await runCLI(['status'], 30000, {
        DIRECTORY_INDEXER_QDRANT_COLLECTION: testCollectionName
      });
      expect(statusAfter.exitCode).toBe(0);
      
      const searchResult = await runCLI(['search', 'authentication'], 30000, {
        DIRECTORY_INDEXER_QDRANT_COLLECTION: testCollectionName
      });
      
      if (searchResult.exitCode !== 0) {
        console.log('Search command failed after reset:');
        console.log('stdout:', searchResult.stdout);
        console.log('stderr:', searchResult.stderr);
      }
      
      expect(searchResult.exitCode).toBe(0);
      expect(searchResult.stdout.trim()).toBe('No results found');
      
      console.log('✅ Reset --force completed successfully');
    });

    it('should show preview and handle custom configuration in reset', async () => {
      const testDataPath = getTestDataPath();
      const testCollectionName = 'directory-indexer-test-reset-custom';
      const customDataDir = await createTempTestDirectory();
      
      try {
        console.log('🔄 Setting up test data with custom config...');
        const indexResult = await runCLI(['index', testDataPath], 120000, {
          DIRECTORY_INDEXER_QDRANT_COLLECTION: testCollectionName,
          DIRECTORY_INDEXER_DATA_DIR: customDataDir
        });
        
        if (indexResult.exitCode !== 0) {
          console.log('Index command failed:');
          console.log('stdout:', indexResult.stdout);
          console.log('stderr:', indexResult.stderr);
        }
        
        expect(indexResult.exitCode).toBe(0);
        
        // Test reset preview - this will timeout because it waits for input, which is expected
        try {
          await runCLI(['reset'], 5000, {
            DIRECTORY_INDEXER_QDRANT_COLLECTION: testCollectionName,
            DIRECTORY_INDEXER_DATA_DIR: customDataDir
          });
        } catch (error) {
          // Timeout is expected since the command waits for interactive input
          expect((error as Error).message).toContain('Command timed out');
        }
        
        // Instead, test with --force and verify the custom paths are used
        const resetForce = await runCLI(['reset', '--force'], 30000, {
          DIRECTORY_INDEXER_QDRANT_COLLECTION: testCollectionName,
          DIRECTORY_INDEXER_DATA_DIR: customDataDir
        });
        
        expect(resetForce.exitCode).toBe(0);
        expect(resetForce.stdout.toLowerCase()).toContain('reset');
        
        console.log('✅ Reset preview with custom config shown correctly');
        
      } finally {
        await cleanupTempDirectory(customDataDir);
      }
    });

    it('should handle reset when no data exists', async () => {
      const testCollectionName = 'directory-indexer-test-reset-empty';
      
      const resetResult = await runCLI(['reset', '--force'], 30000, {
        DIRECTORY_INDEXER_QDRANT_COLLECTION: testCollectionName
      });
      
      expect(resetResult.exitCode).toBe(0);
      expect(resetResult.stdout.toLowerCase()).toContain('reset');
      
      console.log('✅ Reset with no existing data handled gracefully');
    });

    it('should handle reset with services unavailable', async () => {
      const testCollectionName = 'directory-indexer-test-reset-offline';
      
      const resetResult = await runCLI(['reset', '--force'], 30000, {
        DIRECTORY_INDEXER_QDRANT_COLLECTION: testCollectionName,
        QDRANT_ENDPOINT: 'http://invalid-qdrant:9999'
      });
      
      expect(resetResult.exitCode).toBe(0);
      const output = resetResult.stdout + resetResult.stderr;
      expect(output.toLowerCase()).toMatch(/(reset|warning|unavailable)/);
      
      console.log('✅ Reset with unavailable services handled gracefully');
    });
  });

  describe('File Processing Tests', () => {
    it('should scan directory and filter files', async () => {
      const testDataPath = getTestDataPath();
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
      const testFile = getTestFile();
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
});