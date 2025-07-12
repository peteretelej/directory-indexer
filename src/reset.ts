import { existsSync, statSync } from 'fs';
import { unlink } from 'fs/promises';
import { readlineSync } from './utils.js';
import { QdrantClient } from './storage.js';
import type { Config } from './config.js';

export interface ResetOptions {
  force?: boolean;
  verbose?: boolean;
}

export interface ResetStats {
  sqliteExists: boolean;
  sqliteSize?: string;
  qdrantCollectionExists: boolean;
  qdrantVectorCount?: number;
}

export async function resetEnvironment(config: Config, options: ResetOptions = {}): Promise<void> {
  const stats = await gatherResetStats(config);
  
  if (!options.force) {
    await showConfirmation(stats, config);
  }
  
  await performReset(config, options, stats);
}

async function gatherResetStats(config: Config): Promise<ResetStats> {
  const stats: ResetStats = {
    sqliteExists: false,
    qdrantCollectionExists: false
  };

  if (existsSync(config.storage.sqlitePath)) {
    stats.sqliteExists = true;
    try {
      const fileStats = statSync(config.storage.sqlitePath);
      const sizeInMB = (fileStats.size / (1024 * 1024)).toFixed(1);
      stats.sqliteSize = `${sizeInMB} MB`;
    } catch {
      stats.sqliteSize = 'unknown size';
    }
  }

  try {
    const qdrant = new QdrantClient(config);
    const isHealthy = await qdrant.healthCheck();
    
    if (isHealthy) {
      const collectionInfo = await qdrant.getCollectionInfo();
      if (collectionInfo) {
        stats.qdrantCollectionExists = true;
        stats.qdrantVectorCount = collectionInfo.vectors_count || 0;
      }
    }
  } catch {
    // Qdrant unavailable - will be handled in reset
  }

  return stats;
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

async function performReset(config: Config, options: ResetOptions, stats: ResetStats): Promise<void> {
  let sqliteDeleted = false;
  let qdrantDeleted = false;
  const warnings: string[] = [];

  if (options.verbose) {
    console.log('\nResetting directory-indexer data...');
  }

  // Reset SQLite database
  if (stats.sqliteExists) {
    try {
      if (options.verbose) {
        console.log(`  ✓ Deleting SQLite database: ${config.storage.sqlitePath}`);
      }
      await unlink(config.storage.sqlitePath);
      sqliteDeleted = true;
    } catch (error) {
      const message = `Failed to delete SQLite database: ${error instanceof Error ? error.message : 'Unknown error'}`;
      warnings.push(message);
      if (options.verbose) {
        console.log(`  ⚠ ${message}`);
      }
    }
  } else if (options.verbose) {
    console.log(`  ✓ SQLite database not found (already clean)`);
    sqliteDeleted = true;
  }

  // Reset Qdrant collection
  try {
    const qdrant = new QdrantClient(config);
    const isHealthy = await qdrant.healthCheck();
    
    if (!isHealthy) {
      const message = `Qdrant unavailable at ${config.storage.qdrantEndpoint}`;
      warnings.push(message);
      if (options.verbose) {
        console.log(`  ⚠ ${message}`);
      }
    } else {
      if (stats.qdrantCollectionExists) {
        if (options.verbose) {
          console.log(`  ✓ Deleting Qdrant collection: ${config.storage.qdrantCollection}`);
        }
        await qdrant.deleteCollection();
        qdrantDeleted = true;
      } else {
        if (options.verbose) {
          console.log(`  ✓ Qdrant collection not found (already clean)`);
        }
        qdrantDeleted = true;
      }
    }
  } catch (error) {
    const message = `Failed to reset Qdrant collection: ${error instanceof Error ? error.message : 'Unknown error'}`;
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