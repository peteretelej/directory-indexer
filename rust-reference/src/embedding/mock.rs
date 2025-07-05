use crate::embedding::provider::{EmbeddingProvider, EmbeddingResponse, EmbeddingUsage};
use crate::Result;
use async_trait::async_trait;

/// Mock embedding provider for testing purposes
/// Returns deterministic fake embeddings without making network calls
pub struct MockEmbeddingProvider {
    dimension: usize,
}

impl MockEmbeddingProvider {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }

    /// Create a deterministic embedding based on text content
    fn create_fake_embedding(&self, text: &str) -> Vec<f32> {
        let mut embedding = vec![0.0; self.dimension];

        // Create a simple deterministic pattern based on text content
        let text_bytes = text.as_bytes();
        let text_hash = text_bytes.iter().map(|&b| b as f32).sum::<f32>();

        for i in 0..self.dimension {
            // Use both text hash and character at position to create variation
            let char_influence = if i < text_bytes.len() {
                text_bytes[i] as f32
            } else {
                text_hash / (i + 1) as f32
            };
            embedding[i] =
                (text_hash + char_influence + i as f32) / (self.dimension as f32 * 100.0);
        }

        // Normalize to unit vector (common for embeddings)
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }

        embedding
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbeddingProvider {
    fn model_name(&self) -> &str {
        "mock-embedding-model"
    }

    fn embedding_dimension(&self) -> usize {
        self.dimension
    }

    async fn generate_embeddings(&self, texts: Vec<String>) -> Result<EmbeddingResponse> {
        // Simulate minimal processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(texts.len() as u64)).await;

        let embeddings: Vec<Vec<f32>> = texts
            .iter()
            .map(|text| self.create_fake_embedding(text))
            .collect();

        Ok(EmbeddingResponse {
            embeddings,
            model: "mock-embedding-model".to_string(),
            usage: Some(EmbeddingUsage {
                prompt_tokens: Some(texts.iter().map(|t| t.len()).sum::<usize>() as u32),
                total_tokens: Some(texts.iter().map(|t| t.len()).sum::<usize>() as u32),
            }),
        })
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_embedding_deterministic() {
        let provider = MockEmbeddingProvider::new(384);

        let text = "test text";
        let embedding1 = provider.generate_embedding(text.to_string()).await.unwrap();
        let embedding2 = provider.generate_embedding(text.to_string()).await.unwrap();

        assert_eq!(embedding1, embedding2);
        assert_eq!(embedding1.len(), 384);
    }

    #[tokio::test]
    async fn test_mock_embedding_different_texts() {
        let provider = MockEmbeddingProvider::new(384);

        let embedding1 = provider
            .generate_embedding("text one".to_string())
            .await
            .unwrap();
        let embedding2 = provider
            .generate_embedding("text two".to_string())
            .await
            .unwrap();

        assert_ne!(embedding1, embedding2);
        assert_eq!(embedding1.len(), 384);
        assert_eq!(embedding2.len(), 384);
    }

    #[tokio::test]
    async fn test_batch_embeddings() {
        let provider = MockEmbeddingProvider::new(384);

        let texts = vec!["one".to_string(), "two".to_string(), "three".to_string()];
        let response = provider.generate_embeddings(texts).await.unwrap();

        assert_eq!(response.embeddings.len(), 3);
        assert_eq!(response.model, "mock-embedding-model");
        assert!(response.usage.unwrap().total_tokens.unwrap() > 0);
    }
}
