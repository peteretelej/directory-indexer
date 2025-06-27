// TODO: Add async_trait when implementing full functionality
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::provider::EmbeddingProvider;

#[allow(dead_code)]
pub struct OllamaProvider {
    client: Client,
    endpoint: String,
    model: String,
}

#[derive(Serialize)]
#[allow(dead_code)]
struct OllamaEmbedRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct OllamaEmbedResponse {
    embedding: Vec<f32>,
}

impl OllamaProvider {
    pub fn new(endpoint: String, model: String) -> Self {
        Self {
            client: Client::new(),
            endpoint,
            model,
        }
    }
}

impl EmbeddingProvider for OllamaProvider {
    fn model_name(&self) -> &str {
        &self.model
    }

    fn embedding_dimension(&self) -> usize {
        // This should be configurable based on the model
        // nomic-embed-text typically uses 768 dimensions
        768
    }
}
