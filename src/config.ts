import { homedir } from 'os';
import { join } from 'path';
import { existsSync, statSync } from 'fs';
import { z } from 'zod';
import { normalizePath } from './utils';

const WorkspaceSchema = z.object({
  paths: z.array(z.string()),
  isValid: z.boolean(),
  filesCount: z.number().optional(),
  chunksCount: z.number().optional(),
});

const ConfigSchema = z.object({
  storage: z.object({
    sqlitePath: z.string(),
    qdrantEndpoint: z.string().url(),
    qdrantCollection: z.string(),
    qdrantApiKey: z.string().optional(),
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
  workspaces: z.record(WorkspaceSchema),
});

export type Config = z.infer<typeof ConfigSchema>;
export type WorkspaceConfig = z.infer<typeof WorkspaceSchema>;

export class ConfigError extends Error {
  constructor(message: string, public override cause?: Error) {
    super(message);
    this.name = 'ConfigError';
  }
}

function parseWorkspaces(env: Record<string, string | undefined>): Record<string, WorkspaceConfig> {
  const workspaces: Record<string, WorkspaceConfig> = {};
  
  for (const [key, value] of Object.entries(env)) {
    if (key.startsWith('WORKSPACE_') && value) {
      const name = key.replace('WORKSPACE_', '').toLowerCase();
      
      // Parse paths from comma-separated string or JSON array
      let paths: string[];
      try {
        // Try parsing as JSON array first
        paths = JSON.parse(value);
        if (!Array.isArray(paths)) {
          throw new Error('Not an array');
        }
      } catch {
        // Fall back to comma-separated string
        paths = value.split(',').map(p => p.trim()).filter(p => p.length > 0);
      }
      
      // Normalize paths for consistent comparison
      const normalizedPaths = paths.map(normalizePath);
      
      // Validate that paths exist and are directories
      const isValid = normalizedPaths.every(path => {
        try {
          return existsSync(path) && statSync(path).isDirectory();
        } catch {
          return false;
        }
      });
      
      workspaces[name] = {
        paths: normalizedPaths,
        isValid,
      };
    }
  }
  
  return workspaces;
}

export function getWorkspacePaths(config: Config, workspace: string): string[] {
  const workspaceConfig = config.workspaces[workspace];
  return workspaceConfig?.paths || [];
}

export function getAvailableWorkspaces(config: Config): string[] {
  return Object.keys(config.workspaces);
}

export function loadConfig(options: { verbose?: boolean } = {}): Config {
  const dataDir = process.env.DIRECTORY_INDEXER_DATA_DIR || join(homedir(), '.directory-indexer');
  
  // Use separate database file and collection for tests to avoid contaminating main data
  const isTest = process.env.NODE_ENV === 'test' || process.env.VITEST === 'true';
  const dbFileName = isTest ? 'test-data.db' : 'data.db';
  const defaultCollection = isTest ? 'directory-indexer-test' : 'directory-indexer';
  
  // Parse workspace configurations from environment variables
  const workspaces = parseWorkspaces(process.env);
  
  const config = {
    storage: {
      sqlitePath: join(dataDir, dbFileName),
      qdrantEndpoint: process.env.QDRANT_ENDPOINT || 'http://127.0.0.1:6333',
      qdrantCollection: process.env.DIRECTORY_INDEXER_QDRANT_COLLECTION || defaultCollection,
      qdrantApiKey: process.env.QDRANT_API_KEY,
    },
    embedding: {
      provider: (process.env.EMBEDDING_PROVIDER as Config['embedding']['provider']) || 'ollama',
      model: process.env.EMBEDDING_MODEL || 'nomic-embed-text',
      endpoint: process.env.OLLAMA_ENDPOINT || 'http://127.0.0.1:11434',
    },
    indexing: {
      chunkSize: parseInt(process.env.CHUNK_SIZE || '512'),
      chunkOverlap: parseInt(process.env.CHUNK_OVERLAP || '50'),
      maxFileSize: parseInt(process.env.MAX_FILE_SIZE || '10485760'),
      ignorePatterns: ['.git', 'node_modules', 'target', '.DS_Store'],
    },
    dataDir,
    verbose: options.verbose ?? (process.env.VERBOSE === 'true'),
    workspaces,
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