//! Archive file extractors

use super::ArchiveEntry;
use tracing::warn;

/// Extract files from a ZIP archive
pub fn extract_zip(_data: &[u8]) -> Option<Vec<ArchiveEntry>> {
    #[cfg(feature = "doc-p2")]
    {
        use std::io::{Cursor, Read};
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
                            entries.push(ArchiveEntry {
                                path: name,
                                content,
                            });
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
pub fn extract_tar(_data: &[u8]) -> Option<Vec<ArchiveEntry>> {
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
                                        entries.push(ArchiveEntry {
                                            path: path_str,
                                            content,
                                        });
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
pub fn extract_gz(_data: &[u8]) -> Option<Vec<ArchiveEntry>> {
    #[cfg(feature = "doc-p2")]
    {
        use std::io::Read;
        let mut decoder = flate2::read::GzDecoder::new(_data);
        let mut content = Vec::new();
        if decoder.read_to_end(&mut content).is_ok() && !content.is_empty() {
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
