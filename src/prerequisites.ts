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

export interface ServiceStatus {
  service: string;
  status: 'available' | 'unavailable';
  details?: string;
}

export interface PrerequisiteValidationResult {
  allPassed: boolean;
  services: ServiceStatus[];
}

/**
 * Check all prerequisites and return structured status
 */
export async function checkAllPrerequisitesDetailed(config: Config): Promise<PrerequisiteValidationResult> {
  const services: ServiceStatus[] = [];
  
  // Check Qdrant
  const qdrantOk = await checkQdrant(config);
  services.push({
    service: 'qdrant',
    status: qdrantOk ? 'available' : 'unavailable',
    details: qdrantOk ? undefined : `Cannot connect to Qdrant at ${config.storage.qdrantEndpoint}`
  });
  
  // Check embedding service
  let embeddingOk = false;
  let embeddingDetails: string | undefined;
  
  if (config.embedding.provider === 'ollama') {
    const ollamaOk = await checkOllama(config);
    const modelOk = ollamaOk ? await checkOllamaModel(config) : false;
    embeddingOk = ollamaOk && modelOk;
    
    if (!ollamaOk) {
      embeddingDetails = `Cannot connect to Ollama at ${config.embedding.endpoint}`;
    } else if (!modelOk) {
      embeddingDetails = `Model "${config.embedding.model}" not available in Ollama`;
    }
  } else if (config.embedding.provider === 'openai') {
    embeddingOk = await checkOpenAI(config);
    if (!embeddingOk) {
      embeddingDetails = process.env.OPENAI_API_KEY ? 
        'OpenAI API request failed' : 
        'OPENAI_API_KEY environment variable not set';
    }
  } else if (config.embedding.provider === 'mock') {
    embeddingOk = true;
  } else {
    embeddingDetails = `Unknown embedding provider: ${config.embedding.provider}`;
  }
  
  services.push({
    service: config.embedding.provider,
    status: embeddingOk ? 'available' : 'unavailable',
    details: embeddingDetails
  });
  
  return {
    allPassed: services.every(s => s.status === 'available'),
    services
  };
}

/**
 * Generate comprehensive error message that lists all missing services
 */
function createComprehensiveErrorMessage(result: PrerequisiteValidationResult): string {
  const unavailableServices = result.services.filter(s => s.status === 'unavailable');
  
  const serviceDescriptions = unavailableServices.map(service => {
    return `${service.service} (${service.details})`;
  });
  
  const message = `Required services are not available to use directory-indexer features: ${serviceDescriptions.join(', ')}.`;
  const setup = 'For setup instructions, see: https://github.com/peteretelej/directory-indexer#setup';
  
  return `${message}\n\n${setup}`;
}

/**
 * Validate all prerequisites for indexing (needs both Qdrant and embedding service)
 */
export async function validateIndexPrerequisites(config: Config): Promise<void> {
  const result = await checkAllPrerequisitesDetailed(config);
  
  if (!result.allPassed) {
    throw new PrerequisiteError(createComprehensiveErrorMessage(result));
  }
}

/**
 * Validate prerequisites for search (needs only Qdrant)
 */
export async function validateSearchPrerequisites(config: Config): Promise<void> {
  const result = await checkAllPrerequisitesDetailed(config);
  const qdrantService = result.services.find(s => s.service === 'qdrant');
  
  if (qdrantService?.status === 'unavailable') {
    // Create a result with only Qdrant for error message
    const qdrantOnlyResult: PrerequisiteValidationResult = {
      allPassed: false,
      services: [qdrantService]
    };
    throw new PrerequisiteError(createComprehensiveErrorMessage(qdrantOnlyResult));
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