use log::{error, info};
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
    info!("Indexing directories: {paths:?}");

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
                "Directory not found: {path}"
            )));
        }
        if !path_obj.is_dir() {
            return Err(IndexerError::invalid_input(format!(
                "Path is not a directory: {path}"
            )));
        }
    }

    if output_to_console {
        println!("Indexing {len} directories", len = paths.len());
        for path in &paths {
            println!("  {path}");
        }
    }

    // Load configuration
    let config = Config::load()?;

    // Validate environment before proceeding
    if let Err(e) = crate::environment::validate_environment(&config).await {
        error!("{e}");
        return Err(e);
    }

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
    info!("Searching for: '{query}' in path: {path:?}, limit: {limit:?}");

    if query.trim().is_empty() {
        return Err(IndexerError::invalid_input("Search query cannot be empty"));
    }

    // Validate path if provided
    if let Some(ref p) = path {
        let path_obj = Path::new(p);
        if !path_obj.exists() {
            return Err(IndexerError::not_found(format!("Directory not found: {p}")));
        }
    }

    if output_to_console {
        println!("Searching for: '{query}'");
        if let Some(p) = &path {
            println!("  Scope: {p}");
        }
        if let Some(l) = limit {
            println!("  Limit: {l}");
        }
    }

    // Load configuration
    let config = Config::load()?;

    // Validate environment before proceeding
    if let Err(e) = crate::environment::validate_environment(&config).await {
        error!("{e}");
        return Err(e);
    }

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
            println!("No results found for query: '{query}'");
        } else {
            println!("\nSearch Results:");
            println!("==============");

            for (i, result) in search_results.iter().enumerate() {
                println!(
                    "\n{num}. {file} (score: {score:.3})",
                    num = i + 1,
                    file = result.file_path,
                    score = result.score
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
    info!("Finding files similar to: '{file}', limit: {limit}");

    let file_path = Path::new(&file);
    if !file_path.exists() {
        return Err(IndexerError::not_found(format!("File not found: {file}")));
    }
    if !file_path.is_file() {
        return Err(IndexerError::invalid_input(format!(
            "Path is not a file: {file}"
        )));
    }

    if output_to_console {
        println!("Finding files similar to: {file}");
        println!("  Limit: {limit}");
    }

    // Load configuration
    let config = Config::load()?;

    // Validate environment before proceeding
    if let Err(e) = crate::environment::validate_environment(&config).await {
        error!("{e}");
        return Err(e);
    }

    // Initialize storage
    let sqlite_store = SqliteStore::new(&config.storage.sqlite_path)?;
    let vector_store = QdrantStore::new(
        &config.storage.qdrant.endpoint,
        config.storage.qdrant.collection.clone(),
    )
    .await?;

    // Initialize embedding provider
    let embedding_provider = create_embedding_provider(&config.embedding)?;

    // Try to get file from database to retrieve chunks
    let normalized_path = crate::utils::normalize_path(file_path)?;
    let file_record = sqlite_store.get_file_by_path(&normalized_path)?;

    // Check if file is indexed
    let file_embedding = if let Some(file_record) = file_record {
        // Parse chunks JSON to get file chunks
        let chunks = match file_record.chunks_json {
            Some(chunks_json) => {
                serde_json::from_value::<Vec<String>>(chunks_json).map_err(|e| {
                    IndexerError::file_processing(format!("Failed to parse chunks: {e}"))
                })?
            }
            None => {
                return Err(IndexerError::not_found(format!(
                    "No chunks found for file: {file}"
                )));
            }
        };

        if chunks.is_empty() {
            return Err(IndexerError::not_found(format!(
                "No chunks found for file: {file}"
            )));
        }

        // Use the first chunk as representative of the file
        let representative_chunk = &chunks[0];

        // Generate embedding for the representative chunk
        embedding_provider
            .generate_embedding(representative_chunk.clone())
            .await?
    } else {
        // File not indexed, read from filesystem and generate embedding
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| IndexerError::file_processing(format!("Failed to read file: {e}")))?;

        // Use first 512 chars as representative content
        let representative_content = if content.len() > 512 {
            &content[..512]
        } else {
            &content
        };

        embedding_provider
            .generate_embedding(representative_content.to_string())
            .await?
    };

    // Search for similar chunks
    let search_results = vector_store.search(file_embedding, limit + 5).await?;

    // Filter out results from the same file and group by file path
    let mut file_scores: std::collections::HashMap<String, (f32, usize)> =
        std::collections::HashMap::new();

    for result in search_results {
        // Skip if it's the same file
        if result.file_path == file {
            continue;
        }

        // Keep track of the best score for each file
        let entry = file_scores
            .entry(result.file_path.clone())
            .or_insert((0.0, 0));
        if result.score > entry.0 {
            entry.0 = result.score;
            entry.1 = result.chunk_id;
        }
    }

    // Sort by score and take top results
    let mut similar_files: Vec<_> = file_scores.into_iter().collect();
    similar_files.sort_by(|a, b| {
        b.1 .0
            .partial_cmp(&a.1 .0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    similar_files.truncate(limit);

    if output_to_console {
        if similar_files.is_empty() {
            println!("No similar files found for: {file}");
        } else {
            println!("\nSimilar Files:");
            println!("==============");

            for (i, (file_path, (score, chunk_id))) in similar_files.iter().enumerate() {
                println!("\n{num}. {file_path} (score: {score:.3})", num = i + 1);
                println!("   Best matching chunk: {chunk_id}");

                // Try to get file metadata
                if let Ok(Some(similar_file_record)) = sqlite_store.get_file_by_path(file_path) {
                    if let Some(chunks_json) = similar_file_record.chunks_json {
                        if let Ok(similar_chunks) =
                            serde_json::from_value::<Vec<String>>(chunks_json)
                        {
                            if *chunk_id < similar_chunks.len() {
                                let chunk_content = &similar_chunks[*chunk_id];
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

pub async fn get(file: String, chunks: Option<String>) -> Result<()> {
    get_internal(file, chunks, true).await
}

pub async fn get_internal(
    file: String,
    chunks: Option<String>,
    output_to_console: bool,
) -> Result<()> {
    info!("Getting content for: '{file}', chunks: {chunks:?}");

    let file_path = Path::new(&file);
    if !file_path.exists() {
        return Err(IndexerError::not_found(format!("File not found: {file}")));
    }
    if !file_path.is_file() {
        return Err(IndexerError::invalid_input(format!(
            "Path is not a file: {file}"
        )));
    }

    // Validate and parse chunk range if provided
    let chunk_range = if let Some(ref chunk_str) = chunks {
        validate_chunk_range(chunk_str)?;
        Some(parse_chunk_range(chunk_str)?)
    } else {
        None
    };

    if output_to_console {
        println!("Getting content from: {file}");
        if let Some(c) = &chunks {
            println!("  Chunks: {c}");
        }
    }

    // Load configuration
    let config = Config::load()?;

    // Initialize storage
    let sqlite_store = SqliteStore::new(&config.storage.sqlite_path)?;

    // Try to get file from database
    let normalized_path = crate::utils::normalize_path(file_path)?;
    let file_record = sqlite_store.get_file_by_path(&normalized_path)?;

    // If chunks are stored in database, use those; otherwise read from file system
    let content = if let Some(file_record) = file_record {
        if let Some(chunks_json) = file_record.chunks_json {
            let chunks = serde_json::from_value::<Vec<String>>(chunks_json).map_err(|e| {
                IndexerError::file_processing(format!("Failed to parse chunks: {e}"))
            })?;

            if let Some((start, end)) = chunk_range {
                // Return specific chunk range (1-indexed to 0-indexed)
                let start_idx = start.saturating_sub(1);
                let end_idx = end.min(chunks.len());

                if start_idx >= chunks.len() {
                    return Err(IndexerError::invalid_input(format!(
                        "Chunk range {start}-{end} exceeds available chunks ({})",
                        chunks.len()
                    )));
                }

                chunks[start_idx..end_idx].join("\n")
            } else {
                // Return all chunks
                chunks.join("\n")
            }
        } else {
            // File indexed but no chunks stored, read from filesystem
            let content = std::fs::read_to_string(file_path)
                .map_err(|e| IndexerError::file_processing(format!("Failed to read file: {e}")))?;

            if let Some((start, end)) = chunk_range {
                // Split content into chunks on-the-fly for files without stored chunks
                let lines: Vec<&str> = content.lines().collect();
                let lines_per_chunk = lines.len().div_ceil(10); // Approximate 10 chunks
                let total_chunks = lines.len().div_ceil(lines_per_chunk);

                if start > total_chunks || start == 0 {
                    return Err(IndexerError::invalid_input(format!(
                        "Chunk {start} is out of range. File has {total_chunks} estimated chunks"
                    )));
                }

                let start_line = (start - 1) * lines_per_chunk;
                let end_line = (end * lines_per_chunk).min(lines.len());

                lines[start_line..end_line].join("\n")
            } else {
                content
            }
        }
    } else {
        // File not indexed, read directly from file system
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| IndexerError::file_processing(format!("Failed to read file: {e}")))?;

        if let Some((start, end)) = chunk_range {
            // Split content into chunks on-the-fly for unindexed files
            let lines: Vec<&str> = content.lines().collect();
            let lines_per_chunk = lines.len().div_ceil(10); // Approximate 10 chunks
            let total_chunks = lines.len().div_ceil(lines_per_chunk);

            if start > total_chunks || start == 0 {
                return Err(IndexerError::invalid_input(format!(
                    "Chunk {start} is out of range. File has {total_chunks} estimated chunks"
                )));
            }

            let start_line = (start - 1) * lines_per_chunk;
            let end_line = (end * lines_per_chunk).min(lines.len());

            lines[start_line..end_line].join("\n")
        } else {
            content
        }
    };

    if output_to_console {
        println!("\nFile Content:");
        println!("=============");
        if let Some((start, end)) = chunk_range {
            println!("Chunks {start}-{end}:");
        }
        println!("{content}");
    }

    Ok(())
}

pub async fn serve() -> Result<()> {
    info!("Starting MCP server");

    // Load configuration
    let config = Config::load()?;

    // Validate environment before proceeding
    if let Err(e) = crate::environment::validate_environment(&config).await {
        error!("{e}");
        return Err(e);
    }

    // Create and start MCP server
    let server = McpServer::new(config).await?;
    server.start().await?;

    Ok(())
}

pub async fn status(format: String) -> Result<()> {
    info!("Showing indexing status in format: {format}");

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
            println!("  \"indexed_directories\": {dir_count},");
            println!("  \"indexed_files\": {file_count},");
            println!("  \"total_chunks\": {chunk_count},");
            if let Some(info) = &collection_info {
                println!("  \"vector_points\": {},", info.points_count);
                println!("  \"indexed_vectors\": {},", info.indexed_vectors_count);
            } else {
                println!("  \"vector_points\": 0,");
                println!("  \"indexed_vectors\": 0,");
            }
            println!("  \"database_size_mb\": {db_size_mb}");
            println!("}}");
        }
        "text" => {
            println!("Directory Indexer Status");
            println!("  Indexed directories: {dir_count}");
            println!("  Indexed files: {file_count}");
            println!("  Total chunks: {chunk_count}");
            if let Some(info) = &collection_info {
                println!("  Vector points: {}", info.points_count);
                println!("  Indexed vectors: {}", info.indexed_vectors_count);
            } else {
                println!("  Vector points: 0 (collection not created)");
                println!("  Indexed vectors: 0");
            }
            println!("  Database size: {db_size_mb} MB");
        }
        _ => {
            return Err(IndexerError::invalid_input(format!(
                "Unsupported format: {format}. Use 'text' or 'json'"
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

fn parse_chunk_range(chunk_str: &str) -> Result<(usize, usize)> {
    if chunk_str.contains('-') {
        let parts: Vec<&str> = chunk_str.split('-').collect();
        let start = parts[0].parse().unwrap(); // validated above
        let end = parts[1].parse().unwrap(); // validated above
        Ok((start, end))
    } else {
        let chunk = chunk_str.parse().unwrap(); // validated above
        Ok((chunk, chunk))
    }
}
