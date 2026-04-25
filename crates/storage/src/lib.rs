//! Storage Module
//!
//! Qdrant vector storage integration for CodeRAG.

pub mod client;
pub mod repository;

pub use client::{QdrantClient, QdrantConfig, SearchResult, SearchOptions, ChunkPayload, SearchFilter};
pub use repository::ChunkRepository;

/// Storage error types
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Collection not found: {0}")]
    CollectionNotFound(String),

    #[error("Connection error: {0}")]
    Connection(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;
