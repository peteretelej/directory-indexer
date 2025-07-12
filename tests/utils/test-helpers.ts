import { spawn } from 'child_process';
import { join } from 'path';
import { promises as fs } from 'fs';

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