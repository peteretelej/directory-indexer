use log::{info, warn};
use std::path::{Path, PathBuf};

use crate::embedding::create_embedding_provider;
use crate::indexing::engine::IndexingEngine;
use crate::mcp::McpServer;
use crate::storage::{QdrantStore, SqliteStore};
use crate::{Config, IndexerError, Result};

pub async fn index(paths: Vec<String>) -> Result<()> {
    index_internal(paths, true).await
}

pub async fn index_internal(paths: Vec<String>, output_to_console: bool) -> Result<()> {
    info!("Indexing directories: {:?}", paths);

    if paths.is_empty() {
        return Err(IndexerError::invalid_input(
            "At least one directory path is required",
        ));
    }

    // Validate all paths exist before starting indexing
    for path in &paths {
        let path_obj = Path::new(path);
        if !path_obj.exists() {
            return Err(IndexerError::not_found(format!(
                "Directory not found: {}",
                path
            )));
        }
        if !path_obj.is_dir() {
            return Err(IndexerError::invalid_input(format!(
                "Path is not a directory: {}",
                path
            )));
        }
    }

    if output_to_console {
        println!("Indexing {} directories", paths.len());
        for path in &paths {
            println!("  {}", path);
        }
    }

    // Load configuration
    let config = Config::load()?;

    // Initialize storage
    let sqlite_store = SqliteStore::new(&config.storage.sqlite_path)?;
    let vector_store = QdrantStore::new(
        &config.storage.qdrant.endpoint,
        config.storage.qdrant.collection.clone(),
    )
    .await?;

    // Initialize embedding provider
    let embedding_provider = create_embedding_provider(&config.embedding)?;

    // Create indexing engine
    let engine =
        IndexingEngine::new(config, sqlite_store, vector_store, embedding_provider).await?;

    // Convert paths to PathBuf
    let path_bufs: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();

    // Start indexing
    let stats = engine.index_directories(path_bufs).await?;

    if output_to_console {
        println!("Indexing completed!");
        println!("  Directories processed: {}", stats.directories_processed);
        println!("  Files processed: {}", stats.files_processed);
        println!("  Files skipped: {}", stats.files_skipped);
        println!("  Files with errors: {}", stats.files_errored);
        println!("  Chunks created: {}", stats.chunks_created);
    }

    Ok(())
}

pub async fn search(query: String, path: Option<String>, limit: Option<usize>) -> Result<()> {
    search_internal(query, path, limit, true).await
}

pub async fn search_internal(
    query: String,
    path: Option<String>,
    limit: Option<usize>,
    output_to_console: bool,
) -> Result<()> {
    info!(
        "Searching for: '{}' in path: {:?}, limit: {:?}",
        query, path, limit
    );

    if query.trim().is_empty() {
        return Err(IndexerError::invalid_input("Search query cannot be empty"));
    }

    // Validate path if provided
    if let Some(ref p) = path {
        let path_obj = Path::new(p);
        if !path_obj.exists() {
            return Err(IndexerError::not_found(format!(
                "Directory not found: {}",
                p
            )));
        }
    }

    if output_to_console {
        println!("Searching for: '{}'", query);
        if let Some(p) = &path {
            println!("  Scope: {}", p);
        }
        if let Some(l) = limit {
            println!("  Limit: {}", l);
        }
    }

    // Load configuration
    let config = Config::load()?;

    // Initialize storage
    let sqlite_store = SqliteStore::new(&config.storage.sqlite_path)?;
    let vector_store = QdrantStore::new(
        &config.storage.qdrant.endpoint,
        config.storage.qdrant.collection.clone(),
    )
    .await?;

    // Initialize embedding provider
    let embedding_provider = create_embedding_provider(&config.embedding)?;

    // Generate embedding for the query
    let query_embedding = embedding_provider.generate_embedding(query.clone()).await?;

    // Perform vector search
    let search_limit = limit.unwrap_or(10);
    let search_results = vector_store.search(query_embedding, search_limit).await?;

    if output_to_console {
        if search_results.is_empty() {
            println!("No results found for query: '{}'", query);
        } else {
            println!("\nSearch Results:");
            println!("==============");

            for (i, result) in search_results.iter().enumerate() {
                println!(
                    "\n{}. {} (score: {:.3})",
                    i + 1,
                    result.file_path,
                    result.score
                );
                println!("   Chunk: {}", result.chunk_id);
                if !result.parent_directories.is_empty() {
                    println!("   Path: {}", result.parent_directories.join(" > "));
                }

                // Try to read the specific chunk content from SQLite
                if let Ok(Some(file_record)) = sqlite_store.get_file_by_path(&result.file_path) {
                    if let Some(chunks_json) = file_record.chunks_json {
                        if let Ok(chunks) = serde_json::from_value::<Vec<String>>(chunks_json) {
                            if result.chunk_id < chunks.len() {
                                let chunk_content = &chunks[result.chunk_id];
                                let preview = if chunk_content.len() > 200 {
                                    format!("{}...", &chunk_content[..200])
                                } else {
                                    chunk_content.clone()
                                };
                                println!("   Preview: {}", preview.replace('\n', " "));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn similar(file: String, limit: usize) -> Result<()> {
    similar_internal(file, limit, true).await
}

pub async fn similar_internal(file: String, limit: usize, output_to_console: bool) -> Result<()> {
    info!("Finding files similar to: '{}', limit: {}", file, limit);

    let file_path = Path::new(&file);
    if !file_path.exists() {
        return Err(IndexerError::not_found(format!("File not found: {}", file)));
    }
    if !file_path.is_file() {
        return Err(IndexerError::invalid_input(format!(
            "Path is not a file: {}",
            file
        )));
    }

    if output_to_console {
        println!("Finding files similar to: {}", file);
        println!("  Limit: {}", limit);
    }

    warn!("Similar file search not yet implemented - this is a placeholder");
    Ok(())
}

pub async fn get(file: String, chunks: Option<String>) -> Result<()> {
    get_internal(file, chunks, true).await
}

pub async fn get_internal(
    file: String,
    chunks: Option<String>,
    output_to_console: bool,
) -> Result<()> {
    info!("Getting content for: '{}', chunks: {:?}", file, chunks);

    let file_path = Path::new(&file);
    if !file_path.exists() {
        return Err(IndexerError::not_found(format!("File not found: {}", file)));
    }
    if !file_path.is_file() {
        return Err(IndexerError::invalid_input(format!(
            "Path is not a file: {}",
            file
        )));
    }

    // Validate chunk range if provided
    if let Some(ref chunk_str) = chunks {
        validate_chunk_range(chunk_str)?;
    }

    if output_to_console {
        println!("Getting content from: {}", file);
        if let Some(c) = chunks {
            println!("  Chunks: {}", c);
        }
    }

    warn!("File content retrieval not yet implemented - this is a placeholder");
    Ok(())
}

pub async fn serve() -> Result<()> {
    info!("Starting MCP server");

    // Load configuration
    let config = Config::load()?;

    // Create and start MCP server
    let server = McpServer::new(config).await?;
    server.start().await?;

    Ok(())
}

pub async fn status(format: String) -> Result<()> {
    info!("Showing indexing status in format: {}", format);

    // Load configuration
    let config = Config::load()?;

    // Initialize storage
    let sqlite_store = SqliteStore::new(&config.storage.sqlite_path)?;
    let vector_store = QdrantStore::new_without_init(
        &config.storage.qdrant.endpoint,
        config.storage.qdrant.collection.clone(),
    );

    // Get statistics from SQLite
    let (dir_count, file_count, chunk_count) = sqlite_store.get_stats()?;

    // Get vector store info (may not exist yet)
    let collection_info = (vector_store.get_collection_info().await).ok();

    // Calculate database size (approximate)
    let db_size_mb = if config.storage.sqlite_path.exists() {
        std::fs::metadata(&config.storage.sqlite_path)?.len() / (1024 * 1024)
    } else {
        0
    };

    match format.as_str() {
        "json" => {
            println!("{{");
            println!("  \"indexed_directories\": {},", dir_count);
            println!("  \"indexed_files\": {},", file_count);
            println!("  \"total_chunks\": {},", chunk_count);
            if let Some(info) = &collection_info {
                println!("  \"vector_points\": {},", info.points_count);
                println!("  \"indexed_vectors\": {},", info.indexed_vectors_count);
            } else {
                println!("  \"vector_points\": 0,");
                println!("  \"indexed_vectors\": 0,");
            }
            println!("  \"database_size_mb\": {}", db_size_mb);
            println!("}}");
        }
        "text" => {
            println!("Directory Indexer Status");
            println!("  Indexed directories: {}", dir_count);
            println!("  Indexed files: {}", file_count);
            println!("  Total chunks: {}", chunk_count);
            if let Some(info) = &collection_info {
                println!("  Vector points: {}", info.points_count);
                println!("  Indexed vectors: {}", info.indexed_vectors_count);
            } else {
                println!("  Vector points: 0 (collection not created)");
                println!("  Indexed vectors: 0");
            }
            println!("  Database size: {} MB", db_size_mb);
        }
        _ => {
            return Err(IndexerError::invalid_input(format!(
                "Unsupported format: {}. Use 'text' or 'json'",
                format
            )));
        }
    }

    Ok(())
}

fn validate_chunk_range(chunk_str: &str) -> Result<()> {
    if chunk_str.contains('-') {
        let parts: Vec<&str> = chunk_str.split('-').collect();
        if parts.len() != 2 {
            return Err(IndexerError::invalid_input(
                "Invalid chunk range format. Use 'start-end' (e.g., '1-5')",
            ));
        }

        let start: usize = parts[0]
            .parse()
            .map_err(|_| IndexerError::invalid_input("Invalid start chunk number"))?;
        let end: usize = parts[1]
            .parse()
            .map_err(|_| IndexerError::invalid_input("Invalid end chunk number"))?;

        if start == 0 || end == 0 {
            return Err(IndexerError::invalid_input(
                "Chunk numbers must be greater than 0",
            ));
        }
        if start > end {
            return Err(IndexerError::invalid_input(
                "Start chunk must be less than or equal to end chunk",
            ));
        }
    } else {
        let chunk: usize = chunk_str
            .parse()
            .map_err(|_| IndexerError::invalid_input("Invalid chunk number"))?;
        if chunk == 0 {
            return Err(IndexerError::invalid_input(
                "Chunk number must be greater than 0",
            ));
        }
    }
    Ok(())
}
