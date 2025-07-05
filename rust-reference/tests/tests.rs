// Unit tests - no external services required

use clap::Parser;
use directory_indexer::{
    storage::{FileRecord, SqliteStore},
    Config,
};
use serde_json::json;
use std::fs;
use tempfile::{NamedTempFile, TempDir};

// ============================================================================
// CLI Argument Parsing Tests
// ============================================================================

#[derive(Parser)]
#[command(name = "directory-indexer")]
#[command(about = "AI-powered directory indexing with semantic search")]
#[command(version)]
struct TestCli {
    #[command(subcommand)]
    command: TestCommands,

    #[arg(short, long, global = true)]
    verbose: bool,

    #[arg(short, long, global = true)]
    config: Option<String>,
}

#[derive(clap::Subcommand)]
enum TestCommands {
    Index {
        paths: Vec<String>,
    },
    Search {
        query: String,
        #[arg(short, long)]
        path: Option<String>,
        #[arg(short, long)]
        limit: Option<usize>,
    },
    Similar {
        file: String,
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    Get {
        file: String,
    },
    Status,
    Serve,
}

#[test]
fn test_cli_parsing_basic_commands() {
    // Test index command
    let cli = TestCli::try_parse_from(&["program", "index", "/path1", "/path2"]).unwrap();
    match cli.command {
        TestCommands::Index { paths } => {
            assert_eq!(paths, vec!["/path1", "/path2"]);
        }
        _ => panic!("Expected Index command"),
    }

    // Test search command
    let cli = TestCli::try_parse_from(&["program", "search", "test query"]).unwrap();
    match cli.command {
        TestCommands::Search { query, path, limit } => {
            assert_eq!(query, "test query");
            assert_eq!(path, None);
            assert_eq!(limit, None);
        }
        _ => panic!("Expected Search command"),
    }

    // Test search with options
    let cli = TestCli::try_parse_from(&[
        "program", "search", "test", "--path", "/docs", "--limit", "5",
    ])
    .unwrap();
    match cli.command {
        TestCommands::Search { query, path, limit } => {
            assert_eq!(query, "test");
            assert_eq!(path, Some("/docs".to_string()));
            assert_eq!(limit, Some(5));
        }
        _ => panic!("Expected Search command with options"),
    }

    // Test similar command
    let cli = TestCli::try_parse_from(&["program", "similar", "/path/file.txt"]).unwrap();
    match cli.command {
        TestCommands::Similar { file, limit } => {
            assert_eq!(file, "/path/file.txt");
            assert_eq!(limit, 10); // default
        }
        _ => panic!("Expected Similar command"),
    }
}

#[test]
fn test_cli_global_flags() {
    let cli = TestCli::try_parse_from(&["program", "--verbose", "status"]).unwrap();
    assert!(cli.verbose);

    let cli =
        TestCli::try_parse_from(&["program", "--config", "/path/config.json", "status"]).unwrap();
    assert_eq!(cli.config, Some("/path/config.json".to_string()));
}

#[test]
fn test_cli_validation() {
    // Missing required argument should fail
    assert!(TestCli::try_parse_from(&["program", "search"]).is_err());
    assert!(TestCli::try_parse_from(&["program", "similar"]).is_err());
    assert!(TestCli::try_parse_from(&["program", "get"]).is_err());
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[test]
fn test_config_defaults() {
    // Test default config without any environment variable influence
    let config = Config::default();

    assert_eq!(config.embedding.provider, "ollama");
    assert_eq!(config.embedding.model, "nomic-embed-text");
    assert_eq!(config.indexing.chunk_size, 512);
    assert_eq!(config.indexing.concurrency, 4);
    // Default config should use "directory-indexer-test" when running under cargo
    assert_eq!(config.storage.qdrant.collection, "directory-indexer-test");
    assert_eq!(config.monitoring.batch_size, 100);
    assert!(!config.monitoring.file_watching);
    assert!(config.storage.qdrant.endpoint.starts_with("http://"));
    assert!(config.embedding.endpoint.starts_with("http://"));
    assert_eq!(config.storage.qdrant.api_key, None);
    assert_eq!(config.embedding.api_key, None);
}

#[test]
fn test_config_serialization() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = Config::load().expect("Config should load");

    let mut test_config = config.clone();
    test_config.storage.sqlite_path = temp_dir.path().join("test.db");

    let config_path = temp_dir.path().join("config.json");
    let json = serde_json::to_string_pretty(&test_config).expect("Should serialize");
    fs::write(&config_path, json).expect("Should write config file");

    assert!(config_path.exists());
    let content = fs::read_to_string(&config_path).expect("Should read config file");

    assert!(content.contains("directory-indexer"));
    assert!(content.contains("ollama"));
    assert!(content.contains("nomic-embed-text"));
    assert!(content.contains("512"));

    let deserialized: Config = serde_json::from_str(&content).expect("Should deserialize");
    assert_eq!(deserialized.embedding.provider, "ollama");
    assert_eq!(deserialized.indexing.chunk_size, 512);
    assert_eq!(
        deserialized.storage.sqlite_path,
        temp_dir.path().join("test.db")
    );
}

// ============================================================================
// Storage Tests
// ============================================================================

#[test]
fn test_sqlite_store_creation() {
    let temp_db = NamedTempFile::new().unwrap();
    let store = SqliteStore::new(temp_db.path()).expect("Failed to create SQLite store");

    let (dir_count, file_count, chunk_count) = store.get_stats().expect("Failed to get stats");
    assert_eq!(dir_count, 0);
    assert_eq!(file_count, 0);
    assert_eq!(chunk_count, 0);
}

#[test]
fn test_directory_operations() {
    let temp_db = NamedTempFile::new().unwrap();
    let store = SqliteStore::new(temp_db.path()).unwrap();

    let dir_path = "/home/user/documents";
    let dir_id = store
        .add_directory(dir_path)
        .expect("Failed to add directory");
    assert!(dir_id > 0);

    let directories = store.get_directories().expect("Failed to get directories");
    assert_eq!(directories.len(), 1);
    assert_eq!(directories[0].path, dir_path);
    assert_eq!(directories[0].status, "pending");

    store
        .update_directory_status(dir_path, "completed")
        .expect("Failed to update status");
    let directories = store
        .get_directories()
        .expect("Failed to get directories after update");
    assert_eq!(directories[0].status, "completed");
}

#[test]
fn test_file_operations() {
    let temp_db = NamedTempFile::new().unwrap();
    let store = SqliteStore::new(temp_db.path()).unwrap();

    // Test file record with actual structure
    let file_record = FileRecord {
        id: 0, // Will be set by database
        path: "/test/file.txt".to_string(),
        size: 12,
        modified_time: 1640995200, // 2022-01-01 timestamp
        hash: "abc123".to_string(),
        parent_dirs: vec!["/test".to_string()],
        chunks_json: Some(json!({"chunks": [{"start": 0, "end": 12}]})),
        errors_json: None,
    };

    let file_id = store.add_file(&file_record).expect("Failed to add file");
    assert!(file_id > 0);

    let retrieved_file = store
        .get_file_by_path("/test/file.txt")
        .expect("Failed to get file");
    assert!(retrieved_file.is_some());
    let file = retrieved_file.unwrap();
    assert_eq!(file.path, "/test/file.txt");
    assert_eq!(file.size, 12);
    assert_eq!(file.hash, "abc123");
}

#[test]
fn test_file_deletion() {
    let temp_db = NamedTempFile::new().unwrap();
    let store = SqliteStore::new(temp_db.path()).unwrap();

    let file_record = FileRecord {
        id: 0,
        path: "/test/file.txt".to_string(),
        size: 12,
        modified_time: 1640995200,
        hash: "abc123".to_string(),
        parent_dirs: vec!["/test".to_string()],
        chunks_json: None,
        errors_json: None,
    };

    store.add_file(&file_record).expect("Failed to add file");

    // Verify file exists
    let file = store
        .get_file_by_path("/test/file.txt")
        .expect("Failed to get file");
    assert!(file.is_some());

    // Delete file
    store
        .delete_file("/test/file.txt")
        .expect("Failed to delete file");

    // Verify file is gone
    let file = store
        .get_file_by_path("/test/file.txt")
        .expect("Failed to get file");
    assert!(file.is_none());
}

#[test]
fn test_stats() {
    let temp_db = NamedTempFile::new().unwrap();
    let store = SqliteStore::new(temp_db.path()).unwrap();

    // Initial stats should be zero
    let (dir_count, file_count, chunk_count) = store.get_stats().expect("Failed to get stats");
    assert_eq!(dir_count, 0);
    assert_eq!(file_count, 0);
    assert_eq!(chunk_count, 0);

    // Add directory
    store
        .add_directory("/test")
        .expect("Failed to add directory");
    let (dir_count, _, _) = store.get_stats().expect("Failed to get stats");
    assert_eq!(dir_count, 1);

    // Add file with chunks
    let file_record = FileRecord {
        id: 0,
        path: "/test/file.txt".to_string(),
        size: 12,
        modified_time: 1640995200,
        hash: "abc123".to_string(),
        parent_dirs: vec!["/test".to_string()],
        chunks_json: Some(json!({"chunks": [{"start": 0, "end": 12}]})),
        errors_json: None,
    };
    store.add_file(&file_record).expect("Failed to add file");

    let (dir_count, file_count, chunk_count) = store.get_stats().expect("Failed to get stats");
    assert_eq!(dir_count, 1);
    assert_eq!(file_count, 1);
    assert_eq!(chunk_count, 1); // Files with chunks_json
}
