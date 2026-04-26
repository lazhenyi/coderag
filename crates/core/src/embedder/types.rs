//! Embedder types

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct EmbedderConfig {
    pub api_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub dimension: usize,
    pub batch_size: usize,
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

#[derive(Debug, Clone)]
pub enum EmbedderBackend {
    OpenAI,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    pub vector: Vec<f32>,
    pub model: String,
}

impl Embedding {
    pub fn new(vector: Vec<f32>, model: String) -> Self {
        Self { vector, model }
    }
    pub fn dimension(&self) -> usize {
        self.vector.len()
    }
    pub fn cosine_similarity(&self, other: &Embedding) -> f32 {
        if self.vector.len() != other.vector.len() {
            return 0.0;
        }
        let dot: f32 = self
            .vector
            .iter()
            .zip(other.vector.iter())
            .map(|(a, b)| a * b)
            .sum();
        let na: f32 = self.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        let nb: f32 = other.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if na == 0.0 || nb == 0.0 {
            return 0.0;
        }
        dot / (na * nb)
    }
}
