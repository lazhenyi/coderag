//! Embedder Module
//!
//! Generates embeddings for code chunks using external embedding services.

mod client;

pub use client::{EmbedderClient, EmbedderConfig, Embedding};

use crate::chunker::{Chunk, chunk_to_embedding_text};
use anyhow::{Context, Result as AnyResult};
use std::collections::HashMap;

/// Embedder for generating code embeddings
pub struct Embedder {
    client: EmbedderClient,
    batch_size: usize,
}

impl Embedder {
    /// Create a new embedder
    pub fn new(config: EmbedderConfig) -> AnyResult<Self> {
        let client = EmbedderClient::new(config)?;
        Ok(Self {
            client,
            batch_size: 100,
        })
    }

    /// Create a new embedder with custom batch size
    pub fn with_batch_size(config: EmbedderConfig, batch_size: usize) -> AnyResult<Self> {
        let client = EmbedderClient::new(config)?;
        Ok(Self {
            client,
            batch_size,
        })
    }

    /// Generate embedding for a single chunk
    pub async fn embed_chunk(&self, chunk: &Chunk) -> AnyResult<Embedding> {
        let text = chunk_to_embedding_text(chunk);
        self.client.embed_text(&text).await
    }

    /// Generate embeddings for multiple chunks (batched)
    pub async fn embed_chunks(&self, chunks: &[Chunk]) -> AnyResult<Vec<Embedding>> {
        let mut all_embeddings = Vec::with_capacity(chunks.len());

        for chunk_batch in chunks.chunks(self.batch_size) {
            let texts: Vec<String> = chunk_batch
                .iter()
                .map(chunk_to_embedding_text)
                .collect();

            let batch_embeddings = self.client.embed_batch(&texts).await?;
            all_embeddings.extend(batch_embeddings);
        }

        Ok(all_embeddings)
    }

    /// Generate embeddings synchronously
    pub fn embed_chunks_sync(&self, chunks: &[Chunk]) -> AnyResult<Vec<Embedding>>
    where
        Self: Sized,
    {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(self.embed_chunks(chunks))
    }
}

/// Embedding with metadata
#[derive(Debug, Clone)]
pub struct EmbeddingResult {
    pub embedding: Embedding,
    pub chunk_id: String,
    pub chunk: Chunk,
}

/// Embedding batch with results
pub struct EmbeddingBatch {
    pub results: Vec<EmbeddingResult>,
    pub failed_ids: Vec<String>,
}

impl EmbeddingBatch {
    /// Create a new batch result
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            failed_ids: Vec::new(),
        }
    }

    /// Get the number of successful embeddings
    pub fn success_count(&self) -> usize {
        self.results.len()
    }

    /// Get the number of failed embeddings
    pub fn failure_count(&self) -> usize {
        self.failed_ids.len()
    }

    /// Check if all embeddings succeeded
    pub fn is_complete(&self) -> bool {
        self.failed_ids.is_empty()
    }
}

impl Default for EmbeddingBatch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunker::Chunk;
    use crate::analyzer::symbol::SymbolKind;

    #[test]
    fn test_embedding_result() {
        let chunk = Chunk {
            id: "test-id".to_string(),
            content_hash: "abc".to_string(),
            repo: "test-repo".to_string(),
            branch: "main".to_string(),
            commit: "123".to_string(),
            language: "rust".to_string(),
            file: "lib.rs".to_string(),
            module: "crate".to_string(),
            symbol: "hello".to_string(),
            kind: "function".to_string(),
            signature: "fn hello()".to_string(),
            doc: None,
            code: "fn hello() {}".to_string(),
            start_line: 1,
            end_line: 2,
        };

        let text = chunk_to_embedding_text(&chunk);
        assert!(text.contains("[rust]"));
        assert!(text.contains("fn hello()"));
    }
}
