//! CodeRAG Core Library
//!
//! AST-based code semantic indexing system using git2 and tree-sitter.

pub mod analyzer;
pub mod chunker;
pub mod diff;
pub mod document;
pub mod embedder;
pub mod error;
pub mod parser;
pub mod repo;

pub use error::{Error, Result};
