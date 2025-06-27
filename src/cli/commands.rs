use log::{info, warn};
use std::path::Path;

use crate::{Config, IndexerError, Result};
use crate::mcp::McpServer;

pub async fn index(paths: Vec<String>) -> Result<()> {
    index_internal(paths, true).await
}

pub async fn index_internal(paths: Vec<String>, output_to_console: bool) -> Result<()> {
    info!("Indexing directories: {:?}", paths);

    if paths.is_empty() {
        return Err(IndexerError::invalid_input("At least one directory path is required"));
    }

    // Validate all paths exist before starting indexing
    for path in &paths {
        let path_obj = Path::new(path);
        if !path_obj.exists() {
            return Err(IndexerError::not_found(format!("Directory not found: {}", path)));
        }
        if !path_obj.is_dir() {
            return Err(IndexerError::invalid_input(format!("Path is not a directory: {}", path)));
        }
    }

    if output_to_console {
        println!("Indexing {} directories", paths.len());
        for path in &paths {
            println!("  {}", path);
        }
    }

    warn!("Indexing not yet implemented - this is a placeholder");
    Ok(())
}

pub async fn search(query: String, path: Option<String>, limit: Option<usize>) -> Result<()> {
    search_internal(query, path, limit, true).await
}

pub async fn search_internal(query: String, path: Option<String>, limit: Option<usize>, output_to_console: bool) -> Result<()> {
    info!("Searching for: '{}' in path: {:?}, limit: {:?}", query, path, limit);

    if query.trim().is_empty() {
        return Err(IndexerError::invalid_input("Search query cannot be empty"));
    }

    // Validate path if provided
    if let Some(ref p) = path {
        let path_obj = Path::new(p);
        if !path_obj.exists() {
            return Err(IndexerError::not_found(format!("Directory not found: {}", p)));
        }
    }

    if output_to_console {
        println!("Searching for: '{}'", query);
        if let Some(p) = path {
            println!("  Scope: {}", p);
        }
        if let Some(l) = limit {
            println!("  Limit: {}", l);
        }
    }

    warn!("Search not yet implemented - this is a placeholder");
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
        return Err(IndexerError::invalid_input(format!("Path is not a file: {}", file)));
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

pub async fn get_internal(file: String, chunks: Option<String>, output_to_console: bool) -> Result<()> {
    info!("Getting content for: '{}', chunks: {:?}", file, chunks);

    let file_path = Path::new(&file);
    if !file_path.exists() {
        return Err(IndexerError::not_found(format!("File not found: {}", file)));
    }
    if !file_path.is_file() {
        return Err(IndexerError::invalid_input(format!("Path is not a file: {}", file)));
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

    match format.as_str() {
        "json" => {
            println!("{{");
            println!("  \"indexed_directories\": 0,");
            println!("  \"indexed_files\": 0,");
            println!("  \"total_chunks\": 0,");
            println!("  \"database_size_mb\": 0");
            println!("}}");
        }
        "text" => {
            println!("Directory Indexer Status");
            println!("  Indexed directories: 0");
            println!("  Indexed files: 0");
            println!("  Total chunks: 0");
            println!("  Database size: 0 MB");
        }
        _ => {
            return Err(IndexerError::invalid_input(format!("Unsupported format: {}. Use 'text' or 'json'", format)));
        }
    }

    warn!("Status reporting not yet implemented - this is a placeholder");
    Ok(())
}

fn validate_chunk_range(chunk_str: &str) -> Result<()> {
    if chunk_str.contains('-') {
        let parts: Vec<&str> = chunk_str.split('-').collect();
        if parts.len() != 2 {
            return Err(IndexerError::invalid_input("Invalid chunk range format. Use 'start-end' (e.g., '1-5')"));
        }
        
        let start: usize = parts[0].parse()
            .map_err(|_| IndexerError::invalid_input("Invalid start chunk number"))?;
        let end: usize = parts[1].parse()
            .map_err(|_| IndexerError::invalid_input("Invalid end chunk number"))?;
            
        if start == 0 || end == 0 {
            return Err(IndexerError::invalid_input("Chunk numbers must be greater than 0"));
        }
        if start > end {
            return Err(IndexerError::invalid_input("Start chunk must be less than or equal to end chunk"));
        }
    } else {
        let chunk: usize = chunk_str.parse()
            .map_err(|_| IndexerError::invalid_input("Invalid chunk number"))?;
        if chunk == 0 {
            return Err(IndexerError::invalid_input("Chunk number must be greater than 0"));
        }
    }
    Ok(())
}
