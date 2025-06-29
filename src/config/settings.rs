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
                    collection: "directory-indexer".to_string(),
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

        if let Ok(qdrant_collection) = std::env::var("DIRECTORY_INDEXER_QDRANT_COLLECTION") {
            // If collection name is "test", make it unique per process for test isolation
            if qdrant_collection == "test" {
                config.storage.qdrant.collection = format!(
                    "directory-indexer-test-{}-{}",
                    std::process::id(),
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
                );
            } else {
                config.storage.qdrant.collection = qdrant_collection;
            }
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
