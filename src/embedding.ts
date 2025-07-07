import { Config } from './config.js';

export interface EmbeddingProvider {
  name: string;
  dimensions: number;
  generateEmbedding(text: string): Promise<number[]>;
  generateEmbeddings(texts: string[]): Promise<number[][]>;
}

export interface EmbeddingConfig {
  model: string;
  endpoint: string;
  dimensions: number;
}

export class EmbeddingError extends Error {
  constructor(message: string, public override cause?: Error) {
    super(message);
    this.name = 'EmbeddingError';
  }
}

class MockEmbeddingProvider implements EmbeddingProvider {
  name = 'mock';
  
  constructor(private config: EmbeddingConfig) {}
  
  get dimensions(): number {
    return this.config.dimensions;
  }
  
  async generateEmbedding(text: string): Promise<number[]> {
    const hash = this.hashString(text);
    return Array.from({ length: this.dimensions }, (_, i) => 
      Math.sin(hash + i) * 0.1
    );
  }
  
  async generateEmbeddings(texts: string[]): Promise<number[][]> {
    return Promise.all(texts.map(text => this.generateEmbedding(text)));
  }
  
  private hashString(str: string): number {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash;
    }
    return Math.abs(hash);
  }
}

class OllamaEmbeddingProvider implements EmbeddingProvider {
  name = 'ollama';
  dimensions = 768;
  
  constructor(private config: EmbeddingConfig) {}
  
  async generateEmbedding(text: string): Promise<number[]> {
    try {
      const response = await fetch(`${this.config.endpoint}/api/embed`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          model: this.config.model,
          input: text
        })
      });
      
      if (!response.ok) {
        throw new Error(`Ollama API error: ${response.statusText}`);
      }
      
      const data = await response.json();
      return data.embeddings[0];
    } catch (error) {
      throw new EmbeddingError(`Failed to generate Ollama embedding`, error as Error);
    }
  }
  
  async generateEmbeddings(texts: string[]): Promise<number[][]> {
    try {
      const response = await fetch(`${this.config.endpoint}/api/embed`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          model: this.config.model,
          input: texts
        })
      });
      
      if (!response.ok) {
        throw new Error(`Ollama API error: ${response.statusText}`);
      }
      
      const data = await response.json();
      return data.embeddings;
    } catch (error) {
      throw new EmbeddingError(`Failed to generate Ollama embeddings`, error as Error);
    }
  }
}

class OpenAIEmbeddingProvider implements EmbeddingProvider {
  name = 'openai';
  dimensions = 1536;
  
  constructor(private config: EmbeddingConfig) {}
  
  async generateEmbedding(text: string): Promise<number[]> {
    try {
      const response = await fetch('https://api.openai.com/v1/embeddings', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${process.env.OPENAI_API_KEY}`
        },
        body: JSON.stringify({
          model: this.config.model,
          input: text
        })
      });
      
      if (!response.ok) {
        throw new Error(`OpenAI API error: ${response.statusText}`);
      }
      
      const data = await response.json();
      return data.data[0].embedding;
    } catch (error) {
      throw new EmbeddingError(`Failed to generate OpenAI embedding`, error as Error);
    }
  }
  
  async generateEmbeddings(texts: string[]): Promise<number[][]> {
    try {
      const response = await fetch('https://api.openai.com/v1/embeddings', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${process.env.OPENAI_API_KEY}`
        },
        body: JSON.stringify({
          model: this.config.model,
          input: texts
        })
      });
      
      if (!response.ok) {
        throw new Error(`OpenAI API error: ${response.statusText}`);
      }
      
      const data = await response.json();
      return data.data.map((item: { embedding: number[] }) => item.embedding);
    } catch (error) {
      throw new EmbeddingError(`Failed to generate OpenAI embeddings`, error as Error);
    }
  }
}

export function createEmbeddingProvider(provider: string, config: EmbeddingConfig): EmbeddingProvider {
  switch (provider) {
    case 'mock':
      return new MockEmbeddingProvider(config);
    case 'ollama':
      return new OllamaEmbeddingProvider(config);
    case 'openai':
      return new OpenAIEmbeddingProvider(config);
    default:
      throw new EmbeddingError(`Unknown embedding provider: ${provider}`);
  }
}

export async function generateEmbedding(text: string, config: Config): Promise<number[]> {
  const provider = createEmbeddingProvider(config.embedding.provider, {
    model: config.embedding.model,
    endpoint: config.embedding.endpoint,
    dimensions: config.embedding.provider === 'mock' ? 384 : (config.embedding.provider === 'ollama' ? 768 : 1536)
  });
  
  return provider.generateEmbedding(text);
}