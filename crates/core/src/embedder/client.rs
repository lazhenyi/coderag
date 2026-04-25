//! Embedder Client
//!
//! Client for interacting with embedding services (OpenAI, local models, etc.).

use anyhow::{Context, Result as AnyResult};
use serde::{Deserialize, Serialize};

/// Configuration for embedder
#[derive(Debug, Clone)]
pub struct EmbedderConfig {
    /// API endpoint URL
    pub api_url: String,
    /// API key (if required)
    pub api_key: Option<String>,
    /// Model name
    pub model: String,
    /// Embedding dimension
    pub dimension: usize,
    /// Batch size for requests
    pub batch_size: usize,
    /// Timeout in seconds
    pub timeout_secs: u64,
}

impl Default for EmbedderConfig {
    fn default() -> Self {
        Self {
            api_url: "https://api.openai.com/v1/embeddings".to_string(),
            api_key: None,
            model: "text-embedding-ada-002".to_string(),
            dimension: 1536,
            batch_size: 100,
            timeout_secs: 60,
        }
    }
}

/// Embedder with local model support
#[derive(Debug, Clone)]
pub enum EmbedderBackend {
    OpenAI,
    Local,
}

/// Embedding vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    /// The embedding vector
    pub vector: Vec<f32>,
    /// Model used to generate the embedding
    pub model: String,
}

impl Embedding {
    /// Create a new embedding
    pub fn new(vector: Vec<f32>, model: String) -> Self {
        Self { vector, model }
    }

    /// Get the dimension of the embedding
    pub fn dimension(&self) -> usize {
        self.vector.len()
    }

    /// Compute cosine similarity with another embedding
    pub fn cosine_similarity(&self, other: &Embedding) -> f32 {
        if self.vector.len() != other.vector.len() {
            return 0.0;
        }

        let dot_product: f32 = self.vector
            .iter()
            .zip(other.vector.iter())
            .map(|(a, b)| a * b)
            .sum();

        let norm_a: f32 = self.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = other.vector.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }
}

/// Embedder client for making API calls
pub struct EmbedderClient {
    config: EmbedderConfig,
    backend: EmbedderBackend,
    http_client: reqwest::Client,
}

impl EmbedderClient {
    /// Create a new embedder client
    pub fn new(config: EmbedderConfig) -> AnyResult<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .context("Failed to create HTTP client")?;

        let backend = if config.api_url.contains("openai") {
            EmbedderBackend::OpenAI
        } else {
            EmbedderBackend::Local
        };

        Ok(Self {
            config,
            backend,
            http_client,
        })
    }

    /// Generate embedding for a single text
    pub async fn embed_text(&self, text: &str) -> AnyResult<Embedding> {
        self.embed_batch(&[text.to_string()]).await?.pop()
            .context("No embedding returned")
    }

    /// Generate embeddings for a batch of texts
    pub async fn embed_batch(&self, texts: &[String]) -> AnyResult<Vec<Embedding>> {
        match self.backend {
            EmbedderBackend::OpenAI => self.embed_openai(texts).await,
            EmbedderBackend::Local => self.embed_local(texts).await,
        }
    }

    /// Embed using OpenAI API
    async fn embed_openai(&self, texts: &[String]) -> AnyResult<Vec<Embedding>> {
        #[derive(Serialize)]
        struct OpenAIRequest {
            input: Vec<String>,
            model: String,
        }

        #[derive(Deserialize)]
        struct OpenAIResponse {
            data: Vec<OpenAIData>,
        }

        #[derive(Deserialize)]
        struct OpenAIData {
            embedding: Vec<f32>,
        }

        let request = OpenAIRequest {
            input: texts.to_vec(),
            model: self.config.model.clone(),
        };

        let mut req_builder = self.http_client
            .post(&self.config.api_url)
            .json(&request);

        if let Some(ref api_key) = self.config.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req_builder.send().await
            .context("Failed to send request to OpenAI")?;

        let result: OpenAIResponse = response.json().await
            .context("Failed to parse OpenAI response")?;

        Ok(result.data.into_iter()
            .map(|d| Embedding::new(d.embedding, self.config.model.clone()))
            .collect())
    }

    /// Embed using a local model (Ollama, etc.)
    async fn embed_local(&self, texts: &[String]) -> AnyResult<Vec<Embedding>> {
        #[derive(Serialize)]
        struct LocalRequest {
            prompt: String,
        }

        #[derive(Deserialize)]
        struct LocalResponse {
            embedding: Option<Vec<f32>>,
        }

        let mut embeddings = Vec::with_capacity(texts.len());

        for text in texts {
            let request = LocalRequest {
                prompt: text.clone(),
            };

            let response = self.http_client
                .post(&self.config.api_url)
                .json(&request)
                .send()
                .await
                .context("Failed to send request to local embedder")?;

            let result: LocalResponse = response.json().await
                .context("Failed to parse local embedder response")?;

            let vector = result.embedding.unwrap_or_else(|| {
                // Fallback: generate a dummy embedding for testing
                vec![0.0; self.config.dimension]
            });

            embeddings.push(Embedding::new(vector, self.config.model.clone()));
        }

        Ok(embeddings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_creation() {
        let embedding = Embedding::new(vec![0.1, 0.2, 0.3], "test".to_string());
        assert_eq!(embedding.dimension(), 3);
        assert_eq!(embedding.model, "test");
    }

    #[test]
    fn test_cosine_similarity() {
        let e1 = Embedding::new(vec![1.0, 0.0, 0.0], "test".to_string());
        let e2 = Embedding::new(vec![1.0, 0.0, 0.0], "test".to_string());
        let e3 = Embedding::new(vec![0.0, 1.0, 0.0], "test".to_string());

        assert!((e1.cosine_similarity(&e2) - 1.0).abs() < 0.001);
        assert!((e1.cosine_similarity(&e3) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_default_config() {
        let config = EmbedderConfig::default();
        assert_eq!(config.model, "text-embedding-ada-002");
        assert_eq!(config.dimension, 1536);
    }
}
