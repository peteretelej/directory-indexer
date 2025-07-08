import { Config } from './config.js';

export class PrerequisiteError extends Error {
  constructor(message: string, public override cause?: Error) {
    super(message);
    this.name = 'PrerequisiteError';
  }
}

/**
 * Check if Qdrant is accessible
 */
export async function checkQdrant(config: Config): Promise<boolean> {
  try {
    const response = await fetch(`${config.storage.qdrantEndpoint}/healthz`);
    return response.ok;
  } catch {
    return false;
  }
}

/**
 * Check if Ollama is accessible
 */
export async function checkOllama(config: Config): Promise<boolean> {
  try {
    const response = await fetch(`${config.embedding.endpoint}/api/tags`);
    return response.ok;
  } catch {
    return false;
  }
}

/**
 * Check if Ollama model is available
 */
export async function checkOllamaModel(config: Config): Promise<boolean> {
  try {
    const response = await fetch(`${config.embedding.endpoint}/api/tags`);
    if (!response.ok) return false;
    
    const data = await response.json();
    const models = data.models || [];
    return models.some((model: { name: string }) => model.name.includes(config.embedding.model));
  } catch {
    return false;
  }
}

/**
 * Check if OpenAI is accessible
 */
export async function checkOpenAI(config: Config): Promise<boolean> {
  if (!process.env.OPENAI_API_KEY) return false;
  
  try {
    const response = await fetch('https://api.openai.com/v1/embeddings', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${process.env.OPENAI_API_KEY}`
      },
      body: JSON.stringify({
        model: config.embedding.model,
        input: 'test'
      })
    });
    return response.ok;
  } catch {
    return false;
  }
}

/**
 * Generate error message for failed prerequisites
 */
function createErrorMessage(qdrantOk: boolean, embeddingOk: boolean, provider: string): string {
  const errors: string[] = [];
  
  if (!qdrantOk) {
    errors.push('Qdrant database is inaccessible');
  }
  
  if (!embeddingOk) {
    if (provider === 'ollama') {
      errors.push('Ollama embedding service is inaccessible or model unavailable');
    } else if (provider === 'openai') {
      errors.push('OpenAI API is inaccessible or key invalid');
    }
  }
  
  errors.push('');
  errors.push('For setup instructions, see: https://github.com/peteretelej/directory-indexer#setup');
  
  return errors.join('\n');
}

/**
 * Validate all prerequisites for indexing (needs both Qdrant and embedding service)
 */
export async function validateIndexPrerequisites(config: Config): Promise<void> {
  const [qdrantOk, embeddingOk] = await Promise.all([
    checkQdrant(config),
    checkEmbeddingService(config)
  ]);
  
  if (!qdrantOk || !embeddingOk) {
    throw new PrerequisiteError(createErrorMessage(qdrantOk, embeddingOk, config.embedding.provider));
  }
}

/**
 * Validate prerequisites for search (needs only Qdrant)
 */
export async function validateSearchPrerequisites(config: Config): Promise<void> {
  const qdrantOk = await checkQdrant(config);
  
  if (!qdrantOk) {
    throw new PrerequisiteError(createErrorMessage(false, true, config.embedding.provider));
  }
}

/**
 * Check embedding service based on provider
 */
async function checkEmbeddingService(config: Config): Promise<boolean> {
  switch (config.embedding.provider) {
    case 'ollama':
      return (await checkOllama(config)) && (await checkOllamaModel(config));
    case 'openai':
      return await checkOpenAI(config);
    case 'mock':
      return true;
    default:
      return false;
  }
}

/**
 * Get status of all services (for status command)
 */
export async function getServiceStatus(config: Config): Promise<{
  qdrant: boolean;
  embedding: boolean;
  embeddingProvider: string;
}> {
  const [qdrantOk, embeddingOk] = await Promise.all([
    checkQdrant(config),
    checkEmbeddingService(config)
  ]);
  
  return {
    qdrant: qdrantOk,
    embedding: embeddingOk,
    embeddingProvider: config.embedding.provider
  };
}