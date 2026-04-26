//! Local vector store types

use super::super::client::ChunkPayload;
use serde::{Deserialize, Serialize};

/// A single vector point with its embedding and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorPoint {
    pub id: String,
    pub vector: Vec<f32>,
    pub payload: ChunkPayload,
}

/// Search result from local store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSearchResult {
    pub id: String,
    pub score: f32,
    pub payload: ChunkPayload,
}

/// Local store statistics
#[derive(Debug, Clone, Default)]
pub struct LocalStoreStats {
    pub total_points: usize,
    pub vector_dimension: usize,
    pub file_size_bytes: u64,
}
