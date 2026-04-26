//! Embedder client implementation

use super::types::{EmbedderConfig, EmbedderBackend, Embedding};
use anyhow::{Context, Result as AnyResult};
use serde::{Deserialize, Serialize};

pub struct EmbedderClient {
    config: EmbedderConfig,
    backend: EmbedderBackend,
    http_client: reqwest::Client,
}

impl EmbedderClient {
    pub fn new(config: EmbedderConfig) -> AnyResult<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build().context("Failed to create HTTP client")?;
        let backend = if config.api_url.contains("openai") || config.api_url.contains("dashscope") || config.api_url.contains("v1/embeddings") {
            EmbedderBackend::OpenAI
        } else { EmbedderBackend::Local };
        Ok(Self { config, backend, http_client })
    }

    pub async fn embed_text(&self, text: &str) -> AnyResult<Embedding> {
        self.embed_batch(&[text.to_string()]).await?.pop().context("No embedding returned")
    }

    pub async fn embed_batch(&self, texts: &[String]) -> AnyResult<Vec<Embedding>> {
        match self.backend {
            EmbedderBackend::OpenAI => self.embed_openai(texts).await,
            EmbedderBackend::Local => self.embed_local(texts).await,
        }
    }

    async fn embed_openai(&self, texts: &[String]) -> AnyResult<Vec<Embedding>> {
        if self.config.api_key.is_none() {
            tracing::warn!("No API key provided, using fallback embeddings");
            return Ok(texts.iter().map(|t| Embedding::new(self.deterministic_vector(t), format!("{}-fallback", self.config.model))).collect());
        }
        #[derive(Serialize)] struct OpenAIRequest { input: Vec<String>, model: String }
        #[derive(Deserialize)] struct OpenAIResponse { data: Vec<OpenAIData> }
        #[derive(Deserialize)] struct OpenAIData { embedding: Vec<f32> }

        let request = OpenAIRequest { input: texts.to_vec(), model: self.config.model.clone() };
        let mut req_builder = self.http_client.post(&self.config.api_url).json(&request);
        if let Some(ref key) = self.config.api_key { req_builder = req_builder.header("Authorization", format!("Bearer {}", key)); }

        let response = match req_builder.send().await {
            Ok(r) => r,
            Err(e) => { tracing::warn!("API request failed ({}), using fallback", e); return self.fallback_all(texts); }
        };
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::warn!("API error: {} - {}, using fallback", status, body);
            return self.fallback_all(texts);
        }
        let result: OpenAIResponse = match response.json().await {
            Ok(r) => r,
            Err(e) => { tracing::warn!("API parse failed ({}), using fallback", e); return self.fallback_all(texts); }
        };
        Ok(result.data.into_iter().map(|d| Embedding::new(d.embedding, self.config.model.clone())).collect())
    }

    async fn embed_local(&self, texts: &[String]) -> AnyResult<Vec<Embedding>> {
        #[derive(Serialize)] struct LocalRequest { prompt: String }
        #[derive(Deserialize)] struct LocalResponse { embedding: Option<Vec<f32>> }
        let mut embeddings = Vec::with_capacity(texts.len());
        for text in texts {
            let request = LocalRequest { prompt: text.clone() };
            let response = match self.http_client.post(&self.config.api_url).json(&request).send().await {
                Ok(r) => r, Err(e) => { tracing::warn!("Local request failed ({}), using fallback", e); embeddings.push(Embedding::new(self.deterministic_vector(text), format!("{}-fallback", self.config.model))); continue; }
            };
            let result: LocalResponse = match response.json().await {
                Ok(r) => r, Err(e) => { tracing::warn!("Local parse failed ({}), using fallback", e); embeddings.push(Embedding::new(self.deterministic_vector(text), format!("{}-fallback", self.config.model))); continue; }
            };
            let vector = result.embedding.unwrap_or_else(|| self.deterministic_vector(text));
            embeddings.push(Embedding::new(vector, self.config.model.clone()));
        }
        Ok(embeddings)
    }

    fn fallback_all(&self, texts: &[String]) -> AnyResult<Vec<Embedding>> {
        Ok(texts.iter().map(|t| Embedding::new(self.deterministic_vector(t), format!("{}-fallback", self.config.model))).collect())
    }

    fn simple_hash(&self, text: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new(); text.hash(&mut hasher); hasher.finish()
    }

    fn deterministic_vector(&self, text: &str) -> Vec<f32> {
        let mut vector = vec![0.0f32; self.config.dimension];
        let hash = self.simple_hash(text);
        for (i, v) in vector.iter_mut().enumerate() { *v = ((hash >> i) & 0xFF) as f32 / 255.0; }
        vector
    }
}
