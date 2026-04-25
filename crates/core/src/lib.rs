//! CodeRAG Core Library
//!
//! AST-based code semantic indexing system using git2 and tree-sitter.

pub mod repo;
pub mod diff;
pub mod parser;
pub mod analyzer;
pub mod chunker;
pub mod embedder;
pub mod error;

pub use error::{Error, Result};
