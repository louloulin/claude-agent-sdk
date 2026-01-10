// Embedding provider implementations
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::storage::{StorageError, StorageResult};

/// Embedding configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Provider type
    pub provider: EmbeddingProvider,

    /// API key (for remote providers)
    pub api_key: Option<String>,

    /// Model name
    pub model: Option<String>,

    /// Embedding dimension
    pub dimension: usize,

    /// Batch size for batch embeddings
    pub batch_size: usize,

    /// API endpoint (custom)
    pub endpoint: Option<String>,

    /// Timeout in seconds
    pub timeout_secs: u64,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: EmbeddingProvider::OpenAI,
            api_key: None,
            model: Some("text-embedding-3-small".to_string()),
            dimension: 1536,
            batch_size: 100,
            endpoint: None,
            timeout_secs: 30,
        }
    }
}

/// Embedding provider types
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmbeddingProvider {
    OpenAI,
    Local,
    Cohere,
    HuggingFace,
    Custom(String),
}

/// Embedding request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub text: String,
}

/// Embedding response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub embedding: Vec<f32>,
    pub model: String,
    pub usage: Option<EmbeddingUsage>,
}

/// Token usage information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbeddingUsage {
    pub prompt_tokens: u32,
    pub total_tokens: u32,
}

/// Async embedder trait following Claude Agent SDK async patterns
#[async_trait]
pub trait Embedder: Send + Sync {
    /// Generate embedding for a single text
    async fn embed(&self, text: &str) -> StorageResult<Vec<f32>>;

    /// Generate embeddings for multiple texts (batch)
    async fn embed_batch(&self, texts: &[&str]) -> StorageResult<Vec<Vec<f32>>>;

    /// Get the embedding dimension
    fn dimension(&self) -> usize;

    /// Get the provider type
    fn provider(&self) -> EmbeddingProvider;
}

/// OpenAI embedding provider
pub struct OpenAIEmbedder {
    client: Arc<Client>,
    config: EmbeddingConfig,
    api_key: String,
    model: String,
}

impl OpenAIEmbedder {
    /// Create a new OpenAI embedder
    pub fn new(config: EmbeddingConfig) -> StorageResult<Self> {
        let api_key = config.api_key
            .clone()
            .ok_or_else(|| StorageError::InvalidConfiguration(
                "OpenAI API key is required".to_string()
            ))?;

        let model = config.model.clone()
            .unwrap_or_else(|| "text-embedding-3-small".to_string());

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| StorageError::Internal(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client: Arc::new(client),
            config,
            api_key,
            model,
        })
    }

    /// Create from environment variable
    pub fn from_env() -> StorageResult<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| StorageError::InvalidConfiguration(
                "OPENAI_API_KEY environment variable not set".to_string()
            ))?;

        Self::new(EmbeddingConfig {
            api_key: Some(api_key),
            ..Default::default()
        })
    }
}

#[async_trait]
impl Embedder for OpenAIEmbedder {
    async fn embed(&self, text: &str) -> StorageResult<Vec<f32>> {
        let request = OpenAIRequest {
            input: text.to_string(),
            model: self.model.clone(),
        };

        let response = self.client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| StorageError::EmbeddingGeneration(format!("API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(StorageError::EmbeddingGeneration(
                format!("OpenAI API error {}: {}", status, error_text)
            ));
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| StorageError::EmbeddingGeneration(format!("Failed to parse response: {}", e)))?;

        let embedding = openai_response.data
            .first()
            .ok_or_else(|| StorageError::EmbeddingGeneration("No embedding returned".to_string()))?
            .embedding
            .clone();

        Ok(embedding)
    }

    async fn embed_batch(&self, texts: &[&str]) -> StorageResult<Vec<Vec<f32>>> {
        // Process in batches to respect API limits
        let batch_size = self.config.batch_size.min(texts.len());
        let mut results = Vec::with_capacity(texts.len());

        for chunk in texts.chunks(batch_size) {
            let request = OpenAIRequestBatch {
                input: chunk.iter().map(|s| s.to_string()).collect(),
                model: self.model.clone(),
            };

            let response = self.client
                .post("https://api.openai.com/v1/embeddings")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await
                .map_err(|e| StorageError::EmbeddingGeneration(format!("Batch API request failed: {}", e)))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                return Err(StorageError::EmbeddingGeneration(
                    format!("OpenAI batch API error {}: {}", status, error_text)
                ));
            }

            let openai_response: OpenAIResponseBatch = response
                .json()
                .await
                .map_err(|e| StorageError::EmbeddingGeneration(format!("Failed to parse batch response: {}", e)))?;

            for item in openai_response.data {
                results.push(item.embedding);
            }
        }

        Ok(results)
    }

    fn dimension(&self) -> usize {
        self.config.dimension
    }

    fn provider(&self) -> EmbeddingProvider {
        EmbeddingProvider::OpenAI
    }
}

/// Simple local embedder (placeholder for future local model integration)
pub struct LocalEmbedder {
    dimension: usize,
}

impl LocalEmbedder {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait]
impl Embedder for LocalEmbedder {
    async fn embed(&self, text: &str) -> StorageResult<Vec<f32>> {
        // Placeholder: simple hash-based embedding for testing
        // In production, this would use a local model like sentence-transformers
        let mut embedding = vec![0.0f32; self.dimension];
        let bytes = text.as_bytes();

        for (i, &byte) in bytes.iter().enumerate() {
            let idx = (i * 3) % self.dimension;
            embedding[idx] += (byte as f32) / 255.0;
        }

        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in embedding.iter_mut() {
                *v /= norm;
            }
        }

        Ok(embedding)
    }

    async fn embed_batch(&self, texts: &[&str]) -> StorageResult<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn provider(&self) -> EmbeddingProvider {
        EmbeddingProvider::Local
    }
}

// OpenAI API types

#[derive(Clone, Debug, Serialize)]
struct OpenAIRequest {
    input: String,
    model: String,
}

#[derive(Clone, Debug, Serialize)]
struct OpenAIRequestBatch {
    input: Vec<String>,
    model: String,
}

#[derive(Clone, Debug, Deserialize)]
struct OpenAIResponse {
    data: Vec<OpenAIEmbedding>,
}

#[derive(Clone, Debug, Deserialize)]
struct OpenAIResponseBatch {
    data: Vec<OpenAIEmbedding>,
}

#[derive(Clone, Debug, Deserialize)]
struct OpenAIEmbedding {
    embedding: Vec<f32>,
    index: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_embedder() {
        let embedder = LocalEmbedder::new(384);
        let embedding = embedder.embed("Hello, world!").await.unwrap();

        assert_eq!(embedding.len(), 384);

        // Check normalization
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_local_embedder_batch() {
        let embedder = LocalEmbedder::new(384);
        let texts = vec!["Hello", "world", "test"];
        let embeddings = embedder.embed_batch(&texts).await.unwrap();

        assert_eq!(embeddings.len(), 3);
        for emb in &embeddings {
            assert_eq!(emb.len(), 384);
        }
    }

    #[test]
    fn test_embedding_config_default() {
        let config = EmbeddingConfig::default();
        assert_eq!(config.provider, EmbeddingProvider::OpenAI);
        assert_eq!(config.dimension, 1536);
        assert_eq!(config.batch_size, 100);
    }
}
