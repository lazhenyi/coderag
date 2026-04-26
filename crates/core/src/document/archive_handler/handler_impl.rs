//! Archive handler implementation

use super::extractors;
use crate::chunker::Chunk;
use crate::document::{detect_doc_format, BinaryExtractor, DocFormat, TextChunker};
use std::path::Path;
use tracing::warn;

/// Represents a file extracted from an archive
#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    pub path: String,
    pub content: Vec<u8>,
}

/// Archive handler for recursive processing
pub struct ArchiveHandler {
    text_chunker: TextChunker,
    binary_extractor: BinaryExtractor,
}

impl ArchiveHandler {
    pub fn new() -> Self {
        Self {
            text_chunker: TextChunker::new(),
            binary_extractor: BinaryExtractor::new(),
        }
    }

    pub fn with_text_chunker(mut self, chunker: TextChunker) -> Self {
        self.text_chunker = chunker;
        self
    }

    pub fn process_archive(
        &self,
        path: &Path,
        content: &[u8],
        format: DocFormat,
        repo: &str,
        branch: &str,
        commit: &str,
    ) -> Option<Vec<Chunk>> {
        if !format.is_archive() {
            return None;
        }

        let entries = match format {
            DocFormat::Zip => extractors::extract_zip(content),
            DocFormat::Tar => extractors::extract_tar(content),
            DocFormat::Gz => extractors::extract_gz(content),
            _ => return None,
        }?;

        let mut all_chunks = Vec::new();
        for entry in &entries {
            let entry_path = Path::new(&entry.path);
            let doc_format = detect_doc_format(entry_path);
            if doc_format == DocFormat::Unknown { continue; }

            if let Some(chunks) = self.text_chunker.process_file(entry_path, &entry.content, doc_format, repo, branch, commit) {
                all_chunks.extend(chunks);
                continue;
            }

            if doc_format.is_p1_supported() {
                if let Some(text) = self.binary_extractor.extract_text(entry_path, &entry.content, doc_format) {
                    if let Some(mut chunks) = self.text_chunker.process_file(entry_path, text.as_bytes(), doc_format, repo, branch, commit) {
                        for chunk in &mut chunks {
                            chunk.file = format!("{}:{}", path.display(), entry.path);
                        }
                        all_chunks.extend(chunks);
                    }
                }
            }
        }

        if all_chunks.is_empty() { None } else { Some(all_chunks) }
    }
}

impl Default for ArchiveHandler {
    fn default() -> Self { Self::new() }
}
