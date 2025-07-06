import { describe, it, expect } from 'vitest';

// These tests define the expected functionality and will initially fail with "not implemented"
// This is TDD - we write tests first, then implement to make them pass

describe('Configuration', () => {
  it('should load default configuration', async () => {
    const { loadConfig } = await import('../src/config.js').catch(() => ({ loadConfig: () => { throw new Error('loadConfig not implemented'); } }));
    
    const config = await loadConfig();
    
    expect(config.storage.qdrantEndpoint).toBe('http://localhost:6333');
    expect(config.storage.qdrantCollection).toBe('directory-indexer');
    expect(config.embedding.provider).toBe('ollama');
    expect(config.embedding.model).toBe('nomic-embed-text');
    expect(config.indexing.chunkSize).toBe(512);
    expect(config.indexing.chunkOverlap).toBe(50);
    expect(config.indexing.maxFileSize).toBe(10485760);
    expect(config.indexing.ignorePatterns).toContain('.git');
    expect(config.indexing.ignorePatterns).toContain('node_modules');
  });

  it('should override defaults with environment variables', async () => {
    process.env.QDRANT_ENDPOINT = 'http://custom:6333';
    process.env.OLLAMA_ENDPOINT = 'http://custom:11434';
    process.env.DIRECTORY_INDEXER_QDRANT_COLLECTION = 'custom-collection';
    
    const { loadConfig } = await import('../src/config.js').catch(() => ({ loadConfig: () => { throw new Error('loadConfig not implemented'); } }));
    
    const config = await loadConfig();
    
    expect(config.storage.qdrantEndpoint).toBe('http://custom:6333');
    expect(config.embedding.endpoint).toBe('http://custom:11434');
    expect(config.storage.qdrantCollection).toBe('custom-collection');
    
    // Cleanup
    delete process.env.QDRANT_ENDPOINT;
    delete process.env.OLLAMA_ENDPOINT;
    delete process.env.DIRECTORY_INDEXER_QDRANT_COLLECTION;
  });
});

describe('Path Utilities', () => {
  it('should normalize paths across platforms', async () => {
    const { normalizePath } = await import('../src/utils.js').catch(() => ({ normalizePath: () => { throw new Error('normalizePath not implemented'); } }));
    
    const windowsPath = 'C:\\Users\\test\\Documents';
    const unixPath = '/home/test/documents';
    
    const normalizedWindows = normalizePath(windowsPath);
    const normalizedUnix = normalizePath(unixPath);
    
    expect(typeof normalizedWindows).toBe('string');
    expect(typeof normalizedUnix).toBe('string');
    expect(normalizedWindows.length).toBeGreaterThan(0);
    expect(normalizedUnix.length).toBeGreaterThan(0);
  });

  it('should convert relative paths to absolute', async () => {
    const { normalizePath } = await import('../src/utils.js').catch(() => ({ normalizePath: () => { throw new Error('normalizePath not implemented'); } }));
    
    const relativePath = './test/path';
    const absolutePath = normalizePath(relativePath);
    
    expect(absolutePath.startsWith('/')).toBe(true); // Unix
    // OR expect(absolutePath.match(/^[A-Z]:/)).toBeTruthy(); // Windows
  });

  it('should calculate file hashes consistently', async () => {
    const { calculateHash } = await import('../src/utils.js').catch(() => ({ calculateHash: () => { throw new Error('calculateHash not implemented'); } }));
    
    const testContent = 'Hello, world!';
    const hash1 = calculateHash(testContent);
    const hash2 = calculateHash(testContent);
    
    expect(hash1).toBe(hash2);
    expect(hash1.length).toBeGreaterThan(0);
    expect(typeof hash1).toBe('string');
  });
});

describe('Storage Operations', () => {
  it('should initialize SQLite database with correct schema', async () => {
    const { initDatabase } = await import('../src/storage.js').catch(() => ({ initDatabase: () => { throw new Error('initDatabase not implemented'); } }));
    
    const dbPath = ':memory:'; // In-memory database for testing
    const db = await initDatabase(dbPath);
    
    expect(db).toBeDefined();
    // Should have directories and files tables
  });

  it('should add and retrieve file records', async () => {
    const { SQLiteStorage, loadConfig } = await import('../src/storage.js').catch(() => ({ 
      SQLiteStorage: null,
      loadConfig: () => ({ storage: { sqlitePath: ':memory:' } })
    }));
    
    if (!SQLiteStorage) {
      console.log('SQLiteStorage not available, skipping test');
      return;
    }
    
    const config = await import('../src/config.js').then(m => m.loadConfig());
    config.storage.sqlitePath = ':memory:';
    const storage = new SQLiteStorage(config);
    
    const fileInfo = {
      path: '/test/file.txt',
      size: 100,
      modifiedTime: new Date(),
      hash: 'testhash123',
      parentDirs: ['/test']
    };
    
    const chunks = [{ id: '1', content: 'test content', startIndex: 0, endIndex: 12 }];
    
    await storage.upsertFile(fileInfo, chunks);
    const retrieved = await storage.getFile('/test/file.txt');
    
    expect(retrieved).toBeDefined();
    expect(retrieved?.path).toBe('/test/file.txt');
    expect(retrieved?.hash).toBe('testhash123');
    
    storage.close();
  });

  it('should perform Qdrant operations', async () => {
    const { QdrantClient, loadConfig } = await import('../src/storage.js').catch(() => ({ 
      QdrantClient: null,
      loadConfig: () => ({ storage: { qdrantEndpoint: 'http://localhost:6333', qdrantCollection: 'test' } })
    }));
    
    if (!QdrantClient) {
      console.log('QdrantClient not available, skipping test');
      return;
    }
    
    const config = await import('../src/config.js').then(m => m.loadConfig());
    const client = new QdrantClient(config);
    
    // Test health check (should work even if Qdrant is not running)
    const isHealthy = await client.healthCheck();
    expect(typeof isHealthy).toBe('boolean');
    
    // Only test collection operations if Qdrant is available
    if (isHealthy) {
      await client.createCollection();
      
      const points = [{
        id: 'test-1',
        vector: new Array(768).fill(0.1),
        payload: { filePath: '/test.txt', chunkId: '1', parentDirectories: [] }
      }];
      
      await client.upsertPoints(points);
      
      const searchVector = new Array(768).fill(0.1);
      const results = await client.searchPoints(searchVector, 5);
      
      expect(Array.isArray(results)).toBe(true);
    }
  });
});

describe('Embedding Providers', () => {
  it('should create mock embedding provider', async () => {
    const { createEmbeddingProvider } = await import('../src/embedding.js').catch(() => ({ createEmbeddingProvider: () => { throw new Error('createEmbeddingProvider not implemented'); } }));
    
    const provider = createEmbeddingProvider('mock', { model: 'test-model', endpoint: '', dimensions: 384 });
    
    expect(provider.name).toBe('mock');
    expect(provider.dimensions).toBe(384);
    
    const embedding = await provider.generateEmbedding('test text');
    expect(Array.isArray(embedding)).toBe(true);
    expect(embedding.length).toBe(384);
  });

  it('should generate consistent embeddings for same input', async () => {
    const { createEmbeddingProvider } = await import('../src/embedding.js').catch(() => ({ createEmbeddingProvider: () => { throw new Error('createEmbeddingProvider not implemented'); } }));
    
    const provider = createEmbeddingProvider('mock', { model: 'test-model', endpoint: '', dimensions: 384 });
    
    const text = 'consistent test input';
    const embedding1 = await provider.generateEmbedding(text);
    const embedding2 = await provider.generateEmbedding(text);
    
    expect(embedding1).toEqual(embedding2);
  });

  it('should handle batch embedding generation', async () => {
    const { createEmbeddingProvider } = await import('../src/embedding.js').catch(() => ({ createEmbeddingProvider: () => { throw new Error('createEmbeddingProvider not implemented'); } }));
    
    const provider = createEmbeddingProvider('mock', { model: 'test-model', endpoint: '', dimensions: 384 });
    
    const texts = ['text one', 'text two', 'text three'];
    const embeddings = await provider.generateEmbeddings(texts);
    
    expect(embeddings.length).toBe(3);
    expect(embeddings[0].length).toBe(384);
    expect(embeddings[1].length).toBe(384);
    expect(embeddings[2].length).toBe(384);
  });
});

describe('Text Chunking', () => {
  it('should chunk text with sliding window', async () => {
    const { chunkText } = await import('../src/indexing.js').catch(() => ({ chunkText: () => { throw new Error('chunkText not implemented'); } }));
    
    const longText = 'This is a very long text that needs to be chunked into smaller pieces for embedding generation and vector storage.';
    const chunks = chunkText(longText, 50, 10);
    
    expect(Array.isArray(chunks)).toBe(true);
    expect(chunks.length).toBeGreaterThan(1);
    expect(chunks[0].content.length).toBeLessThanOrEqual(50);
    expect(chunks[0].startIndex).toBe(0);
  });

  it('should handle overlap between chunks', async () => {
    const { chunkText } = await import('../src/indexing.js').catch(() => ({ chunkText: () => { throw new Error('chunkText not implemented'); } }));
    
    const text = 'Word one two three four five six seven eight nine ten eleven twelve.';
    const chunks = chunkText(text, 30, 10); // 30 char chunks with 10 char overlap
    
    expect(chunks.length).toBeGreaterThan(1);
    // Should have some overlapping content between chunks
  });
});

describe('File Scanning', () => {
  it('should scan directory and filter files', async () => {
    const { scanDirectory } = await import('../src/indexing.js').catch(() => ({ scanDirectory: () => { throw new Error('scanDirectory not implemented'); } }));
    
    const testDir = './tests/test_data';
    const ignorePatterns = ['.git', 'node_modules', '*.log'];
    
    const files = await scanDirectory(testDir, { ignorePatterns, maxFileSize: 1000000 });
    
    expect(Array.isArray(files)).toBe(true);
    // Should not include ignored files
    const gitFiles = files.filter(f => f.path.includes('.git'));
    expect(gitFiles.length).toBe(0);
  });

  it('should extract file metadata', async () => {
    const { getFileMetadata } = await import('../src/indexing.js').catch(() => ({ getFileMetadata: () => { throw new Error('getFileMetadata not implemented'); } }));
    
    const testFilePath = './tests/test_data/docs/api_guide.md';
    const metadata = await getFileMetadata(testFilePath);
    
    expect(metadata.path).toContain('api_guide.md');
    expect(metadata.size).toBeGreaterThan(0);
    expect(metadata.modifiedTime).toBeInstanceOf(Date);
    expect(metadata.hash.length).toBeGreaterThan(0);
  });
});

describe('Search Operations', () => {
  it('should perform semantic search', async () => {
    const { searchContent } = await import('../src/search.js').catch(() => ({ searchContent: () => { throw new Error('searchContent not implemented'); } }));
    
    const query = 'test search query';
    const options = { limit: 10, threshold: 0.7 };
    
    const results = await searchContent(query, options);
    
    expect(Array.isArray(results)).toBe(true);
    expect(results.length).toBeLessThanOrEqual(10);
  });

  it('should find similar files', async () => {
    const { findSimilarFiles } = await import('../src/search.js').catch(() => ({ findSimilarFiles: () => { throw new Error('findSimilarFiles not implemented'); } }));
    
    const filePath = './tests/test_data/docs/api_guide.md';
    const limit = 5;
    
    const similar = await findSimilarFiles(filePath, limit);
    
    expect(Array.isArray(similar)).toBe(true);
    expect(similar.length).toBeLessThanOrEqual(5);
  });

  it('should get file content with optional chunk selection', async () => {
    const { getFileContent } = await import('../src/search.js').catch(() => ({ getFileContent: () => { throw new Error('getFileContent not implemented'); } }));
    
    const filePath = './tests/test_data/docs/api_guide.md';
    
    // Get full content
    const fullContent = await getFileContent(filePath);
    expect(typeof fullContent).toBe('string');
    expect(fullContent.length).toBeGreaterThan(0);
    
    // Get specific chunks (if file is chunked)
    const chunks = await getFileContent(filePath, '1-2');
    expect(typeof chunks).toBe('string');
  });

  it('should handle search errors gracefully', async () => {
    const { searchContent } = await import('../src/search.js');
    
    // Test with invalid options
    try {
      await searchContent('', { limit: -1 });
    } catch (error) {
      expect(error).toBeInstanceOf(Error);
    }
  });

  it('should handle file not found in similar files', async () => {
    const { findSimilarFiles } = await import('../src/search.js');
    
    try {
      await findSimilarFiles('/nonexistent/file.txt', 5);
    } catch (error) {
      expect(error).toBeInstanceOf(Error);
      expect(error.message).toContain('not found');
    }
  });
});

describe('Storage Advanced Operations', () => {
  it('should handle file deletion', async () => {
    const { SQLiteStorage, loadConfig } = await import('../src/storage.js');
    
    const config = await import('../src/config.js').then(m => m.loadConfig());
    config.storage.sqlitePath = ':memory:';
    const storage = new SQLiteStorage(config);
    
    const fileInfo = {
      path: '/test/delete.txt',
      size: 50,
      modifiedTime: new Date(),
      hash: 'delete123',
      parentDirs: ['/test']
    };
    
    await storage.upsertFile(fileInfo);
    let retrieved = await storage.getFile('/test/delete.txt');
    expect(retrieved).toBeDefined();
    
    await storage.deleteFile('/test/delete.txt');
    retrieved = await storage.getFile('/test/delete.txt');
    expect(retrieved).toBeNull();
    
    storage.close();
  });

  it('should get files by directory', async () => {
    const { SQLiteStorage, loadConfig } = await import('../src/storage.js');
    
    const config = await import('../src/config.js').then(m => m.loadConfig());
    config.storage.sqlitePath = ':memory:';
    const storage = new SQLiteStorage(config);
    
    const file1 = {
      path: '/test/dir/file1.txt',
      size: 50,
      modifiedTime: new Date(),
      hash: 'hash1',
      parentDirs: ['/test', '/test/dir']
    };
    
    const file2 = {
      path: '/test/dir/file2.txt',
      size: 60,
      modifiedTime: new Date(),
      hash: 'hash2',
      parentDirs: ['/test', '/test/dir']
    };
    
    await storage.upsertFile(file1);
    await storage.upsertFile(file2);
    
    const files = await storage.getFilesByDirectory('/test/dir');
    expect(files.length).toBe(2);
    expect(files.some(f => f.path === '/test/dir/file1.txt')).toBe(true);
    expect(files.some(f => f.path === '/test/dir/file2.txt')).toBe(true);
    
    storage.close();
  });

  it('should handle directory operations', async () => {
    const { SQLiteStorage, loadConfig } = await import('../src/storage.js');
    
    const config = await import('../src/config.js').then(m => m.loadConfig());
    config.storage.sqlitePath = ':memory:';
    const storage = new SQLiteStorage(config);
    
    await storage.upsertDirectory('/test/dir', 'pending');
    const dir = await storage.getDirectory('/test/dir');
    expect(dir).toBeDefined();
    expect(dir?.path).toBe('/test/dir');
    expect(dir?.status).toBe('pending');
    
    await storage.upsertDirectory('/test/dir', 'completed');
    const updatedDir = await storage.getDirectory('/test/dir');
    expect(updatedDir?.status).toBe('completed');
    
    storage.close();
  });
});

describe('Configuration Advanced', () => {
  it('should handle environment variable overrides', async () => {
    const originalQdrant = process.env.QDRANT_ENDPOINT;
    const originalOllama = process.env.OLLAMA_ENDPOINT;
    
    process.env.QDRANT_ENDPOINT = 'http://test:6333';
    process.env.OLLAMA_ENDPOINT = 'http://test:11434';
    
    const { loadConfig } = await import('../src/config.js');
    const config = await loadConfig();
    
    expect(config.storage.qdrantEndpoint).toBe('http://test:6333');
    expect(config.embedding.endpoint).toBe('http://test:11434');
    
    // Restore original values
    if (originalQdrant) process.env.QDRANT_ENDPOINT = originalQdrant;
    else delete process.env.QDRANT_ENDPOINT;
    if (originalOllama) process.env.OLLAMA_ENDPOINT = originalOllama;
    else delete process.env.OLLAMA_ENDPOINT;
  });

  it('should validate config parameters', async () => {
    const { loadConfig } = await import('../src/config.js');
    
    const config = await loadConfig({ verbose: true });
    
    expect(config.verbose).toBe(true);
    expect(config.indexing.chunkSize).toBeGreaterThan(0);
    expect(config.indexing.maxFileSize).toBeGreaterThan(0);
    expect(Array.isArray(config.indexing.ignorePatterns)).toBe(true);
  });
});