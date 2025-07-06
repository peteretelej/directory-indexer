#!/usr/bin/env node
import('../dist/cli.js').then(({ main }) => {
  main().catch((error) => {
    console.error('CLI Error:', error);
    process.exit(1);
  });
}).catch((error) => {
  console.error('Failed to load CLI:', error);
  process.exit(1);
});