//! Full indexer types

use coderag_core::analyzer::Analyzer;
use coderag_core::chunker::Chunker;
use coderag_core::document::TextChunker;
use coderag_core::embedder::Embedder;
use coderag_core::parser::Parser;
use coderag_storage::StorageBackend;
use crate::IndexConfig;

/// Full indexer for complete repository indexing
pub struct FullIndexer {
    pub(crate) config: IndexConfig,
    pub(crate) parser: Parser,
    pub(crate) analyzer: Analyzer,
    pub(crate) chunker: Chunker,
    pub(crate) text_chunker: TextChunker,
    pub(crate) embedder: Option<Embedder>,
    pub(crate) storage: Option<StorageBackend>,
}
