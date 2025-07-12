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
  
  if (config.storage.qdrantEndpoint !== 'http://localhost:6333') {
    console.log(`  • Qdrant endpoint: ${config.storage.qdrantEndpoint}`);
  }
  
  console.log('\nYour original files will not be touched.');
  
  const answer = await readlineSync('\nContinue? (y/N): ');
  if (!answer || !['y', 'yes'].includes(answer.toLowerCase().trim())) {
    throw new Error('Reset cancelled by user');
  }
}

async function performReset(config: Config, options: ResetOptions): Promise<void> {
  let sqliteDeleted = false;
  let qdrantDeleted = false;
  const warnings: string[] = [];

  if (options.verbose) {
    console.log('\nResetting directory-indexer data...');
  }

  // Reset SQLite database
  try {
    if (options.verbose) {
      console.log(`  ✓ Deleting SQLite database: ${config.storage.sqlitePath}`);
    }
    sqliteDeleted = await clearDatabase(config);
    if (options.verbose && sqliteDeleted) {
      console.log(`  ✓ SQLite database cleared`);
    }
  } catch (error) {
    const message = error instanceof StorageError ? error.message : `Failed to clear database: ${error}`;
    warnings.push(message);
    if (options.verbose) {
      console.log(`  ⚠ ${message}`);
    }
  }

  // Reset Qdrant collection
  try {
    if (options.verbose) {
      console.log(`  ✓ Deleting Qdrant collection: ${config.storage.qdrantCollection}`);
    }
    qdrantDeleted = await clearVectorCollection(config);
    if (options.verbose && qdrantDeleted) {
      console.log(`  ✓ Qdrant collection cleared`);
    }
  } catch (error) {
    const message = error instanceof StorageError ? error.message : `Failed to clear collection: ${error}`;
    warnings.push(message);
    if (options.verbose) {
      console.log(`  ⚠ ${message}`);
    }
  }

  // Show results
  if (warnings.length > 0) {
    console.log('\nReset completed with warnings:');
    warnings.forEach(warning => console.log(`  ⚠ ${warning}`));
    
    if (sqliteDeleted || qdrantDeleted) {
      console.log('\nPartial reset successful. Directory-indexer is ready for fresh indexing.');
    } else {
      console.log('\nReset had issues but you can try again or check your configuration.');
    }
  } else {
    console.log('\nReset complete. Directory-indexer is ready for fresh indexing.');
  }
}