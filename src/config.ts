import { homedir } from 'os';
import { join } from 'path';
import { z } from 'zod';

const ConfigSchema = z.object({
  storage: z.object({
    sqlitePath: z.string(),
    qdrantEndpoint: z.string().url(),
    qdrantCollection: z.string(),
  }),
  embedding: z.object({
    provider: z.enum(['ollama', 'openai', 'mock']),
    model: z.string(),
    endpoint: z.string().url(),
  }),
  indexing: z.object({
    chunkSize: z.number().positive(),
    chunkOverlap: z.number().nonnegative(),
    maxFileSize: z.number().positive(),
    ignorePatterns: z.array(z.string()),
  }),
  dataDir: z.string(),
  verbose: z.boolean(),
});

export type Config = z.infer<typeof ConfigSchema>;

export class ConfigError extends Error {
  constructor(message: string, public override cause?: Error) {
    super(message);
    this.name = 'ConfigError';
  }
}

export function loadConfig(options: { verbose?: boolean } = {}): Config {
  const dataDir = process.env.DIRECTORY_INDEXER_DATA_DIR || join(homedir(), '.directory-indexer');
  
  // Use separate database file and collection for tests to avoid contaminating main data
  const isTest = process.env.NODE_ENV === 'test' || process.env.VITEST === 'true';
  const dbFileName = isTest ? 'test-data.db' : 'data.db';
  const defaultCollection = isTest ? 'directory-indexer-test' : 'directory-indexer';
  
  const config = {
    storage: {
      sqlitePath: join(dataDir, dbFileName),
      qdrantEndpoint: process.env.QDRANT_ENDPOINT || 'http://localhost:6333',
      qdrantCollection: process.env.DIRECTORY_INDEXER_QDRANT_COLLECTION || defaultCollection,
    },
    embedding: {
      provider: (process.env.EMBEDDING_PROVIDER as Config['embedding']['provider']) || 'ollama',
      model: process.env.EMBEDDING_MODEL || 'nomic-embed-text',
      endpoint: process.env.OLLAMA_ENDPOINT || 'http://localhost:11434',
    },
    indexing: {
      chunkSize: parseInt(process.env.CHUNK_SIZE || '512'),
      chunkOverlap: parseInt(process.env.CHUNK_OVERLAP || '50'),
      maxFileSize: parseInt(process.env.MAX_FILE_SIZE || '10485760'),
      ignorePatterns: ['.git', 'node_modules', 'target', '.DS_Store'],
    },
    dataDir,
    verbose: options.verbose ?? (process.env.VERBOSE === 'true'),
  };

  try {
    return ConfigSchema.parse(config);
  } catch (error) {
    if (error instanceof z.ZodError) {
      const messages = error.errors.map(e => `${e.path.join('.')}: ${e.message}`);
      throw new ConfigError(`Configuration validation failed: ${messages.join(', ')}`, error);
    }
    throw new ConfigError('Failed to load configuration', error as Error);
  }
}