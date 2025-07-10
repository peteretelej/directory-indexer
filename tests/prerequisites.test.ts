import { describe, it, expect, beforeAll } from 'vitest';
import { loadConfig } from '../src/config.js';
import {
  validateIndexPrerequisites,
  validateSearchPrerequisites,
  getServiceStatus,
  PrerequisiteError
} from '../src/prerequisites.js';

describe('Prerequisites Tests', () => {
  let config: any;

  beforeAll(async () => {
    config = await loadConfig();
  });

  describe('With Working Services', () => {
    it('should validate all prerequisites when services are running', async () => {
      const status = await getServiceStatus(config);
      
      // Should work with real services in CI
      expect(status.qdrant).toBe(true);
      expect(status.embedding).toBe(true);
      expect(status.embeddingProvider).toBe('ollama');
      
      await expect(validateSearchPrerequisites(config)).resolves.not.toThrow();
      await expect(validateIndexPrerequisites(config)).resolves.not.toThrow();
    });
  });

  describe('With Invalid Endpoints', () => {
    it('should fail when Qdrant is unreachable', async () => {
      const invalidConfig = {
        ...config,
        storage: { ...config.storage, qdrantEndpoint: 'http://localhost:9999' }
      };
      
      await expect(validateSearchPrerequisites(invalidConfig)).rejects.toThrow(PrerequisiteError);
      await expect(validateSearchPrerequisites(invalidConfig)).rejects.toThrow('Required services are not available');
    });

    it('should fail when Ollama is unreachable', async () => {
      const invalidConfig = {
        ...config,
        embedding: { ...config.embedding, endpoint: 'http://localhost:9999' }
      };
      
      await expect(validateIndexPrerequisites(invalidConfig)).rejects.toThrow(PrerequisiteError);
      await expect(validateIndexPrerequisites(invalidConfig)).rejects.toThrow('Required services are not available');
    });

    it('should fail when OpenAI API key is missing', async () => {
      const originalKey = process.env.OPENAI_API_KEY;
      delete process.env.OPENAI_API_KEY;
      
      try {
        const openaiConfig = {
          ...config,
          embedding: { ...config.embedding, provider: 'openai' }
        };
        
        await expect(validateIndexPrerequisites(openaiConfig)).rejects.toThrow(PrerequisiteError);
        await expect(validateIndexPrerequisites(openaiConfig)).rejects.toThrow('Required services are not available');
      } finally {
        if (originalKey) process.env.OPENAI_API_KEY = originalKey;
      }
    });
  });
});