use crate::error::Result;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct EmbeddingResponse {
    pub embeddings: Vec<Vec<f32>>,
    pub model: String,
    pub usage: Option<EmbeddingUsage>,
}

#[derive(Debug, Clone)]
pub struct EmbeddingUsage {
    pub prompt_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    fn model_name(&self) -> &str;
    fn embedding_dimension(&self) -> usize;

    async fn generate_embeddings(&self, texts: Vec<String>) -> Result<EmbeddingResponse>;
    async fn generate_embedding(&self, text: String) -> Result<Vec<f32>> {
        let response = self.generate_embeddings(vec![text]).await?;
        response.embeddings.into_iter().next().ok_or_else(|| {
            crate::error::IndexerError::embedding("No embedding returned".to_string())
        })
    }

    async fn health_check(&self) -> Result<bool>;
}
