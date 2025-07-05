use async_trait::async_trait;
use log::{debug, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::provider::{EmbeddingProvider, EmbeddingResponse, EmbeddingUsage};
use crate::error::{IndexerError, Result};

pub struct OllamaProvider {
    client: Client,
    endpoint: String,
    model: String,
}

#[derive(Serialize)]
struct OllamaEmbedRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
struct OllamaEmbedResponse {
    embedding: Vec<f32>,
}

#[derive(Deserialize)]
struct OllamaModelsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Deserialize)]
struct OllamaModel {
    name: String,
}

impl OllamaProvider {
    pub fn new(endpoint: String, model: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60)) // Embedding can take longer
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            endpoint,
            model,
        }
    }
}

#[async_trait]
impl EmbeddingProvider for OllamaProvider {
    fn model_name(&self) -> &str {
        &self.model
    }

    fn embedding_dimension(&self) -> usize {
        // Common dimensions for popular models
        match self.model.as_str() {
            "nomic-embed-text" => 768,
            "mxbai-embed-large" => 1024,
            "all-minilm" => 384,
            _ => {
                let model = &self.model;
                warn!("Unknown model '{model}', assuming 768 dimensions");
                768
            }
        }
    }

    async fn generate_embeddings(&self, texts: Vec<String>) -> Result<EmbeddingResponse> {
        debug!(
            "Generating embeddings for {} texts using model '{}'",
            texts.len(),
            self.model
        );

        let mut embeddings = Vec::new();

        // Ollama API typically handles one text at a time
        for text in &texts {
            let embedding = self.generate_single_embedding(text).await?;
            embeddings.push(embedding);
        }

        Ok(EmbeddingResponse {
            embeddings,
            model: self.model.clone(),
            usage: Some(EmbeddingUsage {
                prompt_tokens: Some(texts.iter().map(|t| t.len() as u32).sum::<u32>() / 4), // Rough token estimate
                total_tokens: Some(texts.iter().map(|t| t.len() as u32).sum::<u32>() / 4),
            }),
        })
    }

    async fn health_check(&self) -> Result<bool> {
        let models_url = format!("{}/api/tags", self.endpoint);

        let response =
            self.client.get(&models_url).send().await.map_err(|e| {
                IndexerError::embedding(format!("Failed to connect to Ollama: {e}"))
            })?;

        if !response.status().is_success() {
            return Ok(false);
        }

        let models_response: OllamaModelsResponse = response.json().await.map_err(|e| {
            IndexerError::embedding(format!("Failed to parse models response: {e}"))
        })?;

        let model_available = models_response
            .models
            .iter()
            .any(|m| m.name.contains(&self.model));

        Ok(model_available)
    }
}

impl OllamaProvider {
    async fn generate_single_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let embed_url = format!("{}/api/embeddings", self.endpoint);

        let request = OllamaEmbedRequest {
            model: self.model.clone(),
            prompt: text.to_string(),
        };

        let response = self
            .client
            .post(&embed_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                IndexerError::embedding(format!("Failed to send embedding request: {e}"))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(IndexerError::embedding(format!(
                "Ollama API returned error: {status}"
            )));
        }

        let embed_response: OllamaEmbedResponse = response.json().await.map_err(|e| {
            IndexerError::embedding(format!("Failed to parse embedding response: {e}"))
        })?;

        debug!(
            "Generated embedding with dimension: {}",
            embed_response.embedding.len()
        );
        Ok(embed_response.embedding)
    }
}
