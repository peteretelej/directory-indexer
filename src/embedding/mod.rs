pub mod ollama;
pub mod openai;
pub mod provider;

pub use provider::{EmbeddingProvider, EmbeddingResponse};
use crate::config::settings::EmbeddingConfig;
use crate::Result;

pub fn create_embedding_provider(config: &EmbeddingConfig) -> Result<Box<dyn EmbeddingProvider>> {
    match config.provider.as_str() {
        "ollama" => {
            let ollama_provider = ollama::OllamaProvider::new(
                config.endpoint.clone(),
                config.model.clone(),
            );
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
        _ => Err(crate::IndexerError::invalid_input(format!(
            "Unsupported embedding provider: {}",
            config.provider
        ))),
    }
}
