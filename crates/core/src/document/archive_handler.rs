//! Archive Handler for P2 Formats
//!
//! Handles: zip, tar, gz

use crate::chunker::Chunk;
use crate::document::{detect_doc_format, DocFormat, TextChunker, BinaryExtractor};
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

    /// Process an archive file and return chunks from all contained files.
    /// Returns None if the format is not an archive or extraction fails.
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
            DocFormat::Zip => extract_zip(content),
            DocFormat::Tar => extract_tar(content),
            DocFormat::Gz => extract_gz(content),
            _ => return None,
        }?;

        let mut all_chunks = Vec::new();

        for entry in &entries {
            let entry_path = Path::new(&entry.path);
            let doc_format = detect_doc_format(entry_path);

            // Skip binary files and unsupported formats
            if doc_format == DocFormat::Unknown {
                continue;
            }

            // Try P0 first
            if let Some(chunks) = self.text_chunker.process_file(
                entry_path,
                &entry.content,
                doc_format,
                repo,
                branch,
                commit,
            ) {
                all_chunks.extend(chunks);
                continue;
            }

            // Try P1
            if doc_format.is_p1_supported() {
                if let Some(text) = self.binary_extractor.extract_text(entry_path, &entry.content, doc_format) {
                    let temp_chunks = self.text_chunker.process_file(
                        entry_path,
                        text.as_bytes(),
                        doc_format,
                        repo,
                        branch,
                        commit,
                    );
                    if let Some(mut chunks) = temp_chunks {
                        // Update file path to include archive context
                        for chunk in &mut chunks {
                            chunk.file = format!("{}:{}", path.display(), entry.path);
                        }
                        all_chunks.extend(chunks);
                    }
                }
            }
        }

        if all_chunks.is_empty() {
            None
        } else {
            Some(all_chunks)
        }
    }
}

impl Default for ArchiveHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract files from a ZIP archive
fn extract_zip(_data: &[u8]) -> Option<Vec<ArchiveEntry>> {
    #[cfg(feature = "doc-p2")]
    {
        use std::io::{Read, Cursor};
        match zip::ZipArchive::new(Cursor::new(_data)) {
            Ok(mut archive) => {
                let mut entries = Vec::new();
                for i in 0..archive.len() {
                    if let Ok(mut file) = archive.by_index(i) {
                        if file.is_dir() {
                            continue;
                        }
                        let name = file.name().to_string();
                        let mut content = Vec::new();
                        if file.read_to_end(&mut content).is_ok() {
                            entries.push(ArchiveEntry { path: name, content });
                        }
                    }
                }
                if entries.is_empty() {
                    None
                } else {
                    Some(entries)
                }
            }
            Err(e) => {
                warn!("Failed to open ZIP: {}", e);
                None
            }
        }
    }

    #[cfg(not(feature = "doc-p2"))]
    {
        warn!("P2 archive features not enabled. Add 'doc-p2' feature.");
        if _data.len() >= 4 && _data[0] == b'P' && _data[1] == b'K' {
            warn!("ZIP detected but feature not enabled, returning None");
        }
        None
    }
}

/// Extract files from a TAR archive
fn extract_tar(_data: &[u8]) -> Option<Vec<ArchiveEntry>> {
    #[cfg(feature = "doc-p2")]
    {
        use std::io::Cursor;
        match tar::Archive::new(Cursor::new(_data)) {
            Ok(mut archive) => {
                let mut entries = Vec::new();
                if let Ok(entries_iter) = archive.entries() {
                    for entry_result in entries_iter {
                        if let Ok(mut entry) = entry_result {
                            if entry.header().entry_type() == tar::EntryType::Regular {
                                if let Ok(path) = entry.path() {
                                    let path_str = path.to_string_lossy().to_string();
                                    let mut content = Vec::new();
                                    if entry.read_to_end(&mut content).is_ok() {
                                        entries.push(ArchiveEntry { path: path_str, content });
                                    }
                                }
                            }
                        }
                    }
                }
                if entries.is_empty() {
                    None
                } else {
                    Some(entries)
                }
            }
            Err(e) => {
                warn!("Failed to open TAR: {}", e);
                None
            }
        }
    }

    #[cfg(not(feature = "doc-p2"))]
    {
        warn!("P2 archive features not enabled. Add 'doc-p2' feature.");
        None
    }
}

/// Extract content from a GZ archive
fn extract_gz(_data: &[u8]) -> Option<Vec<ArchiveEntry>> {
    #[cfg(feature = "doc-p2")]
    {
        use std::io::Read;
        let mut decoder = flate2::read::GzDecoder::new(_data);
        let mut content = Vec::new();
        if decoder.read_to_end(&mut content).is_ok() && !content.is_empty() {
            // GZ usually contains a single file
            Some(vec![ArchiveEntry {
                path: "extracted".to_string(),
                content,
            }])
        } else {
            None
        }
    }

    #[cfg(not(feature = "doc-p2"))]
    {
        warn!("P2 archive features not enabled. Add 'doc-p2' feature.");
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_format_detection() {
        assert!(DocFormat::Zip.is_archive());
        assert!(DocFormat::Tar.is_archive());
        assert!(DocFormat::Gz.is_archive());
        assert!(!DocFormat::Text.is_archive());
    }

    #[test]
    fn test_non_archive_returns_none() {
        let handler = ArchiveHandler::new();
        let result = handler.process_archive(
            Path::new("test.txt"),
            b"hello",
            DocFormat::Text,
            "test",
            "main",
            "abc",
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_invalid_zip_returns_none() {
        let handler = ArchiveHandler::new();
        let result = handler.process_archive(
            Path::new("test.zip"),
            b"not a real zip",
            DocFormat::Zip,
            "test",
            "main",
            "abc",
        );
        // Should return None since it's not a valid ZIP
        assert!(result.is_none());
    }
}
