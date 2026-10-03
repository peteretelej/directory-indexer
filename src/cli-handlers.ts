import { indexDirectories } from './indexing.js';
import { searchContent, findSimilarFiles, getFileContent } from './search.js';
import { loadConfig } from './config.js';
import { getIndexStatus } from './storage.js';
import { startMcpServer } from './mcp.js';
import { validateIndexPrerequisites, validateSearchPrerequisites, getServiceStatus } from './prerequisites.js';
import { resetEnvironment } from './reset.js';

export interface IndexOptions {
  verbose?: boolean;
}

export interface SearchOptions {
  limit?: number;
  showChunks?: boolean;
  verbose?: boolean;
}

export interface SimilarOptions {
  limit?: number;
  verbose?: boolean;
}

export interface GetOptions {
  chunks?: string;
  verbose?: boolean;
}

export interface ServeOptions {
  verbose?: boolean;
}

export interface ResetOptions {
  force?: boolean;
  verbose?: boolean;
}

export interface StatusOptions {
  verbose?: boolean;
}

export async function handleIndex(paths: string[], options: IndexOptions): Promise<void> {
  const config = await loadConfig({ verbose: options.verbose });
  await validateIndexPrerequisites(config);
  
  console.log(`Indexing ${paths.length} ${paths.length === 1 ? 'directory' : 'directories'}: ${paths.join(', ')}`);
  if (!config.verbose) {
    console.log('Run with --verbose for detailed per-file indexing reports');
    console.log('Indexing can be safely stopped and resumed - progress is automatically saved');
    console.log('You can start using the MCP server while indexing continues');
    console.log('Indexing may take time due to embedding generation - see project README for performance tips');
  }
  
  const result = await indexDirectories(paths, config, {
    progress: m => console.log(m),
    error: m => console.error(m)
  });
  console.log(`Indexed ${result.indexed} files, skipped ${result.skipped} files, cleaned up ${result.deleted} deleted files, ${result.failed} failed`);
  
  if (result.errors.length > 0) {
    console.log(`Errors: [`);
    result.errors.forEach(error => {
      console.log(`  '${error}'`);
    });
    console.log(`]`);
  }
}

export async function handleSearch(query: string, options: SearchOptions): Promise<void> {
  const config = await loadConfig({ verbose: options.verbose });
  await validateSearchPrerequisites(config);
  
  const results = await searchContent(query, { limit: options.limit || 10 });
  
  if (results.length === 0) {
    console.log('No results found');
    return;
  }

  console.log(`Found ${results.length} results:\n`);
  results.forEach((result, index) => {
    console.log(`${index + 1}. ${result.filePath}`);
    console.log(`   Score: ${result.score.toFixed(3)} (${result.matchingChunks} chunks)`);
    
    if (options.showChunks && result.chunks.length > 0) {
      console.log(`   Chunks:`);
      result.chunks.forEach(chunk => {
        console.log(`     - Chunk ${chunk.chunkId}: ${chunk.score.toFixed(3)}`);
      });
    }
    
    console.log();
  });
}

export async function handleSimilar(filePath: string, options: SimilarOptions): Promise<void> {
  const config = await loadConfig({ verbose: options.verbose });
  await validateSearchPrerequisites(config);
  
  const results = await findSimilarFiles(filePath, options.limit || 10);
  
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
}

export async function handleGet(filePath: string, options: GetOptions): Promise<void> {
  await loadConfig({ verbose: options.verbose });
  const content = await getFileContent(filePath, options.chunks);
  console.log(content);
}

export async function handleServe(options: ServeOptions): Promise<void> {
  const config = await loadConfig({ verbose: options.verbose });
  await startMcpServer(config);
}

export async function handleReset(options: ResetOptions): Promise<void> {
  const config = await loadConfig({ verbose: options.verbose });
  await resetEnvironment(config, { 
    force: options.force, 
    verbose: options.verbose 
  });
}

export async function handleStatus(options: StatusOptions): Promise<void> {
  const config = await loadConfig({ verbose: options.verbose });
  const [status, serviceStatus] = await Promise.all([
    getIndexStatus(),
    getServiceStatus(config)
  ]);
  
  console.log('Directory Indexer Status Report');
  console.log('=====================================');
  console.log('');
  console.log('SERVICE STATUS:');
  console.log(`  • Qdrant database: ${serviceStatus.qdrant ? 'Connected' : 'Disconnected'}`);
  console.log(`  • Embedding service (${serviceStatus.embeddingProvider}): ${serviceStatus.embedding ? 'Connected' : 'Disconnected'}`);
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
  
  if (status.workspaces.length > 0) {
    console.log('');
    console.log('WORKSPACES:');
    console.log(`  • ${status.workspaceHealth.healthy} healthy, ${status.workspaceHealth.warnings} warnings, ${status.workspaceHealth.errors} errors`);
    
    if (status.workspaceHealth.errors > 0) {
      console.log('');
      console.log('WORKSPACE ERRORS:');
      status.workspaceHealth.criticalIssues.forEach(issue => {
        console.log(`  ❌ ${issue}`);
      });
    }
    
    if (status.workspaceHealth.recommendations.length > 0) {
      console.log('');
      console.log('WORKSPACE RECOMMENDATIONS:');
      status.workspaceHealth.recommendations.forEach(rec => {
        console.log(`  💡 ${rec}`);
      });
    }
    
    if (options.verbose) {
      console.log('');
      console.log('WORKSPACE DETAILS:');
      status.workspaces.forEach(workspace => {
        console.log('');
        console.log(`  Workspace: ${workspace.name}`);
        console.log(`    • Status: ${workspace.health.status}`);
        console.log(`    • Paths: ${workspace.paths.join(', ')}`);
        console.log(`    • Files: ${workspace.filesCount}, Chunks: ${workspace.chunksCount}`);
        if (workspace.health.issues.length > 0) {
          console.log(`    • Issues: ${workspace.health.issues.join('; ')}`);
        }
      });
    }
  }

  if (!status.qdrantConsistency.isConsistent) {
    console.log('');
    console.log('SYSTEM STATUS:');
    status.qdrantConsistency.issues.forEach(issue => {
      console.log(`  • ${issue}`);
    });
    console.log('');
    console.log('Note: Status messages above may be normal during setup or active indexing.');
  } else {
    console.log('');
    console.log('SYSTEM STATUS:');
    console.log('  • All systems operational - ready for AI-powered search');
  }
}