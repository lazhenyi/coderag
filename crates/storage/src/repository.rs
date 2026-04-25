//! Chunk Repository
//!
//! Repository pattern for managing chunks in storage.

use crate::client::{ChunkPayload, QdrantClient, SearchOptions, SearchResult, SearchFilter};
use anyhow::Result as AnyResult;

/// Repository for chunk operations
pub struct ChunkRepository {
    client: QdrantClient,
}

impl ChunkRepository {
    /// Create a new chunk repository
    pub fn new(client: QdrantClient) -> Self {
        Self { client }
    }

    /// Store a chunk with its embedding
    pub async fn store(
        &self,
        chunk_id: &str,
        vector: &[f32],
        payload: ChunkPayload,
    ) -> AnyResult<()> {
        self.client.upsert_points_batch(vec![(chunk_id.to_string(), vector.to_vec(), payload)]).await
    }

    /// Store multiple chunks
    pub async fn store_batch(
        &self,
        chunks: Vec<(String, Vec<f32>, ChunkPayload)>,
    ) -> AnyResult<()> {
        self.client.upsert_points_batch(chunks).await
    }

    /// Search for similar chunks
    pub async fn search(
        &self,
        vector: &[f32],
        options: SearchOptions,
    ) -> AnyResult<Vec<SearchResult>> {
        self.client.search(vector, options).await
    }

    /// Search with filters
    pub async fn search_with_filter(
        &self,
        vector: &[f32],
        language: Option<&str>,
        kind: Option<&str>,
        file: Option<&str>,
        limit: usize,
    ) -> AnyResult<Vec<SearchResult>> {
        let filter = SearchFilter {
            language: language.map(String::from),
            file: file.map(String::from),
            kind: kind.map(String::from),
            repo: None,
        };

        let options = SearchOptions {
            limit,
            offset: None,
            score_threshold: None,
            filter: Some(filter),
        };

        self.search(vector, options).await
    }

    /// Delete a chunk by ID
    pub async fn delete(&self, chunk_id: &str) -> AnyResult<()> {
        self.client.delete_point(chunk_id).await
    }

    /// Delete chunks by language
    pub async fn delete_by_language(&self, language: &str) -> AnyResult<()> {
        let filter = SearchFilter {
            language: Some(language.to_string()),
            file: None,
            kind: None,
            repo: None,
        };

        self.client.delete_by_filter(filter).await
    }

    /// Delete chunks by file path
    pub async fn delete_by_file(&self, file: &str) -> AnyResult<()> {
        let filter = SearchFilter {
            language: None,
            file: Some(file.to_string()),
            kind: None,
            repo: None,
        };

        self.client.delete_by_filter(filter).await
    }

    /// Get collection statistics
    pub async fn stats(&self) -> AnyResult<CollectionStats> {
        let info = self.client.collection_info().await?;

        Ok(CollectionStats {
            total_chunks: info.points_count as usize,
            total_vectors: info.vectors_count as usize,
            status: info.status,
        })
    }
}

/// Collection statistics
#[derive(Debug, Clone)]
pub struct CollectionStats {
    pub total_chunks: usize,
    pub total_vectors: usize,
    pub status: String,
}
