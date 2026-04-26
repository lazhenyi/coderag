//! Chunk types and config

use serde::{Deserialize, Serialize};

/// A code chunk for embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
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

/// Configuration for chunking
#[derive(Debug, Clone)]
pub struct ChunkConfig {
    pub max_chunk_size: usize,
    pub include_docs: bool,
    pub include_signatures: bool,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            max_chunk_size: 2000,
            include_docs: true,
            include_signatures: true,
        }
    }
}

/// Embedding text representation of a chunk
pub fn chunk_to_embedding_text(chunk: &Chunk) -> String {
    let mut parts = Vec::new();
    parts.push(format!("[{}] ", chunk.language));
    if !chunk.signature.is_empty() {
        parts.push(chunk.signature.clone());
    }
    if let Some(ref doc) = chunk.doc {
        if !doc.is_empty() {
            parts.push(doc.clone());
        }
    }
    parts.push(chunk.code.clone());
    parts.join("\n")
}
