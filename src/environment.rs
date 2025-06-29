use crate::{Config, Result};
use log::{debug, info};

pub async fn validate_environment(config: &Config) -> Result<()> {
    // Test Qdrant connection
    let qdrant_url = format!("{}/collections", config.storage.qdrant.endpoint);
    match reqwest::Client::new().get(&qdrant_url).send().await {
        Ok(response) => {
            if !response.status().is_success() {
                return Err(crate::error::IndexerError::environment_setup(format!(
                    "Qdrant not accessible at {}. Setup required: https://github.com/peteretelej/directory-indexer#setup",
                    config.storage.qdrant.endpoint
                )));
            }
            info!("Connected to Qdrant at {}", config.storage.qdrant.endpoint);
        }
        Err(_) => {
            return Err(crate::error::IndexerError::environment_setup(format!(
                "Cannot connect to Qdrant at {}. Setup required: https://github.com/peteretelej/directory-indexer#setup",
                config.storage.qdrant.endpoint
            )));
        }
    }

    // Test embedding provider
    match config.embedding.provider.as_str() {
        "ollama" => {
            let ollama_url = format!("{}/api/tags", config.embedding.endpoint);
            match reqwest::Client::new().get(&ollama_url).send().await {
                Ok(response) => {
                    if !response.status().is_success() {
                        return Err(crate::error::IndexerError::environment_setup(format!(
                            "Ollama not accessible at {}. Setup required: https://github.com/peteretelej/directory-indexer#setup",
                            config.embedding.endpoint
                        )));
                    }
                    info!("Connected to Ollama at {}", config.embedding.endpoint);
                }
                Err(_) => {
                    return Err(crate::error::IndexerError::environment_setup(format!(
                        "Cannot connect to Ollama at {}. Setup required: https://github.com/peteretelej/directory-indexer#setup",
                        config.embedding.endpoint
                    )));
                }
            }
        }
        "openai" => {
            if config.embedding.api_key.is_none() {
                return Err(crate::error::IndexerError::environment_setup(
                    "OpenAI API key required but not provided (set OPENAI_API_KEY). Setup required: https://github.com/peteretelej/directory-indexer#setup".to_string()
                ));
            }
            info!("OpenAI API key configured for embeddings");
        }
        _ => {
            return Err(crate::error::IndexerError::environment_setup(format!(
                "Unknown embedding provider: {}. Setup required: https://github.com/peteretelej/directory-indexer#setup",
                config.embedding.provider
            )));
        }
    }

    debug!("Environment validation passed");
    Ok(())
}
