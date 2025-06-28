use log::{error, info, warn};
use std::path::PathBuf;

use crate::{
    config::Config,
    embedding::EmbeddingProvider,
    error::{IndexerError, Result},
    indexing::files::{FileInfo, FileScanner},
    storage::{qdrant::VectorPoint, FileRecord, QdrantStore, SqliteStore},
    utils::{calculate_file_hash, chunk_text},
};

pub struct IndexingEngine {
    config: Config,
    sqlite_store: SqliteStore,
    vector_store: QdrantStore,
    embedding_provider: Box<dyn EmbeddingProvider>,
}

#[derive(Debug)]
pub struct IndexingStats {
    pub directories_processed: usize,
    pub files_processed: usize,
    pub files_skipped: usize,
    pub files_errored: usize,
    pub chunks_created: usize,
}

impl IndexingEngine {
    pub async fn new(
        config: Config,
        sqlite_store: SqliteStore,
        vector_store: QdrantStore,
        embedding_provider: Box<dyn EmbeddingProvider>,
    ) -> Result<Self> {
        Ok(Self {
            config,
            sqlite_store,
            vector_store,
            embedding_provider,
        })
    }

    pub async fn index_directories(&self, paths: Vec<PathBuf>) -> Result<IndexingStats> {
        info!("Starting indexing for {} directories", paths.len());

        let mut stats = IndexingStats {
            directories_processed: 0,
            files_processed: 0,
            files_skipped: 0,
            files_errored: 0,
            chunks_created: 0,
        };

        for path in paths {
            match self.index_directory(&path).await {
                Ok(dir_stats) => {
                    stats.directories_processed += 1;
                    stats.files_processed += dir_stats.files_processed;
                    stats.files_skipped += dir_stats.files_skipped;
                    stats.files_errored += dir_stats.files_errored;
                    stats.chunks_created += dir_stats.chunks_created;
                }
                Err(e) => {
                    error!("Failed to index directory {:?}: {}", path, e);
                    stats.files_errored += 1;
                }
            }
        }

        info!("Indexing completed: {:?}", stats);
        Ok(stats)
    }

    async fn index_directory(&self, path: &PathBuf) -> Result<IndexingStats> {
        info!("Indexing directory: {:?}", path);

        let mut stats = IndexingStats {
            directories_processed: 0,
            files_processed: 0,
            files_skipped: 0,
            files_errored: 0,
            chunks_created: 0,
        };

        // Add directory to SQLite for tracking
        self.sqlite_store.add_directory(&path.to_string_lossy())?;

        // Scan directory for files
        let scanner = FileScanner::new();
        let files = scanner.scan_directory(path).await?;

        info!("Found {} files to process in {:?}", files.len(), path);

        // Process each file
        for file_info in files {
            match self.process_file(&file_info).await {
                Ok(chunks_count) => {
                    stats.files_processed += 1;
                    stats.chunks_created += chunks_count;
                }
                Err(e) => {
                    error!("Failed to process file {:?}: {}", file_info.path, e);
                    stats.files_errored += 1;
                }
            }
        }

        // Update directory status
        self.sqlite_store
            .update_directory_status(&path.to_string_lossy(), "completed")?;

        stats.directories_processed = 1;
        Ok(stats)
    }

    async fn process_file(&self, file_info: &FileInfo) -> Result<usize> {
        info!("Processing file: {:?}", file_info.path);

        // Calculate file hash
        let file_hash = calculate_file_hash(&file_info.path)?;

        // Check if file already exists and is up to date
        if let Some(existing) = self.sqlite_store.get_file_by_path(&file_info.path)? {
            if existing.hash == file_hash
                && existing.modified_time == (file_info.modified_time as i64)
            {
                info!("File unchanged, skipping: {:?}", file_info.path);
                return Ok(0);
            }

            // File changed, remove old data
            self.vector_store
                .delete_points_by_file(&file_info.path)
                .await?;
        }

        // Extract and chunk content
        let content = tokio::fs::read_to_string(&file_info.path)
            .await
            .map_err(|e| {
                IndexerError::file_processing(format!(
                    "Failed to read file {:?}: {}",
                    file_info.path, e
                ))
            })?;

        let chunks = chunk_text(
            &content,
            self.config.indexing.chunk_size,
            self.config.indexing.overlap,
        );

        if chunks.is_empty() {
            info!("No chunks generated for file: {:?}", file_info.path);
            return Ok(0);
        }

        // Generate embeddings for each chunk in batches to avoid overwhelming the service
        let mut vector_points = Vec::new();
        let batch_size = 10; // Process 10 chunks concurrently at a time

        for (batch_num, chunk_batch) in chunks.chunks(batch_size).enumerate() {
            // Create futures for current batch
            let chunk_futures: Vec<_> = chunk_batch
                .iter()
                .enumerate()
                .map(|(batch_idx, chunk_content)| {
                    let embedding_provider = &self.embedding_provider;
                    let chunk_content = chunk_content.clone();
                    let global_chunk_id = batch_num * batch_size + batch_idx; // Calculate global chunk ID
                    async move {
                        match embedding_provider.generate_embedding(chunk_content).await {
                            Ok(embedding) => Some((global_chunk_id, embedding)),
                            Err(e) => {
                                error!(
                                    "Failed to generate embedding for chunk {}: {}",
                                    global_chunk_id, e
                                );
                                None
                            }
                        }
                    }
                })
                .collect();

            // Execute current batch concurrently
            let results = futures::future::join_all(chunk_futures).await;

            // Process batch results
            for (chunk_id, embedding) in results.into_iter().flatten() {
                let point_id = uuid::Uuid::new_v4().to_string();
                let point = VectorPoint {
                    id: point_id,
                    vector: embedding,
                    file_path: file_info.path.clone(),
                    chunk_id,
                    parent_directories: file_info.parent_dirs.clone(),
                };
                vector_points.push(point);
            }
        }

        // Store vectors in Qdrant
        if !vector_points.is_empty() {
            self.vector_store.upsert_points(vector_points).await?;
        }

        // Store file record in SQLite
        let file_record = FileRecord {
            id: 0, // Will be set by database
            path: file_info.path.clone(),
            size: file_info.size as i64,
            modified_time: file_info.modified_time as i64,
            hash: file_hash,
            parent_dirs: file_info.parent_dirs.clone(),
            chunks_json: Some(serde_json::json!(chunks)),
            errors_json: None,
        };

        self.sqlite_store.add_file(&file_record)?;

        info!(
            "Successfully processed file: {:?} ({} chunks)",
            file_info.path,
            chunks.len()
        );
        Ok(chunks.len())
    }

    pub async fn update_file(&self, file_path: &PathBuf) -> Result<()> {
        info!("Updating file: {:?}", file_path);

        // TODO: Implement file update logic
        // This would include:
        // 1. Remove old chunks from vector store
        // 2. Re-process the file
        // 3. Update SQLite and Qdrant

        warn!("File update not yet implemented");
        Ok(())
    }

    pub async fn remove_file(&self, file_path: &PathBuf) -> Result<()> {
        info!("Removing file: {:?}", file_path);

        // TODO: Implement file removal logic
        // This would include:
        // 1. Remove from vector store
        // 2. Remove from SQLite

        warn!("File removal not yet implemented");
        Ok(())
    }
}
