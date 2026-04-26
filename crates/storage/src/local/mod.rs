//! Local Vector Store Module
//!
//! File-based vector storage as alternative to Qdrant.
//! Persists embeddings directly in git repository:
//! - Worktree repos: `.git/embed/`
//! - Bare repos: `embed/`

mod backend;
mod search_ops;
mod store;
mod tests;
mod types;

pub use backend::{StorageBackend, StorageConfig};
pub use store::{LocalStore, cosine_similarity_fn as cosine_similarity};
pub use types::{LocalSearchResult, LocalStoreStats, VectorPoint};
