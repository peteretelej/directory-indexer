use log::{error, info, warn};
use std::path::PathBuf;

use crate::{
    config::Config,
    embedding::EmbeddingProvider,
    error::{IndexerError, Result},
    indexing::files::{FileScanner, FileInfo},
    storage::{qdrant::VectorPoint, QdrantStore, SqliteStore, FileRecord},
    utils::{chunk_text, calculate_file_hash},
};

#[allow(dead_code)]
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
        let files = scanner.scan_directory(path)?;

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
        self.sqlite_store.update_directory_status(
            &path.to_string_lossy(), 
            "completed"
        )?;

        stats.directories_processed = 1;
        Ok(stats)
    }

    async fn process_file(&self, file_info: &FileInfo) -> Result<usize> {
        info!("Processing file: {:?}", file_info.path);

        // Calculate file hash
        let file_hash = calculate_file_hash(&file_info.path)?;

        // Check if file already exists and is up to date
        if let Some(existing) = self.sqlite_store.get_file_by_path(&file_info.path)? {
            if existing.hash == file_hash && existing.modified_time == (file_info.modified_time as i64) {
                info!("File unchanged, skipping: {:?}", file_info.path);
                return Ok(0);
            }
            
            // File changed, remove old data
            self.vector_store.delete_points_by_file(&file_info.path).await?;
        }

        // Extract and chunk content
        let content = std::fs::read_to_string(&file_info.path)
            .map_err(|e| IndexerError::file_processing(format!("Failed to read file {:?}: {}", file_info.path, e)))?;

        let chunks = chunk_text(&content, self.config.indexing.chunk_size, self.config.indexing.overlap);
        
        if chunks.is_empty() {
            info!("No chunks generated for file: {:?}", file_info.path);
            return Ok(0);
        }

        // Generate embeddings for each chunk
        let mut vector_points = Vec::new();
        
        for (chunk_id, chunk_content) in chunks.iter().enumerate() {
            match self.embedding_provider.generate_embedding(chunk_content.clone()).await {
                Ok(embedding) => {
                    // Generate a UUID for the point ID
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
                Err(e) => {
                    error!("Failed to generate embedding for chunk {} in file {:?}: {}", chunk_id, file_info.path, e);
                }
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

        info!("Successfully processed file: {:?} ({} chunks)", file_info.path, chunks.len());
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
