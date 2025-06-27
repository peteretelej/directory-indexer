use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::{IndexerError, Result};

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
        Self {
            storage: StorageConfig {
                sqlite_path: dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".directory-indexer")
                    .join("data.db"),
                qdrant: QdrantConfig {
                    endpoint: "http://localhost:6335".to_string(),
                    collection: "directory-indexer".to_string(),
                    api_key: None,
                },
            },
            embedding: EmbeddingConfig {
                provider: "ollama".to_string(),
                model: "nomic-embed-text".to_string(),
                endpoint: "http://localhost:11435".to_string(),
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
        let config_path = Self::default_config_path()?;

        if !config_path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let settings = config::Config::builder()
            .add_source(config::File::from(config_path))
            .build()?;

        let config: Config = settings.try_deserialize()?;
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

    fn default_config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            IndexerError::Config(config::ConfigError::Message(
                "Could not determine home directory".to_string(),
            ))
        })?;

        Ok(home.join(".directory-indexer").join("config.json"))
    }
}
