//! Qdrant Client Module

mod client_impl;
mod search_ops;
mod tests;
mod types;

pub use types::{
    ChunkPayload, QdrantClient, QdrantConfig, SearchFilter, SearchOptions, SearchResult,
};
