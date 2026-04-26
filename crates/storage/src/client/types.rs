//! Client types

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for Qdrant connection
#[derive(Debug, Clone)]
pub struct QdrantConfig {
    pub url: String,
    pub api_key: Option<String>,
    pub timeout_secs: u64,
    pub collection_name: String,
    pub vector_dimension: usize,
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:6333".to_string(),
            api_key: None,
            timeout_secs: 60,
            collection_name: "coderag".to_string(),
            vector_dimension: 1536,
        }
    }
}

/// Search result from Qdrant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub payload: ChunkPayload,
}

/// Chunk payload stored in Qdrant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkPayload {
    pub id: String,
    pub content_hash: String,
    pub repo: String,
    pub branch: String,
    pub commit: String,
    pub language: String,
    pub file: String,
    pub module: String,
    pub symbol: String,
    pub kind: String,
    pub signature: String,
    pub doc: Option<String>,
    pub code: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// Search options
#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub limit: usize,
    pub offset: Option<usize>,
    pub score_threshold: Option<f32>,
    pub filter: Option<SearchFilter>,
}

/// Search filter
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilter {
    pub language: Option<String>,
    pub file: Option<String>,
    pub kind: Option<String>,
    pub repo: Option<String>,
}

/// Qdrant client wrapper using REST API
pub struct QdrantClient {
    pub(crate) config: QdrantConfig,
    pub(crate) http: Client,
}

/// Collection information
#[derive(Debug, Clone)]
pub struct CollectionInfo {
    pub points_count: u64,
    pub vectors_count: u64,
    pub status: String,
}
