use directory_indexer::config::{
    settings::{EmbeddingConfig, IndexingConfig, MonitoringConfig, QdrantConfig, StorageConfig},
    Config,
};
use tempfile::NamedTempFile;

/// Test environment with automatic Qdrant collection cleanup
pub struct TestEnvironment {
    #[allow(dead_code)]
    pub config: Config,
    collection_name: String,
}

impl TestEnvironment {
    /// Create a new test environment with deterministic collection name
    ///
    /// # Arguments
    /// * `test_name` - Name of the test (e.g., "connectivity" for test_connectivity)
    pub async fn new(test_name: &str) -> Self {
        let collection_name = format!("di-test-{}", test_name);

        // Pre-cleanup: delete collection if it exists from previous crashed runs
        let _ = Self::cleanup_collection(&collection_name).await;

        let config = Self::create_test_config(&collection_name);

        Self {
            config,
            collection_name,
        }
    }

    fn create_test_config(collection_name: &str) -> Config {
        let temp_db = NamedTempFile::new().unwrap();

        let mut config = Config {
            storage: StorageConfig {
                sqlite_path: temp_db.path().to_path_buf(),
                qdrant: QdrantConfig {
                    endpoint: "http://localhost:6333".to_string(),
                    collection: collection_name.to_string(),
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
                chunk_size: 256,
                overlap: 50,
                max_file_size: 1024 * 1024,
                ignore_patterns: vec![".git".to_string()],
                concurrency: 1,
            },
            monitoring: MonitoringConfig {
                file_watching: false,
                batch_size: 10,
            },
        };

        // Use environment variable overrides
        if let Ok(qdrant_endpoint) = std::env::var("QDRANT_ENDPOINT") {
            config.storage.qdrant.endpoint = qdrant_endpoint;
        }

        if let Ok(ollama_endpoint) = std::env::var("OLLAMA_ENDPOINT") {
            config.embedding.endpoint = ollama_endpoint;
        }

        config
    }

    async fn cleanup_collection(collection_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let qdrant_endpoint = std::env::var("QDRANT_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:6333".to_string());

        // Try to delete the collection, ignore errors if it doesn't exist or Qdrant is unavailable
        let client = reqwest::Client::new();
        let url = format!("{}/collections/{}", qdrant_endpoint, collection_name);

        let _ = client.delete(&url).send().await;

        Ok(())
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        // Spawn a blocking task for async cleanup in Drop
        let collection_name = self.collection_name.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _ = TestEnvironment::cleanup_collection(&collection_name).await;
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_environment_creates_deterministic_names() {
        let env = TestEnvironment::new("sample-test").await;
        assert_eq!(env.collection_name, "di-test-sample-test");
    }
}
