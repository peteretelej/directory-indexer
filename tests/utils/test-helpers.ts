import { spawn } from 'child_process';
import { join } from 'path';
import { promises as fs } from 'fs';
import { QdrantClient } from '../../src/storage.js';
import { loadConfig } from '../../src/config.js';

export function checkServicesAvailable(): Promise<boolean> {
  return Promise.all([
    checkQdrantHealth(),
    checkOllamaHealth()
  ]).then(([qdrant, ollama]) => qdrant && ollama);
}

export async function checkQdrantHealth(): Promise<boolean> {
  try {
    const response = await fetch('http://localhost:6333/healthz');
    if (!response.ok) return false;
    
    const collectionsResponse = await fetch('http://localhost:6333/collections');
    return collectionsResponse.ok;
  } catch {
    return false;
  }
}

export async function checkOllamaHealth(): Promise<boolean> {
  try {
    const response = await fetch('http://localhost:11434/api/tags');
    if (!response.ok) return false;
    
    await fetch('http://localhost:11434/api/embeddings', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        model: 'nomic-embed-text',
        prompt: 'test'
      })
    });
    
    return response.ok;
  } catch {
    return false;
  }
}

export function runCLI(args: string[], timeout = 30000, extraEnv: Record<string, string> = {}): Promise<{ stdout: string; stderr: string; exitCode: number }> {
  return new Promise((resolve, reject) => {
    const child = spawn('node', ['bin/directory-indexer.js', ...args], {
      env: {
        ...process.env,
        NODE_ENV: 'test',
        DIRECTORY_INDEXER_QDRANT_COLLECTION: 'directory-indexer-test-node',
        ...extraEnv
      }
    });

    let stdout = '';
    let stderr = '';

    child.stdout?.on('data', (data) => {
      stdout += data.toString();
    });

    child.stderr?.on('data', (data) => {
      stderr += data.toString();
    });

    const timer = setTimeout(() => {
      child.kill();
      reject(new Error(`Command timed out after ${timeout}ms`));
    }, timeout);

    child.on('close', (code) => {
      clearTimeout(timer);
      resolve({ stdout, stderr, exitCode: code || 0 });
    });

    child.on('error', (error) => {
      clearTimeout(timer);
      reject(error);
    });
  });
}

export async function createTempTestDirectory(): Promise<string> {
  const tempDir = join(process.cwd(), 'tests', 'temp-' + Date.now());
  await fs.mkdir(tempDir, { recursive: true });
  return tempDir;
}

export async function cleanupTempDirectory(dir: string): Promise<void> {
  await fs.rm(dir, { recursive: true, force: true });
}

export function getTestDataPath(): string {
  return join(process.cwd(), 'tests', 'test_data');
}

export function getTestFile(): string {
  return join(getTestDataPath(), 'docs', 'api_guide.md');
}

export async function setupServicesCheck() {
  const servicesAvailable = await checkServicesAvailable();
  if (!servicesAvailable) {
    const qdrantHealthy = await checkQdrantHealth();
    const ollamaHealthy = await checkOllamaHealth();
    
    console.error('❌ Integration tests require both Qdrant and Ollama services');
    console.error(`Qdrant (localhost:6333): ${qdrantHealthy ? '✅' : '❌'}`);
    console.error(`Ollama (localhost:11434): ${ollamaHealthy ? '✅' : '❌'}`);
    
    if (!qdrantHealthy) {
      console.error('  - Start Qdrant: docker run -p 127.0.0.1:6333:6333 qdrant/qdrant');
    }
    if (!ollamaHealthy) {
      console.error('  - Start Ollama and install model: ollama pull nomic-embed-text');
    }
    
    throw new Error('Required services not available for integration tests');
  }
  return servicesAvailable;
}

// Test isolation utilities
export interface TestEnvironment {
  collection: string;
  dataDir: string;
  env: Record<string, string>;
  cleanup: () => Promise<void>;
}

export async function createIsolatedTestEnvironment(prefix: string): Promise<TestEnvironment> {
  const collection = `directory-indexer-test-${prefix}-${Date.now()}`;
  const dataDir = await createTempTestDirectory();
  
  const originalEnv = {
    collection: process.env.DIRECTORY_INDEXER_QDRANT_COLLECTION,
    dataDir: process.env.DIRECTORY_INDEXER_DATA_DIR
  };

  const env = {
    DIRECTORY_INDEXER_QDRANT_COLLECTION: collection,
    DIRECTORY_INDEXER_DATA_DIR: dataDir
  };

  // Set environment for this test
  Object.assign(process.env, env);

  const cleanup = async () => {
    const cleanupTasks: Array<Promise<void>> = [];
    
    // 1. Clean up Qdrant collection (gracefully handle failures)
    cleanupTasks.push(
      (async () => {
        try {
          const config = await loadConfig({ verbose: false });
          const qdrant = new QdrantClient(config);
          
          // Only attempt cleanup if Qdrant is healthy
          if (await qdrant.healthCheck()) {
            await qdrant.deleteCollection();
            console.log(`🧹 Cleaned up Qdrant collection: ${collection}`);
          } else {
            console.log(`⚠️ Qdrant unavailable, skipping collection cleanup: ${collection}`);
          }
        } catch (error) {
          // Log error but don't fail the test
          console.log(`⚠️ Failed to cleanup Qdrant collection ${collection}:`, (error as Error).message);
        }
      })()
    );

    // 2. Clean up temp directory
    cleanupTasks.push(
      cleanupTempDirectory(dataDir).catch(error => {
        console.log(`⚠️ Failed to cleanup temp directory ${dataDir}:`, (error as Error).message);
      })
    );

    // 3. Restore environment variables (synchronous, always succeeds)
    if (originalEnv.collection) {
      process.env.DIRECTORY_INDEXER_QDRANT_COLLECTION = originalEnv.collection;
    } else {
      delete process.env.DIRECTORY_INDEXER_QDRANT_COLLECTION;
    }
    
    if (originalEnv.dataDir) {
      process.env.DIRECTORY_INDEXER_DATA_DIR = originalEnv.dataDir;
    } else {
      delete process.env.DIRECTORY_INDEXER_DATA_DIR;
    }

    // Wait for all cleanup tasks to complete (but don't propagate failures)
    await Promise.allSettled(cleanupTasks);
  };

  return { collection, dataDir, env, cleanup };
}

export async function runCLIWithLogging(
  args: string[], 
  env: Record<string, string>, 
  timeout = 30000
): Promise<{ stdout: string; stderr: string; exitCode: number }> {
  const result = await runCLI(args, timeout, env);
  
  if (result.exitCode !== 0) {
    console.log(`CLI command failed: ${args.join(' ')}`);
    console.log('stdout:', result.stdout);
    console.log('stderr:', result.stderr);
  }
  
  return result;
}

export function expectCLISuccess(result: { exitCode: number; stdout: string; stderr: string }) {
  if (result.exitCode !== 0) {
    throw new Error(`CLI command failed with exit code ${result.exitCode}\nstdout: ${result.stdout}\nstderr: ${result.stderr}`);
  }
}

// Cleanup verification utilities
export async function listQdrantTestCollections(): Promise<string[]> {
  try {
    const response = await fetch('http://localhost:6333/collections');
    if (!response.ok) return [];
    
    const data = await response.json();
    const collections = data.result?.collections || [];
    
    // Filter for test collections only
    return collections
      .map((c: any) => c.name)
      .filter((name: string) => name.startsWith('directory-indexer-test-'));
  } catch {
    return [];
  }
}

export async function verifyCleanupComplete(): Promise<{ success: boolean; orphanedCollections: string[] }> {
  const testCollections = await listQdrantTestCollections();
  
  if (testCollections.length > 0) {
    console.log(`⚠️ Found ${testCollections.length} orphaned test collections:`, testCollections);
    return { success: false, orphanedCollections: testCollections };
  }
  
  return { success: true, orphanedCollections: [] };
}

export async function forceCleanupOrphanedCollections(): Promise<void> {
  const testCollections = await listQdrantTestCollections();
  
  if (testCollections.length === 0) {
    console.log('✅ No orphaned test collections found');
    return;
  }
  
  console.log(`🧹 Force cleaning ${testCollections.length} orphaned test collections...`);
  
  const cleanupPromises = testCollections.map(async (collectionName) => {
    try {
      const response = await fetch(`http://localhost:6333/collections/${collectionName}`, {
        method: 'DELETE'
      });
      
      if (response.ok || response.status === 404) {
        console.log(`🗑️ Deleted orphaned collection: ${collectionName}`);
      } else {
        console.log(`⚠️ Failed to delete collection ${collectionName}: ${response.statusText}`);
      }
    } catch (error) {
      console.log(`⚠️ Error deleting collection ${collectionName}:`, (error as Error).message);
    }
  });
  
  await Promise.allSettled(cleanupPromises);
  console.log('✅ Force cleanup completed');
}