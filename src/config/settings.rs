use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub storage: StorageConfig,
    pub embedding: EmbeddingConfig,
    pub indexing: IndexingConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub sqlite_path: PathBuf,
    pub qdrant: QdrantConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantConfig {
    pub endpoint: String,
    pub collection: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub provider: String,
    pub model: String,
    pub endpoint: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingConfig {
    pub chunk_size: usize,
    pub overlap: usize,
    pub max_file_size: u64,
    pub ignore_patterns: Vec<String>,
    pub concurrency: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub file_watching: bool,
    pub batch_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        let app_dir = Self::default_app_dir();

        Self {
            storage: StorageConfig {
                sqlite_path: app_dir.join("data.db"),
                qdrant: QdrantConfig {
                    endpoint: "http://localhost:6333".to_string(),
                    collection: if std::env::var("CARGO_PKG_NAME").is_ok()
                        && std::env::var("CARGO_MANIFEST_DIR").is_ok()
                    {
                        // We're running under cargo (likely tests or development)
                        "directory-indexer-test".to_string()
                    } else {
                        "directory-indexer".to_string()
                    },
                    api_key: None,
                },
            },
            embedding: EmbeddingConfig {
                provider: "ollama".to_string(),
                model: "nomic-embed-text".to_string(),
                endpoint: "http://localhost:11434".to_string(),
                api_key: None,
            },
            indexing: IndexingConfig {
                chunk_size: 512,
                overlap: 50,
                max_file_size: 10 * 1024 * 1024, // 10MB
                ignore_patterns: vec![
                    ".git".to_string(),
                    "node_modules".to_string(),
                    "target".to_string(),
                ],
                concurrency: 4,
            },
            monitoring: MonitoringConfig {
                file_watching: false,
                batch_size: 100,
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        // Start with defaults
        let mut config = Self::default();

        // Ensure the app directory exists
        config.ensure_app_dir_exists()?;

        // Use environment variables as primary source
        if let Ok(qdrant_endpoint) = std::env::var("QDRANT_ENDPOINT") {
            config.storage.qdrant.endpoint = qdrant_endpoint;
        }

        if let Ok(ollama_endpoint) = std::env::var("OLLAMA_ENDPOINT") {
            config.embedding.endpoint = ollama_endpoint;
        }

        // Override app directory if specified
        if let Ok(app_dir) = std::env::var("DIRECTORY_INDEXER_DATA_DIR") {
            let app_dir_path = PathBuf::from(app_dir);
            config.storage.sqlite_path = app_dir_path.join("data.db");
        }

        // Handle environment variable override
        if let Ok(qdrant_collection) = std::env::var("DIRECTORY_INDEXER_QDRANT_COLLECTION") {
            config.storage.qdrant.collection = qdrant_collection;
        }

        // If collection name is "test" or "directory-indexer-test", make it unique per process for test isolation
        if config.storage.qdrant.collection == "test"
            || config.storage.qdrant.collection == "directory-indexer-test"
        {
            config.storage.qdrant.collection = format!(
                "directory-indexer-test-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            );
        }

        if let Ok(qdrant_api_key) = std::env::var("QDRANT_API_KEY") {
            config.storage.qdrant.api_key = Some(qdrant_api_key);
        }

        if let Ok(ollama_api_key) = std::env::var("OLLAMA_API_KEY") {
            config.embedding.api_key = Some(ollama_api_key);
        }

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::default_config_path()?;

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, json)?;

        Ok(())
    }

    fn default_app_dir() -> PathBuf {
        std::env::var("DIRECTORY_INDEXER_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".directory-indexer")
            })
    }

    fn default_config_path() -> Result<PathBuf> {
        Ok(Self::default_app_dir().join("config.json"))
    }

    fn ensure_app_dir_exists(&self) -> Result<()> {
        if let Some(parent_dir) = self.storage.sqlite_path.parent() {
            if !parent_dir.exists() {
                std::fs::create_dir_all(parent_dir)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();

        assert_eq!(config.storage.qdrant.endpoint, "http://localhost:6333");
        // Collection name depends on cargo environment, test that it's one of the expected values
        assert!(
            config.storage.qdrant.collection == "directory-indexer-test"
                || config.storage.qdrant.collection == "directory-indexer"
        );
        assert!(config.storage.qdrant.api_key.is_none());

        assert_eq!(config.embedding.provider, "ollama");
        assert_eq!(config.embedding.model, "nomic-embed-text");
        assert_eq!(config.embedding.endpoint, "http://localhost:11434");
        assert!(config.embedding.api_key.is_none());

        assert_eq!(config.indexing.chunk_size, 512);
        assert_eq!(config.indexing.overlap, 50);
        assert_eq!(config.indexing.max_file_size, 10 * 1024 * 1024);
        assert_eq!(config.indexing.concurrency, 4);
        assert!(config
            .indexing
            .ignore_patterns
            .contains(&".git".to_string()));

        assert_eq!(config.monitoring.batch_size, 100);
        assert!(!config.monitoring.file_watching);
    }

    #[test]
    fn test_config_from_environment_variables() {
        // Test just the non-conflicting environment variables
        // Skip testing DIRECTORY_INDEXER_QDRANT_COLLECTION to avoid test interference

        // Save original values
        let original_qdrant = env::var("QDRANT_ENDPOINT").ok();
        let original_ollama = env::var("OLLAMA_ENDPOINT").ok();
        let original_qdrant_key = env::var("QDRANT_API_KEY").ok();
        let original_ollama_key = env::var("OLLAMA_API_KEY").ok();

        // Set test values for non-conflicting variables
        env::set_var("QDRANT_ENDPOINT", "http://test-qdrant:6333");
        env::set_var("OLLAMA_ENDPOINT", "http://test-ollama:11434");
        env::set_var("QDRANT_API_KEY", "test-qdrant-key");
        env::set_var("OLLAMA_API_KEY", "test-ollama-key");

        let config = Config::load().expect("Config should load successfully");

        // Test that environment variables override defaults
        assert_eq!(config.storage.qdrant.endpoint, "http://test-qdrant:6333");
        assert_eq!(config.embedding.endpoint, "http://test-ollama:11434");
        assert_eq!(
            config.storage.qdrant.api_key,
            Some("test-qdrant-key".to_string())
        );
        assert_eq!(
            config.embedding.api_key,
            Some("test-ollama-key".to_string())
        );

        // Test that other defaults are preserved
        assert_eq!(config.embedding.provider, "ollama");
        assert_eq!(config.embedding.model, "nomic-embed-text");
        assert_eq!(config.indexing.chunk_size, 512);

        // Restore original values
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
    fn test_test_collection_name_generation() {
        // Test the unique collection name generation logic directly
        let mut config = Config::default();

        // Test case 1: "test" collection name should get unique suffix
        config.storage.qdrant.collection = "test".to_string();
        let original_collection = config.storage.qdrant.collection.clone();

        // Simulate the uniquification logic from Config::load()
        if config.storage.qdrant.collection == "test"
            || config.storage.qdrant.collection == "directory-indexer-test"
        {
            config.storage.qdrant.collection = format!(
                "directory-indexer-test-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            );
        }

        assert_ne!(config.storage.qdrant.collection, original_collection);
        assert!(config
            .storage
            .qdrant
            .collection
            .starts_with("directory-indexer-test-"));
        assert!(config
            .storage
            .qdrant
            .collection
            .contains(&std::process::id().to_string()));

        // Test case 2: "directory-indexer-test" collection name should get unique suffix
        let mut config2 = Config::default();
        config2.storage.qdrant.collection = "directory-indexer-test".to_string();
        let original_collection2 = config2.storage.qdrant.collection.clone();

        if config2.storage.qdrant.collection == "test"
            || config2.storage.qdrant.collection == "directory-indexer-test"
        {
            config2.storage.qdrant.collection = format!(
                "directory-indexer-test-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            );
        }

        assert_ne!(config2.storage.qdrant.collection, original_collection2);
        assert!(config2
            .storage
            .qdrant
            .collection
            .starts_with("directory-indexer-test-"));

        // Test case 3: Other collection names should remain unchanged
        let mut config3 = Config::default();
        config3.storage.qdrant.collection = "my-custom-collection".to_string();
        let original_collection3 = config3.storage.qdrant.collection.clone();

        if config3.storage.qdrant.collection == "test"
            || config3.storage.qdrant.collection == "directory-indexer-test"
        {
            config3.storage.qdrant.collection = format!(
                "directory-indexer-test-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            );
        }

        assert_eq!(config3.storage.qdrant.collection, original_collection3);
        assert_eq!(config3.storage.qdrant.collection, "my-custom-collection");
    }

    #[test]
    fn test_config_save_to_specific_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("test_config.json");

        let config = Config::default();

        // Create parent directories if needed
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create parent directories");
        }

        // Write config directly to our test path
        let json = serde_json::to_string_pretty(&config).expect("Failed to serialize config");
        std::fs::write(&config_path, json).expect("Failed to write config file");

        // Verify file was written
        assert!(config_path.exists());

        let content = fs::read_to_string(&config_path).expect("Failed to read config file");
        assert!(content.contains("directory-indexer-test"));
        assert!(content.contains("ollama"));
        assert!(content.contains("nomic-embed-text"));

        // Verify it can be deserialized back
        let loaded_config: Config =
            serde_json::from_str(&content).expect("Failed to deserialize config");
        assert_eq!(loaded_config.embedding.provider, config.embedding.provider);
        assert_eq!(
            loaded_config.indexing.chunk_size,
            config.indexing.chunk_size
        );
    }

    #[test]
    fn test_ensure_app_dir_exists() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let test_path = temp_dir.path().join("nested").join("dir");

        let mut config = Config::default();
        config.storage.sqlite_path = test_path.join("data.db");

        let result = config.ensure_app_dir_exists();
        assert!(result.is_ok());
        assert!(test_path.exists());
    }

    #[test]
    fn test_default_app_dir_fallback() {
        let original_data_dir = env::var("DIRECTORY_INDEXER_DATA_DIR").ok();

        env::remove_var("DIRECTORY_INDEXER_DATA_DIR");

        let app_dir = Config::default_app_dir();
        // The app dir should be either the home directory + .directory-indexer
        // or the current directory + .directory-indexer as fallback
        assert!(app_dir.ends_with(".directory-indexer"));

        if let Some(val) = original_data_dir {
            env::set_var("DIRECTORY_INDEXER_DATA_DIR", val);
        }
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();

        let json = serde_json::to_string(&config).expect("Failed to serialize config");
        assert!(json.contains("directory-indexer"));

        let deserialized: Config =
            serde_json::from_str(&json).expect("Failed to deserialize config");
        assert_eq!(
            deserialized.storage.qdrant.collection,
            config.storage.qdrant.collection
        );
        assert_eq!(deserialized.embedding.provider, config.embedding.provider);
    }
}
