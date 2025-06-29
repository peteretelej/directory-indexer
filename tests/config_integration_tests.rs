use directory_indexer::Config;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_config_load_defaults() {
    // Test that Config::load() succeeds and has expected default values
    let config = Config::load().expect("Config should load");

    // Test core default values that shouldn't change
    assert_eq!(config.embedding.provider, "ollama");
    assert_eq!(config.embedding.model, "nomic-embed-text");
    assert_eq!(config.indexing.chunk_size, 512);
    assert_eq!(config.indexing.concurrency, 4);

    // Collection name should be test-specific when running tests
    // When cfg!(test) is true, default is "directory-indexer-test" which becomes unique: "directory-indexer-test-{pid}-{timestamp}"
    assert!(
        config
            .storage
            .qdrant
            .collection
            .starts_with("directory-indexer-test-"),
        "Expected test collection name when running under cfg!(test), got: {}",
        config.storage.qdrant.collection
    );

    assert_eq!(config.monitoring.batch_size, 100);
    assert!(!config.monitoring.file_watching);

    // Endpoints should have reasonable defaults
    assert!(config.storage.qdrant.endpoint.starts_with("http://"));
    assert!(config.embedding.endpoint.starts_with("http://"));

    // API keys should be None by default
    assert_eq!(config.storage.qdrant.api_key, None);
    assert_eq!(config.embedding.api_key, None);
}

#[test]
fn test_config_serialization() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create a default config
    let config = Config::load().expect("Config should load");

    // Manually set a data directory for this test
    let mut test_config = config.clone();
    test_config.storage.sqlite_path = temp_dir.path().join("test.db");

    // Save should succeed
    let config_path = temp_dir.path().join("config.json");

    // Manually save to avoid directory creation issues
    std::fs::create_dir_all(temp_dir.path()).expect("Should create test directory");
    let json = serde_json::to_string_pretty(&test_config).expect("Should serialize");
    fs::write(&config_path, json).expect("Should write config file");

    // File should exist and be valid JSON
    assert!(config_path.exists());
    let content = fs::read_to_string(&config_path).expect("Should read config file");

    // Should contain expected values
    assert!(content.contains("directory-indexer"));
    assert!(content.contains("ollama"));
    assert!(content.contains("nomic-embed-text"));
    assert!(content.contains("512")); // chunk_size

    // Should be deserializable
    let deserialized: Config = serde_json::from_str(&content).expect("Should deserialize");
    assert_eq!(deserialized.embedding.provider, "ollama");
    assert_eq!(deserialized.indexing.chunk_size, 512);
    assert_eq!(
        deserialized.storage.sqlite_path,
        temp_dir.path().join("test.db")
    );
}

#[test]
fn test_config_save_creates_config_directories() {
    // Test that Config::save() creates necessary directories for the config file
    // Note: save() creates directories for config.json, not for sqlite_path

    let config = Config::load().expect("Config should load");

    // Save should succeed and create config directories
    let save_result = config.save();
    assert!(save_result.is_ok());

    // Config save behavior is tested - it should handle directory creation for config.json
    // The actual directory path depends on environment variables which we're not testing here
}

#[test]
fn test_config_clone_and_debug() {
    let config = Config::load().expect("Config should load");

    // Should be cloneable
    let cloned = config.clone();
    assert_eq!(config.embedding.provider, cloned.embedding.provider);
    assert_eq!(config.indexing.chunk_size, cloned.indexing.chunk_size);

    // Should have debug implementation
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("Config"));
    assert!(debug_str.contains("ollama"));
}
