import { describe, it, expect, vi, afterEach } from 'vitest';
import { loadConfig } from '../src/config.js';
import { checkOpenAI } from '../src/prerequisites.js';

describe('checkOpenAI prerequisites probe', () => {
  const original = {
    provider: process.env.EMBEDDING_PROVIDER,
    openaiEndpoint: process.env.OPENAI_ENDPOINT,
    openaiKey: process.env.OPENAI_API_KEY,
    ollamaEndpoint: process.env.OLLAMA_ENDPOINT,
  };

  afterEach(() => {
    vi.restoreAllMocks();
    if (original.provider) process.env.EMBEDDING_PROVIDER = original.provider;
    else delete process.env.EMBEDDING_PROVIDER;
    if (original.openaiEndpoint) process.env.OPENAI_ENDPOINT = original.openaiEndpoint;
    else delete process.env.OPENAI_ENDPOINT;
    if (original.openaiKey) process.env.OPENAI_API_KEY = original.openaiKey;
    else delete process.env.OPENAI_API_KEY;
    if (original.ollamaEndpoint) process.env.OLLAMA_ENDPOINT = original.ollamaEndpoint;
    else delete process.env.OLLAMA_ENDPOINT;
  });

  it('should probe the configured endpoint', async () => {
    process.env.EMBEDDING_PROVIDER = 'openai';
    process.env.OPENAI_ENDPOINT = 'https://example.internal/v1';
    process.env.OPENAI_API_KEY = 'test-api-key';
    delete process.env.OLLAMA_ENDPOINT;

    const mockFetch = vi.fn().mockResolvedValue({ ok: true });
    (globalThis as any).fetch = mockFetch;

    const config = await loadConfig();
    const ok = await checkOpenAI(config);

    expect(ok).toBe(true);
    expect(mockFetch).toHaveBeenCalledWith(
      'https://example.internal/v1/embeddings',
      expect.objectContaining({ method: 'POST' })
    );
  });

  it('should return false without an API key and not call the endpoint', async () => {
    delete process.env.OPENAI_API_KEY;

    const mockFetch = vi.fn();
    (globalThis as any).fetch = mockFetch;

    const config = await loadConfig();
    const ok = await checkOpenAI(config);

    expect(ok).toBe(false);
    expect(mockFetch).not.toHaveBeenCalled();
  });
});
