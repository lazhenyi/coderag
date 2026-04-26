//! Document Parser Module

mod archive_handler;
mod binary_extractor;
mod doc_format;
mod text_chunker;

pub use archive_handler::ArchiveHandler;
pub use binary_extractor::BinaryExtractor;
pub use doc_format::{DocFormat, detect_doc_format};
pub use text_chunker::{TextChunkConfig, TextChunker, strip_markdown, strip_xml_tags};
