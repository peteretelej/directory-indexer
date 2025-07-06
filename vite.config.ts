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
    outputFile: {
      junit: './test-report.junit.xml'
    },
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'json', 'lcov'],
      include: [
        'src/**/*.ts'
      ],
      exclude: [
        'coverage/**',
        'dist/**',
        'tests/**/*.test.ts',
        '**/*.d.ts',
        'vite.config.ts',
        'eslint.config.js'
      ],
      thresholds: {
        statements: 50,
        branches: 60,
        functions: 70,
        lines: 50
      }
    },
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
});