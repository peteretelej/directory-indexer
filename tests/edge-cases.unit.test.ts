import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { loadConfig } from '../src/config.js';
import { chunkText } from '../src/indexing.js';
import { calculateHash } from '../src/utils.js';
import { SQLiteStorage } from '../src/storage.js';

describe('Edge Cases Unit Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('Configuration Edge Cases', () => {
    it('should handle missing environment variables gracefully', async () => {
      const originalEnv = { ...process.env };
      
      try {
        // Clear all relevant environment variables
        delete process.env.QDRANT_ENDPOINT;
        delete process.env.OLLAMA_ENDPOINT;
        delete process.env.DIRECTORY_INDEXER_DATA_DIR;
        delete process.env.DIRECTORY_INDEXER_QDRANT_COLLECTION;
        delete process.env.EMBEDDING_PROVIDER;
        delete process.env.EMBEDDING_MODEL;
        delete process.env.CHUNK_SIZE;
        delete process.env.CHUNK_OVERLAP;
        delete process.env.MAX_FILE_SIZE;
        delete process.env.VERBOSE;
        
        const config = await loadConfig();
        
        // Should use defaults
        expect(config.storage.qdrantEndpoint).toBe('http://localhost:6333');
        expect(config.storage.qdrantCollection).toBe('directory-indexer');
        expect(config.embedding.provider).toBe('ollama');
        expect(config.embedding.model).toBe('nomic-embed-text');
        expect(config.embedding.endpoint).toBe('http://localhost:11434');
        expect(config.indexing.chunkSize).toBe(512);
        expect(config.indexing.chunkOverlap).toBe(50);
        expect(config.indexing.maxFileSize).toBe(10485760);
        expect(config.verbose).toBe(false);
      } finally {
        // Restore environment
        process.env = originalEnv;
      }
    });

    it('should handle invalid numeric environment variables', async () => {
      const originalEnv = { ...process.env };
      
      try {
        process.env.CHUNK_SIZE = 'invalid';
        process.env.CHUNK_OVERLAP = 'not-a-number';
        process.env.MAX_FILE_SIZE = 'text';
        
        try {
          await loadConfig();
          expect(false).toBe(true); // Should not reach here
        } catch (error) {
          expect((error as Error).name).toBe('ConfigError');
          expect((error as Error).message).toContain('Configuration validation failed');
        }
      } finally {
        process.env = originalEnv;
      }
    });

    it('should handle extreme numeric values', async () => {
      const originalEnv = { ...process.env };
      
      try {
        process.env.CHUNK_SIZE = '0';
        process.env.CHUNK_OVERLAP = '-1';
        process.env.MAX_FILE_SIZE = '999999999999999';
        
        try {
          await loadConfig();
          expect(false).toBe(true); // Should not reach here due to validation
        } catch (error) {
          expect((error as Error).name).toBe('ConfigError');
          expect((error as Error).message).toContain('Configuration validation failed');
        }
      } finally {
        process.env = originalEnv;
      }
    });
  });

  describe('Text Chunking Edge Cases', () => {
    it('should handle empty text', () => {
      const chunks = chunkText('', 512, 50);
      expect(chunks.length).toBe(1);
      expect(chunks[0].content).toBe('');
      expect(chunks[0].startIndex).toBe(0);
      expect(chunks[0].endIndex).toBe(0);
    });


    it('should handle zero chunk size', () => {
      const text = 'Some text';
      const chunks = chunkText(text, 0, 0);
      expect(chunks.length).toBeGreaterThan(0);
      // Should handle gracefully, not crash
    });

    it('should handle overlap larger than chunk size', () => {
      const text = 'This is a test text that should be chunked properly even with large overlap.';
      const chunks = chunkText(text, 10, 20); // Overlap > chunk size
      expect(chunks.length).toBeGreaterThan(0);
      // Should handle gracefully
    });

    it('should handle very long text', () => {
      const longText = 'a'.repeat(100000);
      const chunks = chunkText(longText, 512, 50);
      expect(chunks.length).toBeGreaterThan(100);
      expect(chunks[0].content.length).toBeLessThanOrEqual(512);
    });

  });


  describe('Hash Calculation Edge Cases', () => {

    it('should produce consistent hashes', () => {
      const input = 'consistent test string';
      const hash1 = calculateHash(input);
      const hash2 = calculateHash(input);
      expect(hash1).toBe(hash2);
    });

    it('should produce different hashes for different inputs', () => {
      const hash1 = calculateHash('string one');
      const hash2 = calculateHash('string two');
      expect(hash1).not.toBe(hash2);
    });
  });


  describe('Storage Edge Cases', () => {
    it('should handle file operations with invalid data', async () => {
      const config = await loadConfig();
      config.storage.sqlitePath = ':memory:';
      const storage = new SQLiteStorage(config);
      
      try {
        // Test with invalid file info
        await storage.upsertFile({
          path: '',
          size: -1,
          modifiedTime: new Date('invalid'),
          hash: '',
          parentDirs: []
        });
      } catch (error) {
        expect(error).toBeInstanceOf(Error);
      } finally {
        storage.close();
      }
    });
  });


  describe('Boundary Value Testing', () => {
    it('should handle minimum valid chunk size', () => {
      const chunks = chunkText('test', 1, 0);
      expect(chunks.length).toBeGreaterThan(0);
    });

    it('should handle maximum reasonable chunk size', () => {
      const text = 'a'.repeat(10000);
      const chunks = chunkText(text, 100000, 0);
      expect(chunks.length).toBe(1);
      expect(chunks[0].content).toBe(text);
    });

    it('should handle zero overlap', () => {
      const text = 'one two three four five six seven eight nine ten';
      const chunks = chunkText(text, 10, 0);
      expect(chunks.length).toBeGreaterThan(1);
    });

    it('should handle maximum overlap', () => {
      const text = 'one two three four five six seven eight nine ten';
      const chunks = chunkText(text, 10, 9);
      expect(chunks.length).toBeGreaterThan(1);
    });
  });
});