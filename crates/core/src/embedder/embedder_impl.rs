//! Embedder implementation

use crate::chunker::{Chunk, chunk_to_embedding_text};
use crate::embedder::client_impl::EmbedderClient;
use crate::embedder::types::{EmbedderConfig, Embedding};
use anyhow::Result as AnyResult;

/// Embedder for generating code embeddings
pub struct Embedder {
    client: EmbedderClient,
    batch_size: usize,
}

impl Embedder {
    pub fn new(config: EmbedderConfig) -> AnyResult<Self> {
        let client = EmbedderClient::new(config)?;
        Ok(Self {
            client,
            batch_size: 100,
        })
    }

    pub fn with_batch_size(config: EmbedderConfig, batch_size: usize) -> AnyResult<Self> {
        let client = EmbedderClient::new(config)?;
        Ok(Self { client, batch_size })
    }

    pub async fn embed_chunk(&self, chunk: &Chunk) -> AnyResult<Embedding> {
        let text = chunk_to_embedding_text(chunk);
        self.client.embed_text(&text).await
    }

    pub async fn embed_chunks(&self, chunks: &[Chunk]) -> AnyResult<Vec<Embedding>> {
        let mut all = Vec::with_capacity(chunks.len());
        for batch in chunks.chunks(self.batch_size) {
            let texts: Vec<String> = batch.iter().map(chunk_to_embedding_text).collect();
            let emb = self.client.embed_batch(&texts).await?;
            all.extend(emb);
        }
        Ok(all)
    }

    pub fn embed_chunks_sync(&self, chunks: &[Chunk]) -> AnyResult<Vec<Embedding>> {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(self.embed_chunks(chunks))
    }
}
