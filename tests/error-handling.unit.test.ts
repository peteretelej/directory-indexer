import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { ConfigError } from '../src/config.js';
import { EmbeddingError, createEmbeddingProvider } from '../src/embedding.js';
import { SearchError, searchContent } from '../src/search.js';

describe('Error Handling Unit Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('Configuration Errors', () => {
    it('should create ConfigError with cause', () => {
      const originalError = new Error('Original error');
      const configError = new ConfigError('Config failed', originalError);
      
      expect(configError.name).toBe('ConfigError');
      expect(configError.message).toBe('Config failed');
      expect(configError.cause).toBe(originalError);
    });

    it('should create ConfigError without cause', () => {
      const configError = new ConfigError('Config failed');
      
      expect(configError.name).toBe('ConfigError');
      expect(configError.message).toBe('Config failed');
      expect(configError.cause).toBeUndefined();
    });
  });

  describe('Embedding Provider Errors', () => {
    it('should create EmbeddingError with cause', () => {
      const originalError = new Error('Network error');
      const embeddingError = new EmbeddingError('Embedding failed', originalError);
      
      expect(embeddingError.name).toBe('EmbeddingError');
      expect(embeddingError.message).toBe('Embedding failed');
      expect(embeddingError.cause).toBe(originalError);
    });


    it('should handle Ollama provider network errors', async () => {
      const provider = createEmbeddingProvider('ollama', {
        model: 'nomic-embed-text',
        endpoint: 'http://nonexistent-host:11434',
        dimensions: 768
      });
      
      try {
        await provider.generateEmbedding('test text');
        expect(false).toBe(true); // Should not reach here
      } catch (error) {
        expect(error).toBeInstanceOf(EmbeddingError);
      }
    });

    it('should handle invalid provider type', () => {
      expect(() => {
        createEmbeddingProvider('invalid' as any, {
          model: 'test',
          endpoint: 'http://localhost',
          dimensions: 384
        });
      }).toThrow();
    });
  });

  describe('Search Errors', () => {
    it('should create SearchError with cause', () => {
      const originalError = new Error('Search failed');
      const searchError = new SearchError('Search operation failed', originalError);
      
      expect(searchError.name).toBe('SearchError');
      expect(searchError.message).toBe('Search operation failed');
      expect(searchError.cause).toBe(originalError);
    });

    it('should handle search with empty query', async () => {
      try {
        await searchContent('', { limit: 10 });
        expect(false).toBe(true); // Should not reach here  
      } catch (error) {
        expect(error).toBeInstanceOf(Error);
      }
    });

    it('should handle search with negative limit', async () => {
      try {
        await searchContent('test query', { limit: -1 });
        expect(false).toBe(true); // Should not reach here
      } catch (error) {
        expect(error).toBeInstanceOf(Error);
      }
    });

    it('should handle search with invalid threshold', async () => {
      try {
        await searchContent('test query', { threshold: -1 });
        expect(false).toBe(true); // Should not reach here
      } catch (error) {
        expect(error).toBeInstanceOf(Error);
      }
    });
  });

  describe('Network Error Simulation', () => {

    it('should handle non-OK responses in OpenAI provider', async () => {
      // Mock fetch to simulate API error response
      const originalFetch = global.fetch;
      global.fetch = vi.fn().mockResolvedValue({
        ok: false,
        status: 401,
        statusText: 'Unauthorized'
      });
      
      try {
        const provider = createEmbeddingProvider('openai', {
          model: 'text-embedding-3-small',
          endpoint: 'https://api.openai.com/v1',
          dimensions: 1536
        });
        
        await provider.generateEmbedding('test');
        expect(false).toBe(true); // Should not reach here
      } catch (error) {
        expect(error).toBeInstanceOf(EmbeddingError);
        expect(error.message).toContain('Failed to generate OpenAI embedding');
      } finally {
        global.fetch = originalFetch;
      }
    });


    it('should handle fetch failures in Ollama provider', async () => {
      // Mock fetch to simulate network failure
      const originalFetch = global.fetch;
      global.fetch = vi.fn().mockRejectedValue(new Error('Connection refused'));
      
      try {
        const provider = createEmbeddingProvider('ollama', {
          model: 'nomic-embed-text',
          endpoint: 'http://localhost:11434',
          dimensions: 768
        });
        
        await provider.generateEmbedding('test');
        expect(false).toBe(true); // Should not reach here
      } catch (error) {
        expect(error).toBeInstanceOf(EmbeddingError);
        expect(error.message).toContain('Failed to generate Ollama embedding');
      } finally {
        global.fetch = originalFetch;
      }
    });
  });

});