import { defineConfig } from 'tsup';

export default defineConfig({
  entry: ['src/cli.ts'],
  format: ['esm'],
  dts: true,
  splitting: false,
  sourcemap: true,
  clean: true,
  minify: false,
  target: 'node18',
  banner: {
    js: '#!/usr/bin/env node',
  },
  external: ['better-sqlite3'],
  noExternal: ['@modelcontextprotocol/sdk'],
});