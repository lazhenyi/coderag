//! Text Chunker for P0 Formats

mod chunker;
mod chunker_impl;
pub mod config;
mod markdown;
mod tests;
mod xml;

pub use chunker_impl::TextChunker;
pub use config::TextChunkConfig;
pub use markdown::strip_markdown;
pub use xml::strip_xml_tags;
