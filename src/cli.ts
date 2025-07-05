#!/usr/bin/env node

import { Command } from 'commander';
import { indexDirectories } from './indexing.js';
import { searchContent, findSimilarFiles, getFileContent } from './search.js';
import { loadConfig } from './config.js';
import { getIndexStatus } from './storage.js';
import { startMcpServer } from './mcp.js';

export async function main() {
  const program = new Command();
  
  program
    .name('directory-indexer')
    .description('AI-powered directory indexing with semantic search')
    .version('0.0.10');

  program
    .command('index')
    .description('Index directories for semantic search')
    .argument('<paths...>', 'Directory paths to index')
    .option('-v, --verbose', 'Enable verbose logging')
    .action(async (paths: string[], options) => {
      try {
        const config = await loadConfig({ verbose: options.verbose });
        console.log(`Indexing ${paths.length} directories...`);
        const result = await indexDirectories(paths, config);
        console.log(`Indexed ${result.filesProcessed} files, ${result.chunksCreated} chunks, ${result.errors.length} errors`);
        if (result.errors.length > 0 && config.verbose) {
          console.log('Errors:', result.errors);
        }
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
    .option('-v, --verbose', 'Enable verbose logging')
    .action(async (query: string, options) => {
      try {
        const config = await loadConfig({ verbose: options.verbose });
        const results = await searchContent(query, { limit: parseInt(options.limit) });
        
        if (results.length === 0) {
          console.log('No results found');
          return;
        }

        console.log(`Found ${results.length} results:\n`);
        results.forEach((result, index) => {
          console.log(`${index + 1}. ${result.filePath}`);
          console.log(`   Score: ${result.score.toFixed(3)}`);
          if (result.chunk) {
            console.log(`   Chunk: ${result.chunk.substring(0, 150)}...`);
          }
          console.log();
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
        const config = await loadConfig({ verbose: options.verbose });
        const results = await findSimilarFiles(filePath, parseInt(options.limit));
        
        if (results.length === 0) {
          console.log('No similar files found');
          return;
        }

        console.log(`Found ${results.length} similar files:\n`);
        results.forEach((result, index) => {
          console.log(`${index + 1}. ${result.filePath}`);
          console.log(`   Similarity: ${result.similarity.toFixed(3)}`);
          console.log();
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
        const config = await loadConfig({ verbose: options.verbose });
        const content = await getFileContent(filePath, options.chunks);
        console.log(content);
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
        const config = await loadConfig({ verbose: options.verbose });
        await startMcpServer(config);
      } catch (error) {
        console.error('Error starting MCP server:', error);
        process.exit(1);
      }
    });

  program
    .command('status')
    .description('Show indexing status')
    .option('-v, --verbose', 'Enable verbose logging')
    .action(async (options) => {
      try {
        const config = await loadConfig({ verbose: options.verbose });
        const status = await getIndexStatus();
        
        console.log('Indexing Status:');
        console.log(`  Directories indexed: ${status.directoriesIndexed}`);
        console.log(`  Files indexed: ${status.filesIndexed}`);
        console.log(`  Total chunks: ${status.chunksIndexed}`);
        console.log(`  Database size: ${status.databaseSize}`);
        console.log(`  Last indexed: ${status.lastIndexed || 'Never'}`);
        
        if (status.errors.length > 0) {
          console.log(`  Errors: ${status.errors.length}`);
          if (options.verbose) {
            console.log('  Recent errors:');
            status.errors.slice(0, 5).forEach(error => {
              console.log(`    - ${error}`);
            });
          }
        }
      } catch (error) {
        console.error('Error getting status:', error);
        process.exit(1);
      }
    });

  await program.parseAsync();
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(console.error);
}