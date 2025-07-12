import { defineConfig } from 'vitest/config';
import { resolve, dirname } from 'path';
import { readdirSync, statSync } from 'fs';
import { join } from 'path';
import { fileURLToPath } from 'url';

// Get __dirname equivalent in ES modules
const __dirname = dirname(fileURLToPath(import.meta.url));

// Automatically discover all TypeScript files in src/
function getEntryPoints() {
  const entries: Record<string, string> = {};
  
  function scanDir(dir: string, basePath = '') {
    const items = readdirSync(dir);
    
    for (const item of items) {
      const fullPath = join(dir, item);
      const stat = statSync(fullPath);
      
      if (stat.isDirectory()) {
        scanDir(fullPath, basePath ? `${basePath}/${item}` : item);
      } else if (item.endsWith('.ts') && !item.endsWith('.d.ts')) {
        const name = basePath 
          ? `${basePath}/${item.replace('.ts', '')}`.replace(/\//g, '-')
          : item.replace('.ts', '');
        
        entries[name] = resolve(__dirname, fullPath);
      }
    }
  }
  
  scanDir(resolve(__dirname, 'src'));
  return entries;
}

export default defineConfig({
  define: {
    __dirname: 'import.meta.dirname',
  },
  build: {
    lib: {
      entry: getEntryPoints(),
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
        'scripts/**',
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