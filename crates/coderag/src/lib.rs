//! CodeRAG — AST-based semantic code indexing and retrieval
//!
//! **CodeRAG** is a pure-Rust code RAG system that indexes repositories
//! using AST-level symbol extraction across 20 programming languages,
//! powered by tree-sitter and git2.
//!
//! # Core capabilities
//!
//! - **AST-level chunking** — extract functions, classes, methods, and more
//!   from source code, preserving semantic boundaries
//! - **Multi-language** — 20 languages: Rust, Python, JavaScript, TypeScript,
//!   Java, Go, C, C++, C#, Swift, Kotlin, PHP, Ruby, Scala, Dart, Lua, R,
//!   Perl, Bash, SQL
//! - **git2-native** — zero shell/git CLI dependency, pure Rust throughout
//! - **Qdrant storage** — vector similarity search with language filtering
//! - **Incremental indexing** — commit-level delta updates, no full rebuilds
//!
//! # Quick start
//!
//! ```
//! use coderag::{FullIndexer, IndexConfig};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config = IndexConfig {
//!     repo_path: "/path/to/repo".to_string(),
//!     branch: "main".to_string(),
//!     qdrant_url: "http://localhost:6333".to_string(),
//!     collection_name: "my-code".to_string(),
//!     embed_api_url: Some("https://api.openai.com/v1/embeddings".to_string()),
//!     embed_api_key: Some("sk-...".to_string()),
//!     ..Default::default()
//! };
//!
//! let indexer = FullIndexer::new(config)?;
//! let stats = indexer.run().await;
//! # Ok(())
//! # }
//! ```
//!
//! # Crate organization
//!
//! | Crate | Purpose |
//! |-------|---------|
//! | `coderag-core` | Git operations, parsers, analyzers, chunker, embedder |
//! | `coderag-storage` | Qdrant client and repository layer |
//! | `coderag-indexer` | Full/incremental indexing orchestration |
//! | `coderag` | Meta-crate re-exporting all sub-crates |

// Re-export the most commonly used types for ergonomic access
pub use coderag_core::analyzer::Analyzer;
pub use coderag_core::analyzer::{Symbol, SymbolKind};
pub use coderag_core::chunker::Chunk;
pub use coderag_core::chunker::Chunker;
pub use coderag_core::embedder::Embedder;
pub use coderag_core::embedder::EmbedderConfig;
pub use coderag_core::parser::Language;
pub use coderag_core::parser::Parser;
pub use coderag_core::repo::GitRepo;

pub use coderag_storage::ChunkRepository;
pub use coderag_storage::{
    ChunkPayload, QdrantClient, QdrantConfig, SearchFilter, SearchOptions, SearchResult,
};

pub use coderag_indexer::{FullIndexer, IncrementalIndexer, IndexConfig, IndexStats};
