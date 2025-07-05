import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  define: {
    __dirname: 'import.meta.dirname',
  },
  build: {
    lib: {
      entry: {
        cli: resolve(__dirname, 'src/cli.ts'),
        config: resolve(__dirname, 'src/config.ts'),
        storage: resolve(__dirname, 'src/storage.ts'),
        utils: resolve(__dirname, 'src/utils.ts'),
        embedding: resolve(__dirname, 'src/embedding.ts'),
        indexing: resolve(__dirname, 'src/indexing.ts'),
        search: resolve(__dirname, 'src/search.ts'),
      },
      formats: ['es'],
    },
    target: 'node18',
    outDir: 'dist',
    sourcemap: true,
    minify: false,
    ssr: true,
    rollupOptions: {
      external: [
        'better-sqlite3', 
        'commander', 
        'glob', 
        'mime-types', 
        'zod', 
        '@modelcontextprotocol/sdk',
        'node:fs',
        'node:path', 
        'node:os',
        'node:crypto',
        'node:util',
        'fs',
        'path',
        'os', 
        'crypto',
        'util'
      ],
      output: {
        entryFileNames: '[name].js',
      },
    },
  },
  test: {
    globals: true,
    environment: 'node',
    testTimeout: 30000,
    hookTimeout: 30000,
    teardownTimeout: 30000,
    isolate: true,
    pool: 'threads',
    poolOptions: {
      threads: {
        singleThread: false,
      },
    },
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
});