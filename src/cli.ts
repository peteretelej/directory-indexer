#!/usr/bin/env node

import { Command } from 'commander';
import { fileURLToPath } from 'url';
import { readFileSync } from 'fs';
import { join, dirname } from 'path';
import { 
  handleIndex, 
  handleSearch, 
  handleSimilar, 
  handleGet, 
  handleServe, 
  handleReset, 
  handleStatus 
} from './cli-handlers.js';

// Read version from package.json
const __dirname = dirname(fileURLToPath(import.meta.url));
const packageJsonPath = join(__dirname, '../package.json');
const packageJson = JSON.parse(readFileSync(packageJsonPath, 'utf-8'));
const VERSION = packageJson.version;

export async function main() {
  const program = new Command();
  
  program
    .name('directory-indexer')
    .description('AI-powered directory indexing with semantic search')
    .version(VERSION);

  program
    .command('index')
    .description('Index directories for semantic search')
    .argument('<paths...>', 'Directory paths to index')
    .option('-v, --verbose', 'Enable verbose logging')
    .action(async (paths: string[], options) => {
      try {
        await handleIndex(paths, options);
      } catch (error) {
        console.error('Error indexing directories:', error);
        process.exit(1);
      }
    });

  program
    .command('search')
    .description('Search indexed content semantically')
    .argument('<query>', 'Search query')
    .option('-l, --limit <number>', 'Maximum number of results', '10')
    .option('-c, --show-chunks', 'Show individual chunk scores and IDs')
    .option('-v, --verbose', 'Enable verbose logging')
    .action(async (query: string, options) => {
      try {
        await handleSearch(query, {
          limit: parseInt(options.limit),
          showChunks: options.showChunks,
          verbose: options.verbose
        });
      } catch (error) {
        console.error('Error searching content:', error);
        process.exit(1);
      }
    });

  program
    .command('similar')
    .description('Find files similar to a given file')
    .argument('<file>', 'File path to find similar files for')
    .option('-l, --limit <number>', 'Maximum number of results', '10')
    .option('-v, --verbose', 'Enable verbose logging')
    .action(async (filePath: string, options) => {
      try {
        await handleSimilar(filePath, {
          limit: parseInt(options.limit),
          verbose: options.verbose
        });
      } catch (error) {
        console.error('Error finding similar files:', error);
        process.exit(1);
      }
    });

  program
    .command('get')
    .description('Get file content')
    .argument('<file>', 'File path to retrieve')
    .option('-c, --chunks <range>', 'Chunk range (e.g., "2-5")')
    .option('-v, --verbose', 'Enable verbose logging')
    .action(async (filePath: string, options) => {
      try {
        await handleGet(filePath, options);
      } catch (error) {
        console.error('Error getting file content:', error);
        process.exit(1);
      }
    });

  program
    .command('serve')
    .description('Start MCP server')
    .option('-v, --verbose', 'Enable verbose logging')
    .action(async (options) => {
      try {
        await handleServe(options);
      } catch (error) {
        console.error('Error starting MCP server:', error);
        process.exit(1);
      }
    });

  program
    .command('reset')
    .description('Reset directory-indexer data (database and vector collection)')
    .option('--force', 'Skip confirmation prompt')
    .option('-v, --verbose', 'Enable verbose logging')
    .action(async (options) => {
      try {
        await handleReset(options);
      } catch (error) {
        if (error instanceof Error && error.message === 'Reset cancelled by user') {
          console.log('\nReset cancelled.');
          process.exit(0);
        }
        console.error('Error during reset:', error);
        process.exit(1);
      }
    });

  program
    .command('status')
    .description('Show indexing status')
    .option('-v, --verbose', 'Enable verbose logging')
    .action(async (options) => {
      try {
        await handleStatus(options);
      } catch (error) {
        console.error('Error getting status:', error);
        process.exit(1);
      }
    });

  await program.parseAsync();
}

// Main function is already exported above