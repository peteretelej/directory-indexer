#!/usr/bin/env node

import { Command } from 'commander';

export async function main() {
  const program = new Command();
  
  program
    .name('directory-indexer')
    .description('AI-powered directory indexing with semantic search')
    .version('0.0.10');

  // TODO: Add commands
  // - index <paths...>
  // - search <query>
  // - similar <file>
  // - get <file>
  // - serve
  // - status

  await program.parseAsync();
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(console.error);
}