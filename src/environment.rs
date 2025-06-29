use crate::{Config, Result};
use log::debug;

pub async fn validate_environment(config: &Config) -> Result<()> {
    // Test Qdrant connection
    let qdrant_url = format!("{}/collections", config.storage.qdrant.endpoint);
    match reqwest::Client::new().get(&qdrant_url).send().await {
        Ok(response) => {
            if !response.status().is_success() {
                return Err(crate::error::IndexerError::environment_setup(format!(
                    "Qdrant not accessible at {}\n\nSetup required: https://github.com/peteretelej/directory-indexer#requirements",
                    config.storage.qdrant.endpoint
                )));
            }
        }
        Err(_) => {
            return Err(crate::error::IndexerError::environment_setup(format!(
                "Cannot connect to Qdrant at {}\n\nSetup required: https://github.com/peteretelej/directory-indexer#requirements",
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
                            "Ollama not accessible at {}\n\nSetup required: https://github.com/peteretelej/directory-indexer#requirements",
                            config.embedding.endpoint
                        )));
                    }
                }
                Err(_) => {
                    return Err(crate::error::IndexerError::environment_setup(format!(
                        "Cannot connect to Ollama at {}\n\nSetup required: https://github.com/peteretelej/directory-indexer#requirements",
                        config.embedding.endpoint
                    )));
                }
            }
        }
        "openai" => {
            if config.embedding.api_key.is_none() {
                return Err(crate::error::IndexerError::environment_setup(
                    "OpenAI API key required but not provided (set OPENAI_API_KEY)\n\nSetup required: https://github.com/peteretelej/directory-indexer#requirements".to_string()
                ));
            }
        }
        _ => {
            return Err(crate::error::IndexerError::environment_setup(format!(
                "Unknown embedding provider: {}\n\nSetup required: https://github.com/peteretelej/directory-indexer#requirements",
                config.embedding.provider
            )));
        }
    }

    debug!("Environment validation passed");
    Ok(())
}
