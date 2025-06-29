use directory_indexer::{Config, IndexerError};
use std::env;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_config_load_with_missing_directories() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let original_data_dir = env::var("DIRECTORY_INDEXER_DATA_DIR").ok();

    // Set data dir to a non-existent nested path
    let deep_path = temp_dir
        .path()
        .join("very")
        .join("deep")
        .join("nested")
        .join("path");
    env::set_var("DIRECTORY_INDEXER_DATA_DIR", &deep_path);

    let result = Config::load();
    assert!(result.is_ok());

    let config = result.unwrap();
    assert_eq!(config.storage.sqlite_path, deep_path.join("data.db"));

    // The load() method should have created the directory structure
    assert!(deep_path.exists());

    // Restore original
    if let Some(val) = original_data_dir {
        env::set_var("DIRECTORY_INDEXER_DATA_DIR", val);
    } else {
        env::remove_var("DIRECTORY_INDEXER_DATA_DIR");
    }
}

#[test]
fn test_config_save_creates_missing_directories() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let original_data_dir = env::var("DIRECTORY_INDEXER_DATA_DIR").ok();

    let nested_path = temp_dir.path().join("config").join("subdir");
    env::set_var("DIRECTORY_INDEXER_DATA_DIR", &nested_path);

    let config = Config::load().expect("Config should load");
    let result = config.save();

    assert!(result.is_ok());
    assert!(nested_path.join("config.json").exists());

    let content =
        fs::read_to_string(nested_path.join("config.json")).expect("Should read config file");
    assert!(content.contains("directory-indexer"));

    // Restore original
    if let Some(val) = original_data_dir {
        env::set_var("DIRECTORY_INDEXER_DATA_DIR", val);
    } else {
        env::remove_var("DIRECTORY_INDEXER_DATA_DIR");
    }
}

#[test]
fn test_config_with_all_environment_variables() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Save all original values
    let original_qdrant = env::var("QDRANT_ENDPOINT").ok();
    let original_ollama = env::var("OLLAMA_ENDPOINT").ok();
    let original_data_dir = env::var("DIRECTORY_INDEXER_DATA_DIR").ok();
    let original_collection = env::var("DIRECTORY_INDEXER_QDRANT_COLLECTION").ok();
    let original_qdrant_key = env::var("QDRANT_API_KEY").ok();
    let original_ollama_key = env::var("OLLAMA_API_KEY").ok();

    // Set comprehensive test values
    env::set_var("QDRANT_ENDPOINT", "https://my-qdrant.example.com:6333");
    env::set_var("OLLAMA_ENDPOINT", "https://my-ollama.example.com:11434");
    env::set_var("DIRECTORY_INDEXER_DATA_DIR", temp_dir.path());
    env::set_var(
        "DIRECTORY_INDEXER_QDRANT_COLLECTION",
        "my-custom-collection",
    );
    env::set_var("QDRANT_API_KEY", "super-secret-qdrant-key");
    env::set_var("OLLAMA_API_KEY", "super-secret-ollama-key");

    let config = Config::load().expect("Config should load successfully");

    // Verify all values are correctly loaded
    assert_eq!(
        config.storage.qdrant.endpoint,
        "https://my-qdrant.example.com:6333"
    );
    assert_eq!(
        config.embedding.endpoint,
        "https://my-ollama.example.com:11434"
    );
    assert_eq!(config.storage.sqlite_path, temp_dir.path().join("data.db"));
    assert_eq!(config.storage.qdrant.collection, "my-custom-collection");
    assert_eq!(
        config.storage.qdrant.api_key,
        Some("super-secret-qdrant-key".to_string())
    );
    assert_eq!(
        config.embedding.api_key,
        Some("super-secret-ollama-key".to_string())
    );

    // Verify other defaults are preserved
    assert_eq!(config.embedding.provider, "ollama");
    assert_eq!(config.embedding.model, "nomic-embed-text");
    assert_eq!(config.indexing.chunk_size, 512);
    assert_eq!(config.indexing.concurrency, 4);

    // Save and verify persistence
    let save_result = config.save();
    assert!(save_result.is_ok());

    let config_path = temp_dir.path().join("config.json");
    assert!(config_path.exists());

    let saved_content = fs::read_to_string(&config_path).expect("Should read saved config");
    assert!(saved_content.contains("https://my-qdrant.example.com:6333"));
    assert!(saved_content.contains("my-custom-collection"));

    // Restore all original values
    if let Some(val) = original_qdrant {
        env::set_var("QDRANT_ENDPOINT", val);
    } else {
        env::remove_var("QDRANT_ENDPOINT");
    }
    if let Some(val) = original_ollama {
        env::set_var("OLLAMA_ENDPOINT", val);
    } else {
        env::remove_var("OLLAMA_ENDPOINT");
    }
    if let Some(val) = original_data_dir {
        env::set_var("DIRECTORY_INDEXER_DATA_DIR", val);
    } else {
        env::remove_var("DIRECTORY_INDEXER_DATA_DIR");
    }
    if let Some(val) = original_collection {
        env::set_var("DIRECTORY_INDEXER_QDRANT_COLLECTION", val);
    } else {
        env::remove_var("DIRECTORY_INDEXER_QDRANT_COLLECTION");
    }
    if let Some(val) = original_qdrant_key {
        env::set_var("QDRANT_API_KEY", val);
    } else {
        env::remove_var("QDRANT_API_KEY");
    }
    if let Some(val) = original_ollama_key {
        env::set_var("OLLAMA_API_KEY", val);
    } else {
        env::remove_var("OLLAMA_API_KEY");
    }
}

#[test]
fn test_config_test_collection_uniqueness() {
    let original_collection = env::var("DIRECTORY_INDEXER_QDRANT_COLLECTION").ok();

    env::set_var("DIRECTORY_INDEXER_QDRANT_COLLECTION", "test");

    let config1 = Config::load().expect("First config should load");
    let config2 = Config::load().expect("Second config should load");

    // Both should start with the test prefix but be unique
    assert!(config1
        .storage
        .qdrant
        .collection
        .starts_with("directory-indexer-test-"));
    assert!(config2
        .storage
        .qdrant
        .collection
        .starts_with("directory-indexer-test-"));
    assert_ne!(
        config1.storage.qdrant.collection,
        config2.storage.qdrant.collection
    );

    // Both should contain the process ID
    let process_id = std::process::id().to_string();
    assert!(config1.storage.qdrant.collection.contains(&process_id));
    assert!(config2.storage.qdrant.collection.contains(&process_id));

    if let Some(val) = original_collection {
        env::set_var("DIRECTORY_INDEXER_QDRANT_COLLECTION", val);
    } else {
        env::remove_var("DIRECTORY_INDEXER_QDRANT_COLLECTION");
    }
}

#[test]
fn test_config_invalid_json_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let original_data_dir = env::var("DIRECTORY_INDEXER_DATA_DIR").ok();

    env::set_var("DIRECTORY_INDEXER_DATA_DIR", temp_dir.path());

    // Create an invalid JSON config file
    let config_path = temp_dir.path().join("config.json");
    fs::write(&config_path, "{ invalid json content }").expect("Should write invalid JSON");

    // Config load should still succeed (it uses environment variables as primary)
    let result = Config::load();
    assert!(result.is_ok());

    if let Some(val) = original_data_dir {
        env::set_var("DIRECTORY_INDEXER_DATA_DIR", val);
    } else {
        env::remove_var("DIRECTORY_INDEXER_DATA_DIR");
    }
}

#[test]
fn test_config_serialization_roundtrip() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let original_data_dir = env::var("DIRECTORY_INDEXER_DATA_DIR").ok();

    env::set_var("DIRECTORY_INDEXER_DATA_DIR", temp_dir.path());
    env::set_var("QDRANT_ENDPOINT", "http://test:6333");
    env::set_var("OLLAMA_ENDPOINT", "http://test:11434");

    let original_config = Config::load().expect("Config should load");

    // Save to file
    let save_result = original_config.save();
    assert!(save_result.is_ok());

    // Read the JSON directly and deserialize
    let config_path = temp_dir.path().join("config.json");
    let json_content = fs::read_to_string(&config_path).expect("Should read config file");

    let deserialized_config: Config =
        serde_json::from_str(&json_content).expect("Should deserialize config");

    // Compare key fields
    assert_eq!(
        original_config.storage.qdrant.endpoint,
        deserialized_config.storage.qdrant.endpoint
    );
    assert_eq!(
        original_config.embedding.endpoint,
        deserialized_config.embedding.endpoint
    );
    assert_eq!(
        original_config.indexing.chunk_size,
        deserialized_config.indexing.chunk_size
    );
    assert_eq!(
        original_config.monitoring.batch_size,
        deserialized_config.monitoring.batch_size
    );

    env::remove_var("QDRANT_ENDPOINT");
    env::remove_var("OLLAMA_ENDPOINT");

    if let Some(val) = original_data_dir {
        env::set_var("DIRECTORY_INDEXER_DATA_DIR", val);
    } else {
        env::remove_var("DIRECTORY_INDEXER_DATA_DIR");
    }
}
