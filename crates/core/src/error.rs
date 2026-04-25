//! Error types for CodeRAG core library.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Parser error: {0}")]
    Parser(String),

    #[error("Analyzer error: {0}")]
    Analyzer(String),

    #[error("Chunker error: {0}")]
    Chunker(String),

    #[error("Embedder error: {0}")]
    Embedder(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("File too large: {0}")]
    FileTooLarge(String),

    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),
}
