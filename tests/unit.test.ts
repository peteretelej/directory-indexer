import { describe, it, expect } from 'vitest';
import * as path from 'path';
import { loadConfig } from '../src/config.js';
import { normalizePath, calculateHash } from '../src/utils.js';
import { createEmbeddingProvider } from '../src/embedding.js';
import { chunkText } from '../src/indexing.js';

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
});

describe('Path Utilities', () => {
  it('should normalize paths across platforms', () => {
    const windowsPath = 'C:\\Users\\test\\Documents';
    const unixPath = '/home/test/documents';
    
    const normalizedWindows = normalizePath(windowsPath);
    const normalizedUnix = normalizePath(unixPath);
    
    expect(typeof normalizedWindows).toBe('string');
    expect(typeof normalizedUnix).toBe('string');
    expect(normalizedWindows.length).toBeGreaterThan(0);
    expect(normalizedUnix.length).toBeGreaterThan(0);
  });

  it('should convert relative paths to absolute', () => {
    const relativePath = './test/path';
    const absolutePath = normalizePath(relativePath);
    
    expect(path.isAbsolute(absolutePath)).toBe(true);
  });

  it('should calculate file hashes consistently', () => {
    const testContent = 'Hello, world!';
    const hash1 = calculateHash(testContent);
    const hash2 = calculateHash(testContent);
    
    expect(hash1).toBe(hash2);
    expect(hash1.length).toBeGreaterThan(0);
    expect(typeof hash1).toBe('string');
  });
});

describe('Storage Operations', () => {
  it('should initialize SQLite storage', async () => {
    const { SQLiteStorage } = await import('../src/storage.js');
    const config = await loadConfig();
    config.storage.sqlitePath = ':memory:';
    const storage = new SQLiteStorage(config);
    
    expect(storage).toBeDefined();
    expect(storage.db).toBeDefined();
    
    storage.close();
  });

  it('should create QdrantClient', async () => {
    const { QdrantClient } = await import('../src/storage.js');
    const config = await loadConfig();
    const client = new QdrantClient(config);
    
    expect(client).toBeDefined();
    expect(typeof client.healthCheck).toBe('function');
    expect(typeof client.createCollection).toBe('function');
  });
});

describe('Embedding Providers', () => {

  it('should handle batch embedding generation', async () => {
    const provider = createEmbeddingProvider('mock', { model: 'test-model', endpoint: '', dimensions: 384 });
    
    const texts = ['text one', 'text two', 'text three'];
    const embeddings = await provider.generateEmbeddings(texts);
    
    expect(embeddings.length).toBe(3);
    expect(embeddings[0].length).toBe(384);
    expect(embeddings[1].length).toBe(384);
    expect(embeddings[2].length).toBe(384);
  });

  it('should create different provider types', async () => {
    const ollamaProvider = createEmbeddingProvider('ollama', { model: 'nomic-embed-text', endpoint: 'http://localhost:11434', dimensions: 768 });
    expect(ollamaProvider.name).toBe('ollama');
    expect(ollamaProvider.dimensions).toBe(768);
    
    const openaiProvider = createEmbeddingProvider('openai', { model: 'text-embedding-3-small', endpoint: 'https://api.openai.com/v1', dimensions: 1536 });
    expect(openaiProvider.name).toBe('openai');
    expect(openaiProvider.dimensions).toBe(1536);
  });
});

describe('Text Chunking', () => {
  it('should chunk text with sliding window', () => {
    const longText = 'This is a very long text that needs to be chunked into smaller pieces for embedding generation and vector storage.';
    const chunks = chunkText(longText, 50, 10);
    
    expect(Array.isArray(chunks)).toBe(true);
    expect(chunks.length).toBeGreaterThan(1);
    expect(chunks[0].content.length).toBeLessThanOrEqual(50);
    expect(chunks[0].startIndex).toBe(0);
  });

  it('should handle overlap between chunks', () => {
    const text = 'Word one two three four five six seven eight nine ten eleven twelve.';
    const chunks = chunkText(text, 30, 10);
    
    expect(chunks.length).toBeGreaterThan(1);
    expect(chunks[0].endIndex).toBeGreaterThanOrEqual(chunks[1].startIndex - 10);
  });

  it('should handle short text', () => {
    const shortText = 'Short text.';
    const chunks = chunkText(shortText, 50, 10);
    
    expect(chunks.length).toBe(1);
    expect(chunks[0].content).toBe(shortText);
    expect(chunks[0].startIndex).toBe(0);
    expect(chunks[0].endIndex).toBe(shortText.length);
  });
});




