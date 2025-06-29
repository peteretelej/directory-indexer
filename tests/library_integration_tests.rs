/// Library integration tests that call CLI command functions directly
/// This provides better code coverage than spawning CLI processes
use directory_indexer::cli::commands::{
    get_internal, index_internal, search_internal, similar_internal, status,
};
use std::fs;
use tempfile::TempDir;

mod common;
use common::test_env::TestEnvironment;

/// Helper to create test files with content
fn create_test_files(dir: &std::path::Path) -> std::io::Result<()> {
    fs::write(
        dir.join("README.md"),
        "# Test Project\n\nThis is a test project for directory indexer.\nIt contains sample documentation and code files.",
    )?;
    
    fs::write(
        dir.join("main.rs"),
        "// Main Rust file\nfn main() {\n    println!(\"Hello, world!\");\n}\n\n// Database connection function\nfn connect_db() -> Result<(), String> {\n    // Connection logic here\n    Ok(())\n}",
    )?;
    
    fs::write(
        dir.join("config.json"),
        r#"{"database": {"host": "localhost", "port": 5432}, "logging": {"level": "info"}}"#,
    )?;
    
    // Create a subdirectory with more files
    let subdir = dir.join("docs");
    fs::create_dir(&subdir)?;
    
    fs::write(
        subdir.join("api.md"),
        "# API Documentation\n\nThis document describes the API endpoints.\n\n## Database Endpoints\n\n- GET /api/users\n- POST /api/users",
    )?;
    
    fs::write(
        subdir.join("setup.md"),
        "# Setup Guide\n\nHow to set up the development environment.\n\n## Prerequisites\n\n- Rust\n- PostgreSQL database\n- Docker",
    )?;
    
    Ok(())
}

/// Test indexing functionality through library
#[tokio::test]
async fn test_library_index_command() {
    let _env = TestEnvironment::new("library-index-command").await;
    
    // Skip if services aren't available
    if !are_services_available().await {
        println!("Skipping test - required services not available");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    create_test_files(temp_dir.path()).unwrap();
    
    let paths = vec![temp_dir.path().to_string_lossy().to_string()];
    
    // Call the library function directly (no console output)
    let result = index_internal(paths, false).await;
    assert!(result.is_ok(), "Index command should succeed: {:?}", result);
}

/// Test search functionality through library
#[tokio::test]
async fn test_library_search_command() {
    let _env = TestEnvironment::new("library-search-command").await;
    
    // Skip if services aren't available
    if !are_services_available().await {
        println!("Skipping test - required services not available");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    create_test_files(temp_dir.path()).unwrap();
    
    // First index the directory
    let paths = vec![temp_dir.path().to_string_lossy().to_string()];
    let index_result = index_internal(paths, false).await;
    assert!(index_result.is_ok(), "Index should succeed before search");
    
    // Test various search queries
    let test_queries = vec![
        "database connection",
        "API documentation", 
        "setup guide",
        "Rust programming",
        "PostgreSQL",
    ];
    
    for query in test_queries {
        let result = search_internal(query.to_string(), None, Some(5), false).await;
        assert!(result.is_ok(), "Search for '{}' should succeed: {:?}", query, result);
    }
}

/// Test search with path scope
#[tokio::test]
async fn test_library_search_with_path_scope() {
    let _env = TestEnvironment::new("library-search-with-path").await;
    
    if !are_services_available().await {
        println!("Skipping test - required services not available");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    create_test_files(temp_dir.path()).unwrap();
    
    // Index the directory
    let paths = vec![temp_dir.path().to_string_lossy().to_string()];
    let index_result = index_internal(paths, false).await;
    assert!(index_result.is_ok());
    
    // Search with path scope
    let result = search_internal(
        "documentation".to_string(),
        Some(temp_dir.path().join("docs").to_string_lossy().to_string()),
        Some(10),
        false,
    ).await;
    assert!(result.is_ok(), "Search with path scope should succeed: {:?}", result);
}

/// Test similar files functionality
#[tokio::test]
async fn test_library_similar_command() {
    let _env = TestEnvironment::new("library-similar-command").await;
    
    if !are_services_available().await {
        println!("Skipping test - required services not available");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    create_test_files(temp_dir.path()).unwrap();
    
    // Index the directory
    let paths = vec![temp_dir.path().to_string_lossy().to_string()];
    let index_result = index_internal(paths, false).await;
    assert!(index_result.is_ok());
    
    // Test finding similar files
    let test_file = temp_dir.path().join("README.md");
    let result = similar_internal(test_file.to_string_lossy().to_string(), 5, false).await;
    assert!(result.is_ok(), "Similar files command should succeed: {:?}", result);
}

/// Test get content functionality
#[tokio::test]
async fn test_library_get_command() {
    let _env = TestEnvironment::new("library-get-command").await;
    
    let temp_dir = TempDir::new().unwrap();
    create_test_files(temp_dir.path()).unwrap();
    
    // Test getting full file content
    let test_file = temp_dir.path().join("README.md");
    let result = get_internal(test_file.to_string_lossy().to_string(), None, false).await;
    assert!(result.is_ok(), "Get full file content should succeed: {:?}", result);
    
    // Test getting specific chunks
    let result = get_internal(
        test_file.to_string_lossy().to_string(),
        Some("1".to_string()),
        false,
    ).await;
    assert!(result.is_ok(), "Get specific chunk should succeed: {:?}", result);
    
    // Test getting chunk range
    let result = get_internal(
        test_file.to_string_lossy().to_string(),
        Some("1-2".to_string()),
        false,
    ).await;
    assert!(result.is_ok(), "Get chunk range should succeed: {:?}", result);
}

/// Test status functionality
#[tokio::test]
async fn test_library_status_command() {
    let _env = TestEnvironment::new("library-status-command").await;
    
    // Test text format
    let result = status("text".to_string()).await;
    assert!(result.is_ok(), "Status with text format should succeed: {:?}", result);
    
    // Test JSON format
    let result = status("json".to_string()).await;
    assert!(result.is_ok(), "Status with JSON format should succeed: {:?}", result);
    
    // Test invalid format
    let result = status("invalid".to_string()).await;
    assert!(result.is_err(), "Status with invalid format should fail");
}

/// Test error handling for invalid inputs
#[tokio::test]
async fn test_library_error_handling() {
    let _env = TestEnvironment::new("library-error-handling").await;
    
    // Test index with non-existent directory
    let result = index_internal(vec!["/nonexistent/path".to_string()], false).await;
    assert!(result.is_err(), "Index with non-existent path should fail");
    
    // Test search with empty query
    let result = search_internal("".to_string(), None, None, false).await;
    assert!(result.is_err(), "Search with empty query should fail");
    
    // Test similar with non-existent file
    let result = similar_internal("/nonexistent/file.txt".to_string(), 5, false).await;
    assert!(result.is_err(), "Similar with non-existent file should fail");
    
    // Test get with non-existent file
    let result = get_internal("/nonexistent/file.txt".to_string(), None, false).await;
    assert!(result.is_err(), "Get with non-existent file should fail");
}

/// Test end-to-end workflow through library
#[tokio::test]
async fn test_library_end_to_end_workflow() {
    let _env = TestEnvironment::new("library-end-to-end").await;
    
    if !are_services_available().await {
        println!("Skipping test - required services not available");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    create_test_files(temp_dir.path()).unwrap();
    
    // Step 1: Index the directory
    let paths = vec![temp_dir.path().to_string_lossy().to_string()];
    let index_result = index_internal(paths, false).await;
    assert!(index_result.is_ok(), "Index should succeed");
    
    // Step 2: Check status
    let status_result = status("text".to_string()).await;
    assert!(status_result.is_ok(), "Status should succeed");
    
    // Step 3: Search for content
    let search_result = search_internal("database".to_string(), None, Some(5), false).await;
    assert!(search_result.is_ok(), "Search should succeed");
    
    // Step 4: Find similar files
    let readme_path = temp_dir.path().join("README.md");
    let similar_result = similar_internal(readme_path.to_string_lossy().to_string(), 3, false).await;
    assert!(similar_result.is_ok(), "Similar files should succeed");
    
    // Step 5: Get file content
    let get_result = get_internal(readme_path.to_string_lossy().to_string(), None, false).await;
    assert!(get_result.is_ok(), "Get content should succeed");
}

/// Test indexing multiple directories
#[tokio::test]
async fn test_library_index_multiple_directories() {
    let _env = TestEnvironment::new("library-index-multiple").await;
    
    if !are_services_available().await {
        println!("Skipping test - required services not available");
        return;
    }

    let temp_dir1 = TempDir::new().unwrap();
    let temp_dir2 = TempDir::new().unwrap();
    
    // Create different content in each directory
    fs::write(temp_dir1.path().join("file1.md"), "Content from directory 1").unwrap();
    fs::write(temp_dir2.path().join("file2.md"), "Content from directory 2").unwrap();
    
    let paths = vec![
        temp_dir1.path().to_string_lossy().to_string(),
        temp_dir2.path().to_string_lossy().to_string(),
    ];
    
    let result = index_internal(paths, false).await;
    assert!(result.is_ok(), "Index multiple directories should succeed: {:?}", result);
}

/// Test chunk parsing and validation
#[tokio::test]
async fn test_library_chunk_operations() {
    let _env = TestEnvironment::new("library-chunk-operations").await;
    
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    
    // Create a file with multiple lines for chunking
    let content = (0..50)
        .map(|i| format!("Line {} with some content to make it longer", i))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&test_file, content).unwrap();
    
    // Test valid chunk ranges
    let valid_chunks = vec!["1", "1-3", "2-5"];
    
    for chunk_spec in valid_chunks {
        let result = get_internal(
            test_file.to_string_lossy().to_string(),
            Some(chunk_spec.to_string()),
            false,
        ).await;
        assert!(result.is_ok(), "Get with chunk '{}' should succeed: {:?}", chunk_spec, result);
    }
    
    // Test invalid chunk ranges
    let invalid_chunks = vec!["0", "invalid", "1-", "-1", "5-2"];
    
    for chunk_spec in invalid_chunks {
        let result = get_internal(
            test_file.to_string_lossy().to_string(),
            Some(chunk_spec.to_string()),
            false,
        ).await;
        assert!(result.is_err(), "Get with invalid chunk '{}' should fail", chunk_spec);
    }
}

/// Helper function to check if services are available
async fn are_services_available() -> bool {
    use directory_indexer::Config;
    
    let config = match Config::load() {
        Ok(config) => config,
        Err(_) => return false,
    };
    
    // Check if both services are available
    let health = directory_indexer::health::check_system_health(&config).await;
    health.qdrant_available && health.ollama_available
}