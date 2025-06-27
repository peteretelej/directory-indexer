use log::{error, info, warn};
use reqwest::Client;
use serde_json::json;

use crate::{
    config::Config,
    error::{IndexerError, Result},
    storage::QdrantStore,
};

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub ollama_available: bool,
    pub qdrant_available: bool,
    pub sqlite_writable: bool,
    pub ollama_models: Vec<String>,
    pub qdrant_collections: Vec<String>,
}

impl HealthStatus {
    pub fn is_ready_for_indexing(&self) -> bool {
        self.ollama_available && self.qdrant_available && self.sqlite_writable
    }

    pub fn is_ready_for_retrieval(&self) -> bool {
        self.qdrant_available && self.sqlite_writable
    }
}

pub async fn check_system_health(config: &Config) -> HealthStatus {
    info!("Checking system health and connectivity...");

    let ollama_status = check_ollama_connectivity(config).await;
    let qdrant_status = check_qdrant_connectivity(config).await;
    let sqlite_status = check_sqlite_connectivity(config).await;

    let health = HealthStatus {
        ollama_available: ollama_status.0,
        qdrant_available: qdrant_status.0,
        sqlite_writable: sqlite_status,
        ollama_models: ollama_status.1,
        qdrant_collections: qdrant_status.1,
    };

    log_health_status(&health);
    health
}

async fn check_ollama_connectivity(config: &Config) -> (bool, Vec<String>) {
    info!("Checking Ollama connectivity at {}", config.embedding.endpoint);

    let client = Client::new();
    let models_url = format!("{}/api/tags", config.embedding.endpoint);

    match client.get(&models_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        let models: Vec<String> = data
                            .get("models")
                            .and_then(|m| m.as_array())
                            .map(|models| {
                                models
                                    .iter()
                                    .filter_map(|model| {
                                        model.get("name").and_then(|n| n.as_str().map(|s| s.to_string()))
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();

                        let model_available = models.iter().any(|m| m.contains(&config.embedding.model));
                        
                        if model_available {
                            info!("✓ Ollama available with model '{}'", config.embedding.model);
                        } else {
                            warn!("⚠ Ollama available but model '{}' not found. Available models: {:?}", 
                                  config.embedding.model, models);
                        }
                        
                        (model_available, models)
                    }
                    Err(e) => {
                        error!("✗ Ollama responded but returned invalid JSON: {}", e);
                        (false, Vec::new())
                    }
                }
            } else {
                error!("✗ Ollama responded with status: {}", response.status());
                (false, Vec::new())
            }
        }
        Err(e) => {
            error!("✗ Cannot connect to Ollama at {}: {}", config.embedding.endpoint, e);
            error!("  Make sure Ollama is running: ollama serve");
            (false, Vec::new())
        }
    }
}

async fn check_qdrant_connectivity(config: &Config) -> (bool, Vec<String>) {
    info!("Checking Qdrant connectivity at {}", config.storage.qdrant.endpoint);

    // Try a simple HTTP request to check if Qdrant is responding
    let client = reqwest::Client::new();
    let collections_url = format!("{}/collections", config.storage.qdrant.endpoint);
    
    match client.get(&collections_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                info!("✓ Qdrant available and responsive");
                // Try to create a store to test full connectivity
                match QdrantStore::new_with_api_key(&config.storage.qdrant.endpoint, "health-check".to_string(), config.storage.qdrant.api_key.clone()).await {
                    Ok(_) => {
                        info!("✓ Qdrant client connection successful");
                        (true, vec![config.storage.qdrant.collection.clone()])
                    }
                    Err(e) => {
                        warn!("⚠ Qdrant HTTP works but client connection failed: {}", e);
                        (false, Vec::new())
                    }
                }
            } else {
                error!("✗ Qdrant responded with status: {}", response.status());
                (false, Vec::new())
            }
        }
        Err(e) => {
            error!("✗ Cannot connect to Qdrant at {}: {}", config.storage.qdrant.endpoint, e);
            error!("  Make sure Qdrant is running: docker run -p 6335:6333 qdrant/qdrant");
            (false, Vec::new())
        }
    }
}

async fn check_sqlite_connectivity(config: &Config) -> bool {
    info!("Checking SQLite database at {:?}", config.storage.sqlite_path);

    // Try to create parent directory if it doesn't exist
    if let Some(parent) = config.storage.sqlite_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            error!("✗ Cannot create SQLite directory {:?}: {}", parent, e);
            return false;
        }
    }

    match crate::storage::SqliteStore::new(&config.storage.sqlite_path) {
        Ok(store) => {
            match store.get_stats() {
                Ok(_) => {
                    info!("✓ SQLite database accessible at {:?}", config.storage.sqlite_path);
                    true
                }
                Err(e) => {
                    error!("✗ SQLite database exists but operations failed: {}", e);
                    false
                }
            }
        }
        Err(e) => {
            error!("✗ Cannot initialize SQLite database at {:?}: {}", config.storage.sqlite_path, e);
            false
        }
    }
}

fn log_health_status(health: &HealthStatus) {
    info!("=== System Health Status ===");
    
    if health.is_ready_for_indexing() {
        info!("🟢 System ready for full indexing and search operations");
    } else if health.is_ready_for_retrieval() {
        warn!("🟡 System ready for content retrieval only (embedding provider unavailable)");
    } else {
        error!("🔴 System not ready - critical components unavailable");
    }

    info!("Components:");
    info!("  Ollama: {}", if health.ollama_available { "✓" } else { "✗" });
    info!("  Qdrant: {}", if health.qdrant_available { "✓" } else { "✗" });
    info!("  SQLite: {}", if health.sqlite_writable { "✓" } else { "✗" });
    
    if !health.ollama_models.is_empty() {
        info!("  Available models: {:?}", health.ollama_models);
    }
    
    info!("=============================");
}

pub async fn test_embedding_generation(config: &Config) -> Result<()> {
    info!("Testing embedding generation with test text...");
    
    let client = Client::new();
    let embed_url = format!("{}/api/embeddings", config.embedding.endpoint);
    
    let request = json!({
        "model": config.embedding.model,
        "prompt": "This is a test sentence for embedding generation."
    });

    let response = client
        .post(&embed_url)
        .json(&request)
        .send()
        .await
        .map_err(|e| IndexerError::embedding(format!("Failed to send embedding request: {}", e)))?;

    if response.status().is_success() {
        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| IndexerError::embedding(format!("Failed to parse embedding response: {}", e)))?;

        if let Some(embedding) = result.get("embedding").and_then(|e| e.as_array()) {
            info!("✓ Successfully generated test embedding (dimension: {})", embedding.len());
            Ok(())
        } else {
            Err(IndexerError::embedding("Embedding response missing 'embedding' field".to_string()))
        }
    } else {
        Err(IndexerError::embedding(format!("Embedding request failed with status: {}", response.status())))
    }
}