import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { createEmbeddingProvider, EmbeddingError } from '../src/embedding.js';

describe('Embedding Provider Unit Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('MockEmbeddingProvider', () => {
    it('should generate deterministic embeddings based on text input', async () => {
      const provider = createEmbeddingProvider('mock', {
        model: 'test-model',
        endpoint: '',
        dimensions: 10
      });
      
      const text = 'consistent test input';
      const embedding1 = await provider.generateEmbedding(text);
      const embedding2 = await provider.generateEmbedding(text);
      
      expect(embedding1).toEqual(embedding2);
      expect(embedding1.length).toBe(10);
      expect(embedding1.every(val => typeof val === 'number')).toBe(true);
    });

    it('should generate different embeddings for different inputs', async () => {
      const provider = createEmbeddingProvider('mock', {
        model: 'test-model',
        endpoint: '',
        dimensions: 384
      });
      
      const embedding1 = await provider.generateEmbedding('text one');
      const embedding2 = await provider.generateEmbedding('text two');
      
      expect(embedding1).not.toEqual(embedding2);
      expect(embedding1.length).toBe(384);
      expect(embedding2.length).toBe(384);
    });

    it('should handle batch generation correctly', async () => {
      const provider = createEmbeddingProvider('mock', {
        model: 'test-model',
        endpoint: '',
        dimensions: 128
      });
      
      const texts = ['one', 'two', 'three'];
      const embeddings = await provider.generateEmbeddings(texts);
      
      expect(embeddings.length).toBe(3);
      expect(embeddings[0].length).toBe(128);
      expect(embeddings[1].length).toBe(128);
      expect(embeddings[2].length).toBe(128);
      
      // Each embedding should be different
      expect(embeddings[0]).not.toEqual(embeddings[1]);
      expect(embeddings[1]).not.toEqual(embeddings[2]);
    });

  });

  describe('OllamaEmbeddingProvider', () => {
    it('should format request correctly', async () => {
      const mockFetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ embedding: new Array(768).fill(0.1) })
      });
      global.fetch = mockFetch;

      const provider = createEmbeddingProvider('ollama', {
        model: 'nomic-embed-text',
        endpoint: 'http://localhost:11434',
        dimensions: 768
      });

      await provider.generateEmbedding('test text');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:11434/api/embeddings',
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            model: 'nomic-embed-text',
            prompt: 'test text'
          })
        }
      );
    });

    it('should handle API error responses', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: false,
        statusText: 'Model not found'
      });

      const provider = createEmbeddingProvider('ollama', {
        model: 'nonexistent-model',
        endpoint: 'http://localhost:11434',
        dimensions: 768
      });

      try {
        await provider.generateEmbedding('test');
        expect(false).toBe(true); // Should not reach here
      } catch (error) {
        expect(error).toBeInstanceOf(EmbeddingError);
        expect(error.message).toContain('Failed to generate Ollama embedding');
      }
    });

    it('should handle batch embeddings sequentially', async () => {
      const fetchMock = vi.fn()
        .mockResolvedValueOnce({
          ok: true,
          json: () => Promise.resolve({ embedding: new Array(768).fill(0.1) })
        })
        .mockResolvedValueOnce({
          ok: true,
          json: () => Promise.resolve({ embedding: new Array(768).fill(0.2) })
        });
      
      global.fetch = fetchMock;

      const provider = createEmbeddingProvider('ollama', {
        model: 'nomic-embed-text',
        endpoint: 'http://localhost:11434',
        dimensions: 768
      });

      const embeddings = await provider.generateEmbeddings(['one', 'two']);
      
      expect(fetchMock).toHaveBeenCalledTimes(2);
      expect(embeddings.length).toBe(2);
      // The embeddings should be different based on our mock
      expect(embeddings[0][0]).toBe(0.1);
      expect(embeddings[1][0]).toBe(0.2);
    });
  });

  describe('OpenAIEmbeddingProvider', () => {
    it('should format request correctly with API key', async () => {
      const originalEnv = process.env.OPENAI_API_KEY;
      process.env.OPENAI_API_KEY = 'test-api-key';

      const mockFetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({
          data: [{ embedding: new Array(1536).fill(0.1) }]
        })
      });
      global.fetch = mockFetch;

      try {
        const provider = createEmbeddingProvider('openai', {
          model: 'text-embedding-3-small',
          endpoint: 'https://api.openai.com/v1',
          dimensions: 1536
        });

        await provider.generateEmbedding('test text');

        expect(mockFetch).toHaveBeenCalledWith(
          'https://api.openai.com/v1/embeddings',
          {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
              'Authorization': 'Bearer test-api-key'
            },
            body: JSON.stringify({
              model: 'text-embedding-3-small',
              input: 'test text'
            })
          }
        );
      } finally {
        if (originalEnv) {
          process.env.OPENAI_API_KEY = originalEnv;
        } else {
          delete process.env.OPENAI_API_KEY;
        }
      }
    });

    it('should handle missing API key', async () => {
      const originalEnv = process.env.OPENAI_API_KEY;
      delete process.env.OPENAI_API_KEY;

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
        if (originalEnv) {
          process.env.OPENAI_API_KEY = originalEnv;
        }
      }
    });

    it('should handle rate limiting errors', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: false,
        status: 429,
        statusText: 'Too Many Requests'
      });

      const provider = createEmbeddingProvider('openai', {
        model: 'text-embedding-3-small',
        endpoint: 'https://api.openai.com/v1',
        dimensions: 1536
      });

      try {
        await provider.generateEmbedding('test');
        expect(false).toBe(true); // Should not reach here
      } catch (error) {
        expect(error).toBeInstanceOf(EmbeddingError);
        expect(error.message).toContain('Failed to generate OpenAI embedding');
      }
    });


  });

  describe('Provider Selection', () => {
    it('should create correct provider based on type', () => {
      const mockProvider = createEmbeddingProvider('mock', {
        model: 'test',
        endpoint: '',
        dimensions: 384
      });
      expect(mockProvider.name).toBe('mock');
      expect(mockProvider.dimensions).toBe(384);

      const ollamaProvider = createEmbeddingProvider('ollama', {
        model: 'nomic-embed-text',
        endpoint: 'http://localhost:11434',
        dimensions: 768
      });
      expect(ollamaProvider.name).toBe('ollama');
      expect(ollamaProvider.dimensions).toBe(768);

      const openaiProvider = createEmbeddingProvider('openai', {
        model: 'text-embedding-3-small',
        endpoint: 'https://api.openai.com/v1',
        dimensions: 1536
      });
      expect(openaiProvider.name).toBe('openai');
      expect(openaiProvider.dimensions).toBe(1536);
    });

    it('should throw error for unsupported provider type', () => {
      expect(() => {
        createEmbeddingProvider('unsupported' as any, {
          model: 'test',
          endpoint: 'http://localhost',
          dimensions: 384
        });
      }).toThrow('Unknown embedding provider: unsupported');
    });
  });
});