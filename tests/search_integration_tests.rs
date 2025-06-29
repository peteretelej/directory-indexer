/// Search engine integration tests using real Qdrant and embedding services
use directory_indexer::{
    cli::commands::index_internal,
    embedding::EmbeddingProvider,
    search::engine::{SearchEngine, SearchQuery},
    storage::{QdrantStore, SqliteStore},
};
use std::fs;
use tempfile::TempDir;

mod common;
use common::test_env::TestEnvironment;

/// Helper to create test files with searchable content
fn create_test_files(dir: &std::path::Path) -> std::io::Result<()> {
    fs::write(
        dir.join("readme.md"),
        "# Project Documentation\n\nThis project uses machine learning algorithms for text processing.\nIt includes database connectivity and API endpoints for searching documents.",
    )?;

    fs::write(
        dir.join("main.rs"),
        "// Main Rust application\nfn main() {\n    println!(\"Starting database connection\");\n    connect_to_database();\n}\n\nfn connect_to_database() -> Result<(), String> {\n    // Database connection logic\n    Ok(())\n}",
    )?;

    fs::write(
        dir.join("api.md"),
        "# API Documentation\n\nThis document describes the REST API endpoints.\n\n## Search Endpoints\n\n- GET /api/search?q=query\n- POST /api/documents",
    )?;

    // Create subdirectory with related content
    let subdir = dir.join("algorithms");
    fs::create_dir(&subdir)?;

    fs::write(
        subdir.join("ml.py"),
        "# Machine learning algorithms\nimport numpy as np\n\ndef train_model(data):\n    \"\"\"Train a machine learning model on text data\"\"\"\n    return model",
    )?;

    Ok(())
}

/// Helper to check if required services are available
async fn are_services_available() -> bool {
    let qdrant_available = reqwest::get("http://localhost:6333/healthz")
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    let ollama_available = reqwest::get("http://localhost:11434/api/tags")
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    qdrant_available && ollama_available
}

/// Helper to create a search engine with real services
async fn create_search_engine(config: &directory_indexer::config::Config) -> SearchEngine {
    let sqlite_store = SqliteStore::new(&config.storage.sqlite_path).unwrap();

    let qdrant_store = QdrantStore::new_with_api_key(
        &config.storage.qdrant.endpoint,
        config.storage.qdrant.collection.clone(),
        config.storage.qdrant.api_key.clone(),
    )
    .await
    .expect("Failed to create Qdrant store");

    let embedding_provider: Box<dyn EmbeddingProvider> = if config.embedding.provider == "ollama" {
        Box::new(directory_indexer::embedding::ollama::OllamaProvider::new(
            config.embedding.endpoint.clone(),
            config.embedding.model.clone(),
        ))
    } else {
        panic!("Only Ollama provider supported in tests");
    };

    SearchEngine::new(sqlite_store, qdrant_store, embedding_provider)
}

#[tokio::test]
async fn test_search_engine_creation_with_real_services() {
    let _env = TestEnvironment::new("search-engine-creation").await;

    if !are_services_available().await {
        println!("Skipping test - required services not available");
        return;
    }

    let search_engine = create_search_engine(&_env.config).await;

    // Test that we can create search queries
    let query = SearchQuery {
        text: "test query".to_string(),
        directory_filter: None,
        limit: 10,
        similarity_threshold: None,
    };

    // Validate the query using the search engine
    assert!(search_engine.validate_query(&query).is_ok());
}

#[tokio::test]
async fn test_search_with_indexed_content() {
    let _env = TestEnvironment::new("search-indexed-content").await;

    if !are_services_available().await {
        println!("Skipping test - required services not available");
        return;
    }

    // Create test files and index them
    let temp_dir = TempDir::new().unwrap();
    create_test_files(temp_dir.path()).unwrap();

    // Index the test files
    let paths = vec![temp_dir.path().to_string_lossy().to_string()];
    let result = index_internal(paths, false).await;
    assert!(result.is_ok(), "Failed to index test files: {:?}", result);

    // Create search engine
    let search_engine = create_search_engine(&_env.config).await;

    // Test search functionality (currently returns empty results due to TODO implementation)
    let query = SearchQuery {
        text: "machine learning algorithms".to_string(),
        directory_filter: None,
        limit: 5,
        similarity_threshold: Some(0.7),
    };

    let search_results = search_engine.search(query).await;
    assert!(
        search_results.is_ok(),
        "Search should not fail: {:?}",
        search_results
    );

    // Currently returns empty results due to unimplemented search logic
    // This test verifies the infrastructure works correctly
    let results = search_results.unwrap();
    assert_eq!(results.len(), 0); // TODO: Change when search is implemented
}

#[tokio::test]
async fn test_search_query_validation_with_directory_filter() {
    let _env = TestEnvironment::new("search-query-validation").await;

    if !are_services_available().await {
        println!("Skipping test - required services not available");
        return;
    }

    let search_engine = create_search_engine(&_env.config).await;
    let temp_dir = TempDir::new().unwrap();

    // Test with valid directory
    let valid_query = SearchQuery {
        text: "test".to_string(),
        directory_filter: Some(temp_dir.path().to_path_buf()),
        limit: 10,
        similarity_threshold: Some(0.5),
    };

    assert!(search_engine.validate_query(&valid_query).is_ok());

    // Test with non-existent directory
    let invalid_query = SearchQuery {
        text: "test".to_string(),
        directory_filter: Some("/non/existent/directory".into()),
        limit: 10,
        similarity_threshold: Some(0.5),
    };

    let result = search_engine.validate_query(&invalid_query);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("valid directory"));
}

#[tokio::test]
async fn test_find_similar_files_infrastructure() {
    let _env = TestEnvironment::new("find-similar-files").await;

    if !are_services_available().await {
        println!("Skipping test - required services not available");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    create_test_files(temp_dir.path()).unwrap();

    let search_engine = create_search_engine(&_env.config).await;
    let file_path = temp_dir.path().join("readme.md");

    // Test similar file search (currently returns empty due to TODO implementation)
    let results = search_engine.find_similar_files(file_path, 5).await;
    assert!(
        results.is_ok(),
        "Find similar files should not fail: {:?}",
        results
    );

    // Currently returns empty results due to unimplemented logic
    let similar_files = results.unwrap();
    assert_eq!(similar_files.len(), 0); // TODO: Change when implementation is done
}

#[tokio::test]
async fn test_get_file_content_not_implemented() {
    let _env = TestEnvironment::new("get-file-content").await;

    if !are_services_available().await {
        println!("Skipping test - required services not available");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.md");
    fs::write(&file_path, "Test file content").unwrap();

    let search_engine = create_search_engine(&_env.config).await;

    // Test that get_file_content returns the expected error
    let result = search_engine.get_file_content(file_path, None).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not implemented"));
}

#[tokio::test]
async fn test_search_result_filtering_and_ranking() {
    let _env = TestEnvironment::new("search-result-processing").await;

    if !are_services_available().await {
        println!("Skipping test - required services not available");
        return;
    }

    let search_engine = create_search_engine(&_env.config).await;

    // Create mock search results for testing utility functions
    use directory_indexer::search::engine::SearchResult;
    use std::path::PathBuf;

    let mock_results = vec![
        SearchResult {
            file_path: PathBuf::from("/tmp/test/docs/readme.md"),
            chunk_id: 0,
            score: 0.9,
            content_snippet: Some("Test content".to_string()),
            parent_directories: vec!["docs".to_string()],
            file_size: 1024,
            modified_time: 1234567890,
        },
        SearchResult {
            file_path: PathBuf::from("/tmp/test/code/main.rs"),
            chunk_id: 1,
            score: 0.6,
            content_snippet: Some("Code content".to_string()),
            parent_directories: vec!["code".to_string()],
            file_size: 512,
            modified_time: 1234567891,
        },
    ];

    // Test directory filtering
    let docs_filter = Some(PathBuf::from("/tmp/test/docs"));
    let filtered = search_engine.filter_results_by_directory(mock_results.clone(), &docs_filter);
    assert_eq!(filtered.len(), 1);
    assert!(filtered[0].file_path.starts_with("/tmp/test/docs"));

    // Test similarity threshold filtering
    let high_threshold = Some(0.8);
    let threshold_filtered =
        search_engine.apply_similarity_threshold(mock_results.clone(), high_threshold);
    assert_eq!(threshold_filtered.len(), 1);
    assert!(threshold_filtered[0].score >= 0.8);

    // Test ranking
    let ranked = search_engine.rank_results(mock_results.clone());
    assert_eq!(ranked[0].score, 0.9); // Higher score first
    assert_eq!(ranked[1].score, 0.6);

    // Test limiting results
    let limited = search_engine.limit_results(mock_results, 1);
    assert_eq!(limited.len(), 1);
}
