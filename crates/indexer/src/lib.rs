//! Indexer Module
//!
//! Orchestrates the indexing pipeline for code repositories.

mod full;
mod incremental;
mod state;

pub use full::FullIndexer;
pub use incremental::IncrementalIndexer;
pub use state::IndexerState;

/// Indexer configuration
#[derive(Debug, Clone)]
pub struct IndexConfig {
    /// Repository path
    pub repo_path: String,
    /// Branch to index
    pub branch: String,
    /// Languages to track (empty = all)
    pub tracked_languages: Vec<String>,
    /// Batch size for processing
    pub batch_size: usize,
    /// Qdrant collection name
    pub collection_name: String,
    /// Qdrant URL
    pub qdrant_url: String,
    /// Qdrant API key
    pub qdrant_api_key: Option<String>,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            repo_path: ".".to_string(),
            branch: "main".to_string(),
            tracked_languages: Vec::new(),
            batch_size: 100,
            collection_name: "coderag".to_string(),
            qdrant_url: "http://localhost:6334".to_string(),
            qdrant_api_key: None,
        }
    }
}

/// Indexing statistics
#[derive(Debug, Clone, Default)]
pub struct IndexStats {
    pub files_processed: usize,
    pub symbols_extracted: usize,
    pub chunks_created: usize,
    pub embeddings_generated: usize,
    pub duration_secs: f64,
}

impl IndexStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_files(&mut self) {
        self.files_processed += 1;
    }

    pub fn add_symbols(&mut self, count: usize) {
        self.symbols_extracted += count;
    }

    pub fn add_chunks(&mut self, count: usize) {
        self.chunks_created += count;
    }

    pub fn add_embeddings(&mut self, count: usize) {
        self.embeddings_generated += count;
    }
}
