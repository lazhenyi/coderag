//! Chunker Module
//!
//! Creates code chunks from extracted symbols for embedding.

mod chunker_impl;
mod tests;
mod types;

pub use chunker_impl::Chunker;
pub use types::{Chunk, ChunkConfig, chunk_to_embedding_text};
