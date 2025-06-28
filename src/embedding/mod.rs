pub mod ollama;
pub mod openai;
pub mod provider;

#[cfg(test)]
pub mod mock;

use crate::config::settings::EmbeddingConfig;
use crate::Result;
pub use provider::{EmbeddingProvider, EmbeddingResponse};

pub fn create_embedding_provider(config: &EmbeddingConfig) -> Result<Box<dyn EmbeddingProvider>> {
    match config.provider.as_str() {
        "ollama" => {
            let ollama_provider =
                ollama::OllamaProvider::new(config.endpoint.clone(), config.model.clone());
            Ok(Box::new(ollama_provider))
        }
        "openai" => {
            let openai_provider = openai::OpenAIProvider::new(
                config.endpoint.clone(),
                config.model.clone(),
                config.api_key.as_ref().unwrap_or(&String::new()).clone(),
            );
            Ok(Box::new(openai_provider))
        }
        #[cfg(test)]
        "mock" => {
            let mock_provider = mock::MockEmbeddingProvider::new(384); // Standard embedding dimension
            Ok(Box::new(mock_provider))
        }
        _ => Err(crate::IndexerError::invalid_input(format!(
            "Unsupported embedding provider: {}",
            config.provider
        ))),
    }
}
