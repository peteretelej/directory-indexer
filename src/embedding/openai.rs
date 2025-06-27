use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::provider::{EmbeddingProvider, EmbeddingResponse, EmbeddingUsage};
use crate::error::{IndexerError, Result};

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

#[async_trait]
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

    async fn generate_embeddings(&self, texts: Vec<String>) -> Result<EmbeddingResponse> {
        let request = OpenAIEmbedRequest {
            input: texts,
            model: self.model.clone(),
        };

        let response = self.client
            .post(&format!("{}/v1/embeddings", self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| IndexerError::embedding(format!("Failed to send OpenAI request: {}", e)))?;

        if !response.status().is_success() {
            return Err(IndexerError::embedding(format!(
                "OpenAI API returned error: {}",
                response.status()
            )));
        }

        let openai_response: OpenAIEmbedResponse = response
            .json()
            .await
            .map_err(|e| IndexerError::embedding(format!("Failed to parse OpenAI response: {}", e)))?;

        let embeddings = openai_response.data
            .into_iter()
            .map(|data| data.embedding)
            .collect();

        Ok(EmbeddingResponse {
            embeddings,
            model: openai_response.model,
            usage: Some(EmbeddingUsage {
                prompt_tokens: Some(openai_response.usage.prompt_tokens),
                total_tokens: Some(openai_response.usage.total_tokens),
            }),
        })
    }

    async fn health_check(&self) -> Result<bool> {
        // Try a simple request to check if the API key and endpoint work
        let test_request = OpenAIEmbedRequest {
            input: vec!["test".to_string()],
            model: self.model.clone(),
        };

        let response = self.client
            .post(&format!("{}/v1/embeddings", self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&test_request)
            .send()
            .await;

        match response {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}
