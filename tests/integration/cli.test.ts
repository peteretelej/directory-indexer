import { describe, it, expect, beforeAll } from 'vitest';
import { existsSync } from 'fs';
import { join } from 'path';
import { 
  runCLI, 
  runCLIWithLogging, 
  expectCLISuccess, 
  setupServicesCheck, 
  getTestDataPath, 
  getTestFile, 
  createIsolatedTestEnvironment
} from '../utils/test-helpers.js';
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

      const testEnv = await createIsolatedTestEnvironment('main-workflow');
      
      try {
        console.log('🔄 Indexing test data...');
        const indexResult = await runCLIWithLogging(['index', testDataPath], testEnv.env, 120000);
        expectCLISuccess(indexResult);
        expect(indexResult.stdout.toLowerCase()).toContain('index');

        // Test progress reporting messages
        expect(indexResult.stdout).toMatch(/Found \d+ files to process/);
        expect(indexResult.stdout).toMatch(/Indexed \d+ files, skipped \d+ files/);
        expect(indexResult.stdout).toContain('Run with --verbose for detailed per-file indexing reports');
        expect(indexResult.stdout).toContain('Indexing can be safely stopped and resumed');

        console.log('🔄 Checking status...');
        const statusResult = await runCLIWithLogging(['status'], testEnv.env);
        expectCLISuccess(statusResult);
        expect(statusResult.stdout.toLowerCase()).toContain('status');

        console.log('🔄 Testing search...');
        const searchResult = await runCLIWithLogging(['search', 'authentication', '--limit', '5'], testEnv.env);
        expectCLISuccess(searchResult);

        const testFile = getTestFile();
        if (existsSync(testFile)) {
          console.log('🔄 Testing similar files...');
          const similarResult = await runCLIWithLogging(['similar', testFile, '--limit', '3'], testEnv.env);
          expectCLISuccess(similarResult);
        }

        if (existsSync(testFile)) {
          console.log('🔄 Testing get content...');
          const getResult = await runCLIWithLogging(['get', testFile], testEnv.env);
          expectCLISuccess(getResult);
        }

        console.log('✅ Full CLI workflow completed successfully');
        
      } finally {
        await testEnv.cleanup();
      }
    });

    it('should handle search with limit', async () => {
      const testDataPath = getTestDataPath();
      const testEnv = await createIsolatedTestEnvironment('search-limit');
      
      try {
        // Set up test data first
        await runCLIWithLogging(['index', testDataPath], testEnv.env, 120000);
        
        // Then test search with limit
        const result = await runCLIWithLogging(['search', 'configuration', '--limit', '2'], testEnv.env);
        expectCLISuccess(result);
      } finally {
        await testEnv.cleanup();
      }
    });

    it('should handle get content with chunk selection', async () => {
      const testFile = getTestFile();
      
      if (!existsSync(testFile)) {
        return; // Skip if test file doesn't exist
      }
      
      const testDataPath = getTestDataPath();
      const testEnv = await createIsolatedTestEnvironment('get-content');
      
      try {
        // Set up test data first  
        await runCLIWithLogging(['index', testDataPath], testEnv.env, 120000);
        
        // Then test get content with chunks
        const result = await runCLIWithLogging(['get', testFile, '--chunks', '1-2'], testEnv.env);
        expectCLISuccess(result);
      } finally {
        await testEnv.cleanup();
      }
    });

    it('should handle status command with workspace health', async () => {
      const testDataPath = getTestDataPath();
      const testEnv = await createIsolatedTestEnvironment('status-workspace');
      
      try {
        // Set up test data first
        await runCLIWithLogging(['index', testDataPath], testEnv.env, 120000);
        
        // Then test status with workspace configuration
        const result = await runCLIWithLogging(['status'], {
          ...testEnv.env,
          WORKSPACE_DOCS: join(testDataPath, 'docs'),
          WORKSPACE_INVALID: '/nonexistent/path'
        });
        
        expectCLISuccess(result);
        expect(result.stdout.toLowerCase()).toContain('status');
        
        if (result.stdout.includes('WORKSPACES:')) {
          expect(result.stdout).toMatch(/\d+ healthy, \d+ warnings, \d+ errors/);
        }
      } finally {
        await testEnv.cleanup();
      }
    });

    it('should complete full workflow via direct function calls', async () => {
      const testDataPath = getTestDataPath();
      if (!existsSync(testDataPath)) {
        throw new Error(`Test data not found at ${testDataPath}`);
      }

      const testEnv = await createIsolatedTestEnvironment('direct-functions');
      
      try {
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
        
      } finally {
        await testEnv.cleanup();
      }
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
    it('should handle reset command with preview, force, and verification', async () => {
      const testDataPath = getTestDataPath();
      const testEnv = await createIsolatedTestEnvironment('reset');
      
      try {
        // Step 1: Set up test data
        console.log('🔄 Setting up test data...');
        const indexResult = await runCLIWithLogging(['index', testDataPath], testEnv.env, 120000);
        expectCLISuccess(indexResult);
        
        // Step 2: Verify data exists before reset
        const statusBefore = await runCLIWithLogging(['status'], testEnv.env);
        expectCLISuccess(statusBefore);
        expect(statusBefore.stdout.toLowerCase()).toContain('file');
        
        // Step 3: Test reset preview (expects timeout due to interactive prompt)
        console.log('🔄 Testing reset preview...');
        try {
          await runCLI(['reset'], 5000, testEnv.env);
        } catch (error) {
          // Timeout is expected since the command waits for interactive input
          expect((error as Error).message).toContain('Command timed out');
        }
        
        // Step 4: Test reset with --force flag
        console.log('🔄 Testing reset --force...');
        const resetResult = await runCLIWithLogging(['reset', '--force'], testEnv.env);
        expectCLISuccess(resetResult);
        expect(resetResult.stdout.toLowerCase()).toContain('reset');
        
        // Step 5: Verify reset was successful
        const statusAfter = await runCLIWithLogging(['status'], testEnv.env);
        expectCLISuccess(statusAfter);
        
        const searchResult = await runCLIWithLogging(['search', 'authentication'], testEnv.env);
        expectCLISuccess(searchResult);
        expect(searchResult.stdout.trim()).toBe('No results found');
        
        console.log('✅ Reset functionality verified successfully');
        
      } finally {
        await testEnv.cleanup();
      }
    });

    it('should handle reset when no data exists', async () => {
      const testEnv = await createIsolatedTestEnvironment('reset-empty');
      
      try {
        const resetResult = await runCLIWithLogging(['reset', '--force'], testEnv.env);
        expectCLISuccess(resetResult);
        expect(resetResult.stdout.toLowerCase()).toContain('reset');
        
        console.log('✅ Reset with no existing data handled gracefully');
      } finally {
        await testEnv.cleanup();
      }
    });

    it('should handle reset with services unavailable', async () => {
      const testEnv = await createIsolatedTestEnvironment('reset-offline');
      
      try {
        const resetResult = await runCLI(['reset', '--force'], 30000, {
          ...testEnv.env,
          QDRANT_ENDPOINT: 'http://invalid-qdrant:9999'
        });
        
        expect(resetResult.exitCode).toBe(0);
        const output = resetResult.stdout + resetResult.stderr;
        expect(output.toLowerCase()).toMatch(/(reset|warning|unavailable)/);
        
        console.log('✅ Reset with unavailable services handled gracefully');
      } finally {
        await testEnv.cleanup();
      }
    });
  });

  describe('Progress Reporting Tests', () => {
    it('should show progress messages during indexing', async () => {
      const testDataPath = getTestDataPath();
      const testEnv = await createIsolatedTestEnvironment('progress-reporting');
      
      try {
        const indexResult = await runCLIWithLogging(['index', testDataPath], testEnv.env, 120000);
        expectCLISuccess(indexResult);
        
        // Test progress messages
        expect(indexResult.stdout).toMatch(/Found \d+ files to process/);
        expect(indexResult.stdout).toMatch(/Indexed \d+ files, skipped \d+ files/);
        expect(indexResult.stdout).toContain('Run with --verbose for detailed per-file indexing reports');
        expect(indexResult.stdout).toContain('Indexing can be safely stopped and resumed');
      } finally {
        await testEnv.cleanup();
      }
    });

    it('should show detailed progress in verbose mode', async () => {
      const testDataPath = getTestDataPath();
      const testEnv = await createIsolatedTestEnvironment('verbose-progress');
      
      try {
        const indexResult = await runCLIWithLogging(['index', testDataPath, '--verbose'], testEnv.env, 120000);
        expectCLISuccess(indexResult);
        
        // Test verbose mode messages
        expect(indexResult.stdout).toMatch(/Scanning directory:/);
        expect(indexResult.stdout).toMatch(/Found \d+ files to process in/);
        expect(indexResult.stdout).toMatch(/Indexed: .*\.\w+ \(\d+ chunks\)/);
        expect(indexResult.stdout).toMatch(/Directory .* completed:/);
      } finally {
        await testEnv.cleanup();
      }
    });

    it('should report accurate per-directory counts', async () => {
      const testDataPath = getTestDataPath();
      const testEnv = await createIsolatedTestEnvironment('per-directory-counts');
      
      try {
        const indexResult = await runCLIWithLogging(['index', testDataPath, '--verbose'], testEnv.env, 120000);
        expectCLISuccess(indexResult);
        
        const dirCompletionLines = indexResult.stdout.split('\n')
          .filter(line => line.includes('Directory') && line.includes('completed:'));
        
        if (dirCompletionLines.length > 1) {
          for (const line of dirCompletionLines) {
            const match = line.match(/(\d+) indexed, (\d+) skipped/);
            if (match) {
              const indexed = parseInt(match[1]);
              const skipped = parseInt(match[2]);
              expect(indexed).toBeGreaterThanOrEqual(0);
              expect(skipped).toBeGreaterThanOrEqual(0);
              expect(indexed + skipped).toBeGreaterThan(0);
            }
          }
        }
      } finally {
        await testEnv.cleanup();
      }
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