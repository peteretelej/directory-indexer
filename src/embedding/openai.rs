use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::provider::{EmbeddingProvider, EmbeddingResponse, EmbeddingUsage};
use crate::error::{IndexerError, Result};

pub struct OpenAIProvider {
    client: Client,
    endpoint: String,
    model: String,
    api_key: String,
}

#[derive(Serialize)]
struct OpenAIEmbedRequest {
    input: Vec<String>,
    model: String,
}

#[derive(Deserialize)]
struct OpenAIEmbedResponse {
    data: Vec<OpenAIEmbedData>,
    model: String,
    usage: OpenAIUsage,
}

#[derive(Deserialize)]
struct OpenAIEmbedData {
    embedding: Vec<f32>,
    #[allow(dead_code)]
    index: usize,
    #[allow(dead_code)]
    object: String,
}

#[derive(Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    total_tokens: u32,
}

impl OpenAIProvider {
    pub fn new(endpoint: String, model: String, api_key: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60)) // Embedding can take longer
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
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

        let response = self
            .client
            .post(format!("{}/v1/embeddings", self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| IndexerError::embedding(format!("Failed to send OpenAI request: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(IndexerError::embedding(format!(
                "OpenAI API returned error: {status}"
            )));
        }

        let openai_response: OpenAIEmbedResponse = response.json().await.map_err(|e| {
            IndexerError::embedding(format!("Failed to parse OpenAI response: {e}"))
        })?;

        let embeddings = openai_response
            .data
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

        let response = self
            .client
            .post(format!("{}/v1/embeddings", self.endpoint))
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

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_new_provider() {
        let provider = OpenAIProvider::new(
            "https://api.openai.com".to_string(),
            "text-embedding-3-small".to_string(),
            "test-key".to_string(),
        );

        assert_eq!(provider.model_name(), "text-embedding-3-small");
        assert_eq!(provider.embedding_dimension(), 1536);
    }

    #[tokio::test]
    async fn test_embedding_dimensions() {
        let small_provider = OpenAIProvider::new(
            "https://api.openai.com".to_string(),
            "text-embedding-3-small".to_string(),
            "test-key".to_string(),
        );
        assert_eq!(small_provider.embedding_dimension(), 1536);

        let large_provider = OpenAIProvider::new(
            "https://api.openai.com".to_string(),
            "text-embedding-3-large".to_string(),
            "test-key".to_string(),
        );
        assert_eq!(large_provider.embedding_dimension(), 3072);

        let ada_provider = OpenAIProvider::new(
            "https://api.openai.com".to_string(),
            "text-embedding-ada-002".to_string(),
            "test-key".to_string(),
        );
        assert_eq!(ada_provider.embedding_dimension(), 1536);
    }

    #[tokio::test]
    async fn test_generate_embeddings_success() {
        let mock_server = MockServer::start().await;

        let response_body = r#"{
            "object": "list",
            "data": [
                {
                    "object": "embedding",
                    "embedding": [0.1, 0.2, 0.3],
                    "index": 0
                },
                {
                    "object": "embedding", 
                    "embedding": [0.4, 0.5, 0.6],
                    "index": 1
                }
            ],
            "model": "text-embedding-3-small",
            "usage": {
                "prompt_tokens": 10,
                "total_tokens": 10
            }
        }"#;

        Mock::given(method("POST"))
            .and(path("/v1/embeddings"))
            .and(header("authorization", "Bearer test-key"))
            .and(header("content-type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            mock_server.uri(),
            "text-embedding-3-small".to_string(),
            "test-key".to_string(),
        );

        let result = provider
            .generate_embeddings(vec!["hello".to_string(), "world".to_string()])
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.embeddings.len(), 2);
        assert_eq!(response.embeddings[0], vec![0.1, 0.2, 0.3]);
        assert_eq!(response.embeddings[1], vec![0.4, 0.5, 0.6]);
        assert_eq!(response.model, "text-embedding-3-small");
        assert!(response.usage.is_some());
        assert_eq!(response.usage.unwrap().total_tokens, Some(10));
    }

    #[tokio::test]
    async fn test_generate_embeddings_single_text() {
        let mock_server = MockServer::start().await;

        let response_body = r#"{
            "object": "list",
            "data": [
                {
                    "object": "embedding",
                    "embedding": [0.1, 0.2, 0.3],
                    "index": 0
                }
            ],
            "model": "text-embedding-3-small",
            "usage": {
                "prompt_tokens": 5,
                "total_tokens": 5
            }
        }"#;

        Mock::given(method("POST"))
            .and(path("/v1/embeddings"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            mock_server.uri(),
            "text-embedding-3-small".to_string(),
            "test-key".to_string(),
        );

        let result = provider
            .generate_embeddings(vec!["hello world".to_string()])
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.embeddings.len(), 1);
        assert_eq!(response.embeddings[0], vec![0.1, 0.2, 0.3]);
    }

    #[tokio::test]
    async fn test_generate_embeddings_api_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/embeddings"))
            .respond_with(
                ResponseTemplate::new(401)
                    .set_body_string(r#"{"error": {"message": "Invalid API key"}}"#),
            )
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            mock_server.uri(),
            "text-embedding-3-small".to_string(),
            "invalid-key".to_string(),
        );

        let result = provider.generate_embeddings(vec!["test".to_string()]).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("OpenAI API returned error"));
    }

    #[tokio::test]
    async fn test_generate_embeddings_invalid_json() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/embeddings"))
            .respond_with(ResponseTemplate::new(200).set_body_string("invalid json"))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            mock_server.uri(),
            "text-embedding-3-small".to_string(),
            "test-key".to_string(),
        );

        let result = provider.generate_embeddings(vec!["test".to_string()]).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error
            .to_string()
            .contains("Failed to parse OpenAI response"));
    }

    #[tokio::test]
    async fn test_health_check_success() {
        let mock_server = MockServer::start().await;

        let response_body = r#"{
            "object": "list",
            "data": [
                {
                    "object": "embedding",
                    "embedding": [0.1, 0.2, 0.3],
                    "index": 0
                }
            ],
            "model": "text-embedding-3-small",
            "usage": {
                "prompt_tokens": 1,
                "total_tokens": 1
            }
        }"#;

        Mock::given(method("POST"))
            .and(path("/v1/embeddings"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            mock_server.uri(),
            "text-embedding-3-small".to_string(),
            "test-key".to_string(),
        );

        let result = provider.health_check().await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_health_check_failure() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/embeddings"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            mock_server.uri(),
            "text-embedding-3-small".to_string(),
            "invalid-key".to_string(),
        );

        let result = provider.health_check().await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_health_check_network_error() {
        // Use invalid URL to simulate network error
        let provider = OpenAIProvider::new(
            "http://invalid-url-that-does-not-exist:9999".to_string(),
            "text-embedding-3-small".to_string(),
            "test-key".to_string(),
        );

        let result = provider.health_check().await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_request_headers_and_body() {
        let mock_server = MockServer::start().await;

        let response_body = r#"{
            "object": "list",
            "data": [
                {
                    "object": "embedding",
                    "embedding": [0.1, 0.2, 0.3],
                    "index": 0
                }
            ],
            "model": "text-embedding-ada-002",
            "usage": {
                "prompt_tokens": 8,
                "total_tokens": 8
            }
        }"#;

        // Verify correct headers and request body
        Mock::given(method("POST"))
            .and(path("/v1/embeddings"))
            .and(header("authorization", "Bearer secret-key"))
            .and(header("content-type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            mock_server.uri(),
            "text-embedding-ada-002".to_string(),
            "secret-key".to_string(),
        );

        let result = provider
            .generate_embeddings(vec!["The food was delicious".to_string()])
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.model, "text-embedding-ada-002");
    }

    #[tokio::test]
    async fn test_empty_embeddings_list() {
        let mock_server = MockServer::start().await;

        let response_body = r#"{
            "object": "list",
            "data": [],
            "model": "text-embedding-3-small",
            "usage": {
                "prompt_tokens": 0,
                "total_tokens": 0
            }
        }"#;

        Mock::given(method("POST"))
            .and(path("/v1/embeddings"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            mock_server.uri(),
            "text-embedding-3-small".to_string(),
            "test-key".to_string(),
        );

        let result = provider.generate_embeddings(vec!["test".to_string()]).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.embeddings.len(), 0);
        assert_eq!(response.model, "text-embedding-3-small");
    }
}
