import { readlineSync } from './utils.js';
import { getResetPreview, clearDatabase, clearVectorCollection, StorageError, type ResetStats } from './storage.js';
import type { Config } from './config.js';

export interface ResetOptions {
  force?: boolean;
  verbose?: boolean;
}

export async function resetEnvironment(config: Config, options: ResetOptions = {}): Promise<void> {
  const stats = await getResetPreview(config);
  
  if (!options.force) {
    await showConfirmation(stats, config);
  }
  
  await performReset(config, options);
}


async function showConfirmation(stats: ResetStats, config: Config): Promise<void> {
  console.log('\nThe following directory-indexer data will be reset:');
  
  if (stats.sqliteExists) {
    console.log(`  • SQLite database: ${config.storage.sqlitePath} (${stats.sqliteSize})`);
  } else {
    console.log(`  • SQLite database: ${config.storage.sqlitePath} (not found)`);
  }
  
  if (stats.qdrantCollectionExists) {
    const vectorText = stats.qdrantVectorCount === 1 ? 'vector' : 'vectors';
    console.log(`  • Qdrant collection: ${config.storage.qdrantCollection} (${stats.qdrantVectorCount} ${vectorText})`);
  } else {
    console.log(`  • Qdrant collection: ${config.storage.qdrantCollection} (not found)`);
  }
  
  if (config.storage.qdrantEndpoint !== 'http://127.0.0.1:6333') {
    console.log(`  • Qdrant endpoint: ${config.storage.qdrantEndpoint}`);
  }
  
  console.log('\nYour original files will not be touched.');
  
  const answer = await readlineSync('\nContinue? (y/N): ');
  if (!answer || !['y', 'yes'].includes(answer.toLowerCase().trim())) {
    throw new Error('Reset cancelled by user');
  }
}

async function performReset(config: Config, options: ResetOptions): Promise<void> {
  if (options.verbose) {
    console.log('\nResetting directory-indexer data...');
  }

  // Reset Qdrant collection first: if this fails, stop before touching SQLite
  // so metadata and vectors stay consistent and re-running reset can retry both.
  try {
    if (options.verbose) {
      console.log(`  ✓ Deleting Qdrant collection: ${config.storage.qdrantCollection}`);
    }
    const qdrantDeleted = await clearVectorCollection(config);
    if (options.verbose && qdrantDeleted) {
      console.log(`  ✓ Qdrant collection cleared`);
    }
  } catch (error) {
    if (options.verbose) {
      const message = error instanceof StorageError ? error.message : `Failed to clear collection: ${error}`;
      console.log(`  ⚠ ${message}`);
      console.log('\nSQLite database left untouched. Fix the issue and re-run `directory-indexer reset`.');
    }
    throw error;
  }

  // Reset SQLite database
  try {
    if (options.verbose) {
      console.log(`  ✓ Deleting SQLite database: ${config.storage.sqlitePath}`);
    }
    const sqliteDeleted = await clearDatabase(config);
    if (options.verbose && sqliteDeleted) {
      console.log(`  ✓ SQLite database cleared`);
    }
  } catch (error) {
    const message = error instanceof StorageError ? error.message : `Failed to clear database: ${error}`;
    console.log('\nReset incomplete:');
    console.log(`  ✓ Vector data deleted from Qdrant collection: ${config.storage.qdrantCollection}`);
    console.log(`  ⚠ SQLite cleanup failed: ${message}`);
    console.log('\nVector data is gone but SQLite metadata remains.');
    console.log('Fix the issue, then re-run `directory-indexer reset` to finish the reset.');
    throw error;
  }

  console.log('\nReset complete. Directory-indexer is ready for fresh indexing.');
}