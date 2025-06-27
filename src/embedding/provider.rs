// async_trait will be added when we implement the actual embedding logic

// Result import will be added when async methods are implemented

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

// TODO: Implement EmbeddingProvider trait when async_trait is available
pub trait EmbeddingProvider: Send + Sync {
    fn model_name(&self) -> &str;
    fn embedding_dimension(&self) -> usize;
}
