import { describe, it, expect } from 'vitest';
import { chunkText } from '../src/indexing.js';
import { createEmbeddingProvider } from '../src/embedding.js';

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
  });

  it('should handle overlap larger than chunk size', () => {
    const text = 'This is a test text that should be chunked properly even with large overlap.';
    const chunks = chunkText(text, 10, 20); // Overlap > chunk size
    expect(chunks.length).toBeGreaterThan(0);
  });

  it('should handle very long text', () => {
    const longText = 'a'.repeat(100000);
    const chunks = chunkText(longText, 512, 50);
    expect(chunks.length).toBeGreaterThan(100);
    expect(chunks[0].content.length).toBeLessThanOrEqual(512);
  });

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