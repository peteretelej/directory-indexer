import { describe, it, expect } from 'vitest';
import { loadConfig } from '../src/config.js';

describe('Configuration', () => {
  it('should load default configuration', async () => {
    const config = await loadConfig();
    
    expect(config.storage.qdrantEndpoint).toBe('http://127.0.0.1:6333');
    expect(config.embedding.provider).toBe('ollama');
    expect(config.embedding.model).toBe('nomic-embed-text');
    expect(config.indexing.chunkSize).toBe(512);
    expect(config.indexing.chunkOverlap).toBe(50);
    expect(config.indexing.maxFileSize).toBe(10485760);
    expect(config.indexing.ignorePatterns).toContain('.git');
    expect(config.indexing.ignorePatterns).toContain('node_modules');
    expect(config.indexing.respectGitignore).toBe(true);
  });

  it('should override defaults with environment variables', async () => {
    const originalQdrant = process.env.QDRANT_ENDPOINT;
    const originalOllama = process.env.OLLAMA_ENDPOINT;
    const originalCollection = process.env.DIRECTORY_INDEXER_QDRANT_COLLECTION;
    
    try {
      process.env.QDRANT_ENDPOINT = 'http://custom:6333';
      process.env.OLLAMA_ENDPOINT = 'http://custom:11434';
      process.env.DIRECTORY_INDEXER_QDRANT_COLLECTION = 'custom-collection';
      
      const config = await loadConfig();
      
      expect(config.storage.qdrantEndpoint).toBe('http://custom:6333');
      expect(config.embedding.endpoint).toBe('http://custom:11434');
      expect(config.storage.qdrantCollection).toBe('custom-collection');
    } finally {
      if (originalQdrant) process.env.QDRANT_ENDPOINT = originalQdrant;
      else delete process.env.QDRANT_ENDPOINT;
      if (originalOllama) process.env.OLLAMA_ENDPOINT = originalOllama;
      else delete process.env.OLLAMA_ENDPOINT;
      if (originalCollection) process.env.DIRECTORY_INDEXER_QDRANT_COLLECTION = originalCollection;
      else delete process.env.DIRECTORY_INDEXER_QDRANT_COLLECTION;
    }
  });

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
      expect(config.storage.qdrantEndpoint).toBe('http://127.0.0.1:6333');
      expect(config.storage.qdrantCollection).toBe('directory-indexer-test');
      expect(config.embedding.provider).toBe('ollama');
      expect(config.embedding.model).toBe('nomic-embed-text');
      expect(config.embedding.endpoint).toBe('http://127.0.0.1:11434');
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