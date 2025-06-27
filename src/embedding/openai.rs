// TODO: Add async_trait when implementing full functionality
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::provider::EmbeddingProvider;

#[allow(dead_code)]
pub struct OpenAIProvider {
    client: Client,
    endpoint: String,
    model: String,
    api_key: String,
}

#[derive(Serialize)]
#[allow(dead_code)]
struct OpenAIEmbedRequest {
    input: Vec<String>,
    model: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct OpenAIEmbedResponse {
    data: Vec<OpenAIEmbedData>,
    model: String,
    usage: OpenAIUsage,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct OpenAIEmbedData {
    embedding: Vec<f32>,
    index: usize,
    object: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct OpenAIUsage {
    prompt_tokens: u32,
    total_tokens: u32,
}

impl OpenAIProvider {
    pub fn new(endpoint: String, model: String, api_key: String) -> Self {
        Self {
            client: Client::new(),
            endpoint,
            model,
            api_key,
        }
    }
}

impl EmbeddingProvider for OpenAIProvider {
    fn model_name(&self) -> &str {
        &self.model
    }

    fn embedding_dimension(&self) -> usize {
        // text-embedding-ada-002 uses 1536 dimensions
        // text-embedding-3-small uses 1536 dimensions
        // text-embedding-3-large uses 3072 dimensions
        match self.model.as_str() {
            "text-embedding-3-large" => 3072,
            _ => 1536,
        }
    }
}
