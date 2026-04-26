//! Text chunker implementation

use super::chunker;
use super::config::TextChunkConfig;
use super::markdown::strip_markdown;
use super::xml::strip_xml_tags;
use crate::chunker::Chunk;
use crate::document::DocFormat;
use std::path::Path;

/// Text chunker for document files
pub struct TextChunker {
    config: TextChunkConfig,
}

impl TextChunker {
    pub fn new() -> Self {
        Self {
            config: TextChunkConfig::default(),
        }
    }

    pub fn with_config(config: TextChunkConfig) -> Self {
        Self { config }
    }

    /// Process a file and return chunks.
    /// Returns None if the format is not P0 supported.
    pub fn process_file(
        &self,
        path: &Path,
        content: &[u8],
        format: DocFormat,
        repo: &str,
        branch: &str,
        commit: &str,
    ) -> Option<Vec<Chunk>> {
        if !format.is_p0_supported() {
            return None;
        }

        let text = String::from_utf8_lossy(content);
        let processed = match format {
            DocFormat::Markdown => {
                if self.config.strip_markdown {
                    strip_markdown(&text)
                } else {
                    text.to_string()
                }
            }
            DocFormat::Xml => strip_xml_tags(&text),
            DocFormat::Json
            | DocFormat::Toml
            | DocFormat::Ini
            | DocFormat::Csv
            | DocFormat::Log => text.to_string(),
            _ => text.to_string(),
        };

        let chunks = chunker::chunk_text(
            &processed,
            &self.config,
            format.language_label(),
            path.to_str().unwrap_or("unknown"),
            repo,
            branch,
            commit,
        );

        if chunks.is_empty() {
            None
        } else {
            Some(chunks)
        }
    }
}

impl Default for TextChunker {
    fn default() -> Self {
        Self::new()
    }
}
