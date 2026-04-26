//! Archive handler tests

#[cfg(test)]
mod tests {
    use crate::document::archive_handler::ArchiveHandler;
    use crate::document::DocFormat;
    use std::path::Path;

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
        let result = handler.process_archive(Path::new("test.txt"), b"hello", DocFormat::Text, "test", "main", "abc");
        assert!(result.is_none());
    }

    #[test]
    fn test_invalid_zip_returns_none() {
        let handler = ArchiveHandler::new();
        let result = handler.process_archive(Path::new("test.zip"), b"not a real zip", DocFormat::Zip, "test", "main", "abc");
        assert!(result.is_none());
    }
}
