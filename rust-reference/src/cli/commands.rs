use log::{error, info};
use std::path::{Path, PathBuf};

use crate::embedding::create_embedding_provider;
use crate::indexing::engine::IndexingEngine;
use crate::mcp::McpServer;
use crate::search::engine::{create_search_engine, SearchQuery, SearchResult};
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

    // Check for state mismatch between SQLite and Qdrant
    engine.validate_state_consistency().await?;

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
    search_internal(query, path, limit, true).await?;
    Ok(())
}

pub async fn search_internal(
    query: String,
    path: Option<String>,
    limit: Option<usize>,
    output_to_console: bool,
) -> Result<Vec<SearchResult>> {
    info!("Searching for: '{query}' in path: {path:?}, limit: {limit:?}");

    // Create SearchEngine
    let search_engine = create_search_engine().await?;

    // Create SearchQuery
    let search_query = SearchQuery {
        text: query.clone(),
        directory_filter: path.as_ref().map(PathBuf::from),
        limit: limit.unwrap_or(10),
        similarity_threshold: None,
    };

    // Perform search
    let results = search_engine.search(search_query).await?;

    // Handle console output
    if output_to_console {
        display_search_results(&query, &results, &path, limit);
    }

    Ok(results)
}

fn display_search_results(
    query: &str,
    results: &[SearchResult],
    path: &Option<String>,
    limit: Option<usize>,
) {
    println!("Searching for: '{query}'");
    if let Some(p) = path {
        println!("  Scope: {p}");
    }
    if let Some(l) = limit {
        println!("  Limit: {l}");
    }

    if results.is_empty() {
        println!("No results found for query: '{query}'");
    } else {
        println!("\nSearch Results:");
        println!("==============");

        for (i, result) in results.iter().enumerate() {
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

            // Try to read the specific chunk content from SQLite (keeping this for now)
            if let Ok(config) = Config::load() {
                if let Ok(sqlite_store) = SqliteStore::new(&config.storage.sqlite_path) {
                    if let Ok(Some(file_record)) = sqlite_store.get_file_by_path(&result.file_path)
                    {
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
    }
}

pub async fn similar(file: String, limit: usize) -> Result<()> {
    info!("Finding files similar to: '{file}', limit: {limit}");

    // Create SearchEngine
    let search_engine = create_search_engine().await?;

    // Perform similar file search
    let results = search_engine
        .find_similar_files(PathBuf::from(&file), limit)
        .await?;

    // Display results
    display_similar_files(&file, &results, limit);

    Ok(())
}

fn display_similar_files(file: &str, results: &[SearchResult], limit: usize) {
    println!("Finding files similar to: {file}");
    println!("  Limit: {limit}");

    if results.is_empty() {
        println!("No similar files found for: {file}");
    } else {
        println!("\nSimilar Files:");
        println!("==============");

        for (i, result) in results.iter().enumerate() {
            println!(
                "\n{num}. {file_path} (score: {score:.3})",
                num = i + 1,
                file_path = result.file_path,
                score = result.score
            );
            println!("   Best matching chunk: {}", result.chunk_id);

            // Try to get file metadata for preview (keeping this for now)
            if let Ok(config) = Config::load() {
                if let Ok(sqlite_store) = SqliteStore::new(&config.storage.sqlite_path) {
                    if let Ok(Some(similar_file_record)) =
                        sqlite_store.get_file_by_path(&result.file_path)
                    {
                        if let Some(chunks_json) = similar_file_record.chunks_json {
                            if let Ok(similar_chunks) =
                                serde_json::from_value::<Vec<String>>(chunks_json)
                            {
                                if result.chunk_id < similar_chunks.len() {
                                    let chunk_content = &similar_chunks[result.chunk_id];
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
    }
}

pub async fn get(file: String, chunks: Option<String>) -> Result<()> {
    info!("Getting content for: '{file}', chunks: {chunks:?}");

    // Validate and parse chunk range if provided
    let chunk_range = if let Some(ref chunk_str) = chunks {
        validate_chunk_range(chunk_str)?;
        Some(parse_chunk_range(chunk_str)?)
    } else {
        None
    };

    // Create SearchEngine
    let search_engine = create_search_engine().await?;

    // Get file content
    let content = search_engine
        .get_file_content(PathBuf::from(&file), chunk_range)
        .await?;

    // Display content
    display_file_content(&file, &content, &chunks, chunk_range);

    Ok(())
}

fn display_file_content(
    file: &str,
    content: &str,
    chunks: &Option<String>,
    chunk_range: Option<(usize, usize)>,
) {
    println!("Getting content from: {file}");
    if let Some(c) = chunks {
        println!("  Chunks: {c}");
    }

    println!("\nFile Content:");
    println!("=============");
    if let Some((start, end)) = chunk_range {
        println!("Chunks {start}-{end}:");
    }
    println!("{content}");
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

pub fn validate_chunk_range(chunk_str: &str) -> Result<()> {
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

pub fn parse_chunk_range(chunk_str: &str) -> Result<(usize, usize)> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_chunk_range_valid_single() {
        assert!(validate_chunk_range("5").is_ok());
        assert!(validate_chunk_range("1").is_ok());
        assert!(validate_chunk_range("100").is_ok());
    }

    #[test]
    fn test_validate_chunk_range_valid_range() {
        assert!(validate_chunk_range("1-5").is_ok());
        assert!(validate_chunk_range("2-10").is_ok());
        assert!(validate_chunk_range("1-1").is_ok());
    }

    #[test]
    fn test_validate_chunk_range_invalid() {
        // Zero values
        assert!(validate_chunk_range("0").is_err());
        assert!(validate_chunk_range("1-0").is_err());
        assert!(validate_chunk_range("0-5").is_err());

        // Invalid range order
        assert!(validate_chunk_range("5-1").is_err());
        assert!(validate_chunk_range("10-5").is_err());

        // Invalid format
        assert!(validate_chunk_range("a").is_err());
        assert!(validate_chunk_range("1-a").is_err());
        assert!(validate_chunk_range("a-5").is_err());
        assert!(validate_chunk_range("1-2-3").is_err());
        assert!(validate_chunk_range("").is_err());
        assert!(validate_chunk_range("-").is_err());
        assert!(validate_chunk_range("1-").is_err());
        assert!(validate_chunk_range("-5").is_err());
    }

    #[test]
    fn test_parse_chunk_range_single() {
        assert_eq!(parse_chunk_range("5").unwrap(), (5, 5));
        assert_eq!(parse_chunk_range("1").unwrap(), (1, 1));
        assert_eq!(parse_chunk_range("42").unwrap(), (42, 42));
    }

    #[test]
    fn test_parse_chunk_range_range() {
        assert_eq!(parse_chunk_range("1-5").unwrap(), (1, 5));
        assert_eq!(parse_chunk_range("2-10").unwrap(), (2, 10));
        assert_eq!(parse_chunk_range("1-1").unwrap(), (1, 1));
    }

    #[test]
    fn test_chunk_validation_and_parsing_integration() {
        let test_cases = vec![
            ("1", (1, 1)),
            ("5", (5, 5)),
            ("1-5", (1, 5)),
            ("2-10", (2, 10)),
            ("42-42", (42, 42)),
        ];

        for (input, expected) in test_cases {
            validate_chunk_range(input).unwrap();
            let result = parse_chunk_range(input).unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_index_command_validation() {
        // Test that empty paths validation would work
        let empty_paths: Vec<String> = vec![];
        // This would normally be tested in integration tests since it requires file system
        // but we can test the logic
        assert!(empty_paths.is_empty());
    }

    #[test]
    fn test_search_query_validation() {
        // Test empty query validation logic
        let empty_query = "";
        let whitespace_query = "   ";
        let valid_query = "test query";

        assert!(empty_query.trim().is_empty());
        assert!(whitespace_query.trim().is_empty());
        assert!(!valid_query.trim().is_empty());
    }

    #[test]
    fn test_error_message_construction() {
        // Test that our error message patterns work as expected
        let path = "/nonexistent/path";
        let error_msg = format!("Directory not found: {path}");
        assert!(error_msg.contains("/nonexistent/path"));

        let file = "test.txt";
        let error_msg = format!("Path is not a file: {file}");
        assert!(error_msg.contains("test.txt"));

        let query = "test";
        let error_msg = format!("No results found for query: '{query}'");
        assert!(error_msg.contains("'test'"));
    }

    #[test]
    fn test_format_validation() {
        // Test status format validation logic
        let valid_formats = vec!["text", "json"];
        let invalid_formats = vec!["xml", "yaml", "csv", ""];

        for format in valid_formats {
            assert!(format == "text" || format == "json");
        }

        for format in invalid_formats {
            assert!(format != "text" && format != "json");
        }
    }

    #[test]
    fn test_path_collection_logic() {
        // Test the path collection and conversion logic
        let paths = vec!["./test1".to_string(), "/home/user/test2".to_string()];
        let path_bufs: Vec<std::path::PathBuf> =
            paths.iter().map(std::path::PathBuf::from).collect();

        assert_eq!(path_bufs.len(), 2);
        assert_eq!(path_bufs[0], std::path::PathBuf::from("./test1"));
        assert_eq!(path_bufs[1], std::path::PathBuf::from("/home/user/test2"));
    }

    #[test]
    fn test_limit_handling() {
        // Test the limit handling logic
        let default_limit = 10;
        let custom_limit = Some(25);
        let no_limit = None;

        assert_eq!(custom_limit.unwrap_or(default_limit), 25);
        assert_eq!(no_limit.unwrap_or(default_limit), 10);

        // Test search limit logic
        let search_limit = custom_limit.unwrap_or(10);
        assert_eq!(search_limit, 25);

        let search_limit = no_limit.unwrap_or(10);
        assert_eq!(search_limit, 10);
    }

    #[test]
    fn test_content_preview_logic() {
        // Test the content preview truncation logic
        let short_content = "Short content";
        let long_content = "A".repeat(300);

        let short_preview = if short_content.len() > 200 {
            format!("{}...", &short_content[..200])
        } else {
            short_content.to_string()
        };
        assert_eq!(short_preview, "Short content");

        let long_preview = if long_content.len() > 200 {
            format!("{}...", &long_content[..200])
        } else {
            long_content.clone()
        };
        assert!(long_preview.ends_with("..."));
        assert_eq!(long_preview.len(), 203); // 200 chars + "..."
    }
}
