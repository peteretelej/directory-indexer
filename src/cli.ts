#!/usr/bin/env node

import { Command } from 'commander';
import { fileURLToPath } from 'url';
import { readFileSync } from 'fs';
import { join, dirname } from 'path';
import { indexDirectories } from './indexing.js';
import { searchContent, findSimilarFiles, getFileContent } from './search.js';
import { loadConfig } from './config.js';
import { getIndexStatus } from './storage.js';
import { startMcpServer } from './mcp.js';

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
        const config = await loadConfig({ verbose: options.verbose });
        console.log(`Indexing ${paths.length} directories...`);
        const result = await indexDirectories(paths, config);
        console.log(`Indexed ${result.indexed} files, skipped ${result.skipped} files, ${result.errors.length} errors`);
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
        await loadConfig({ verbose: options.verbose });
        const results = await searchContent(query, { limit: parseInt(options.limit) });
        
        if (results.length === 0) {
          console.log('No results found');
          return;
        }

        console.log(`Found ${results.length} results:\n`);
        results.forEach((result, index) => {
          console.log(`${index + 1}. ${result.filePath}`);
          console.log(`   Score: ${result.score.toFixed(3)}`);
          if (result.content) {
            console.log(`   Content: ${result.content.substring(0, 150)}...`);
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
        await loadConfig({ verbose: options.verbose });
        const results = await findSimilarFiles(filePath, parseInt(options.limit));
        
        if (results.length === 0) {
          console.log('No similar files found');
          return;
        }

        console.log(`Found ${results.length} similar files:\n`);
        results.forEach((result, index) => {
          console.log(`${index + 1}. ${result.filePath}`);
          console.log(`   Similarity: ${result.score.toFixed(3)}`);
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
        await loadConfig({ verbose: options.verbose });
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
        await loadConfig({ verbose: options.verbose });
        const status = await getIndexStatus();
        
        console.log('Directory Indexer Status Report');
        console.log('=====================================');
        console.log('');
        console.log('OVERVIEW:');
        console.log(`  • ${status.directoriesIndexed} directories have been indexed`);
        console.log(`  • ${status.filesIndexed} files processed for semantic search`);
        console.log(`  • ${status.chunksIndexed} text chunks available for AI search`);
        console.log(`  • Database storage: ${status.databaseSize}`);
        console.log(`  • Most recent indexing: ${status.lastIndexed || 'No indexing performed yet'}`);
        
        if (status.errors.length > 0) {
          console.log(`  • Processing errors encountered: ${status.errors.length}`);
          if (options.verbose) {
            console.log('');
            console.log('RECENT ERRORS:');
            status.errors.slice(0, 5).forEach(error => {
              console.log(`  - ${error}`);
            });
          }
        }
        
        console.log('');
        console.log('INDEXED DIRECTORIES:');
        if (status.directories.length === 0) {
          console.log('  No directories have been indexed yet.');
          console.log('  Run "directory-indexer index <path>" to start indexing.');
        } else {
          status.directories.forEach(dir => {
            console.log('');
            console.log(`  Directory: ${dir.path}`);
            console.log(`    • Indexing status: ${dir.status}`);
            console.log(`    • Files processed: ${dir.filesCount}`);
            console.log(`    • Searchable chunks: ${dir.chunksCount}`);
            console.log(`    • Last indexed: ${dir.lastIndexed || 'Never completed'}`);
            if (dir.errors.length > 0) {
              console.log(`    • Files with errors: ${dir.errors.length}`);
              if (options.verbose) {
                console.log('    • Recent errors:');
                dir.errors.slice(0, 3).forEach(error => {
                  console.log(`      - ${error}`);
                });
              }
            }
          });
        }
        
        if (!status.qdrantConsistency.isConsistent) {
          console.log('');
          console.log('SYSTEM STATUS:');
          status.qdrantConsistency.issues.forEach(issue => {
            console.log(`  • ${issue}`);
          });
          console.log('');
          console.log('ℹ️  Note: Status messages above may be normal during setup or active indexing.');
        } else {
          console.log('');
          console.log('SYSTEM STATUS:');
          console.log('  • All systems operational - ready for AI-powered search');
        }
      } catch (error) {
        console.error('Error getting status:', error);
        process.exit(1);
      }
    });

  await program.parseAsync();
}

// Main function is already exported above