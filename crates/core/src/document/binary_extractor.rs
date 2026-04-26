//! Binary Document Extractor for P1 Formats
//!
//! Handles: html, xlsx, xls, ods, docx, odt, pdf

use crate::document::DocFormat;
use std::path::Path;
use tracing::warn;

/// Extract text from binary document formats
pub struct BinaryExtractor;

impl BinaryExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Extract text content from a binary document.
    /// Returns None if format is not P1 or extraction fails.
    pub fn extract_text(
        &self,
        path: &Path,
        content: &[u8],
        format: DocFormat,
    ) -> Option<String> {
        match format {
            DocFormat::Html => extract_html(content),
            DocFormat::Docx => extract_docx(content),
            DocFormat::Odt => extract_odt(content),
            DocFormat::Xlsx => extract_xlsx(content),
            DocFormat::Xls => extract_xls(content),
            DocFormat::Ods => extract_ods(content),
            DocFormat::Pdf => extract_pdf(content),
            _ => {
                warn!("Unsupported binary format: {:?} for {}", format, path.display());
                None
            }
        }
    }

    pub fn is_supported(&self, format: DocFormat) -> bool {
        format.is_p1_supported()
    }
}

impl Default for BinaryExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract text from HTML by stripping tags
fn extract_html(content: &[u8]) -> Option<String> {
    let html = String::from_utf8_lossy(content);

    // Simple HTML text extraction: strip tags, handle common entities
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut tag_buf = String::new();

    for ch in html.chars() {
        match ch {
            '<' => {
                in_tag = true;
                tag_buf.clear();
            }
            '>' if in_tag => {
                in_tag = false;
                let tag_lower = tag_buf.to_lowercase();
                if tag_lower.starts_with("script") && !tag_lower.starts_with("/script") {
                    in_script = true;
                } else if tag_lower.starts_with("/script") {
                    in_script = false;
                } else if tag_lower.starts_with("style") && !tag_lower.starts_with("/style") {
                    in_style = true;
                } else if tag_lower.starts_with("/style") {
                    in_style = false;
                } else if matches!(tag_lower.split_whitespace().next(), Some("br" | "p" | "div" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "li" | "tr")) {
                    result.push('\n');
                }
            }
            _ if in_tag => {
                tag_buf.push(ch);
            }
            _ if !in_script && !in_style => {
                result.push(ch);
            }
            _ => {}
        }
    }

    // Handle common HTML entities
    let result = result
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'");

    // Collapse multiple blank lines
    let result = result
        .lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n");

    let trimmed = result.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Extract text from .docx (ZIP + XML)
fn extract_docx(content: &[u8]) -> Option<String> {
    extract_ooxml_text(content, "word/document.xml")
}

/// Extract text from .xlsx (ZIP + XML sheets)
fn extract_xlsx(content: &[u8]) -> Option<String> {
    // Extract shared strings and sheet data
    extract_xlsx_data(content)
}

/// Extract text from .xls (older BIFF format)
fn extract_xls(_content: &[u8]) -> Option<String> {
    // XLS is a binary BIFF format, requires calamine crate
    // Placeholder: return None for now
    warn!("XLS (BIFF) format requires the calamine crate. Install with: cargo add calamine");
    None
}

/// Extract text from .ods (OpenDocument Spreadsheet)
fn extract_ods(content: &[u8]) -> Option<String> {
    extract_ooxml_text(content, "content.xml")
}

/// Extract text from .odt (OpenDocument Text)
fn extract_odt(content: &[u8]) -> Option<String> {
    extract_ooxml_text(content, "content.xml")
}

/// Extract text from PDF
fn extract_pdf(_content: &[u8]) -> Option<String> {
    // PDF requires pdf-extract crate
    // Placeholder: return None for now
    warn!("PDF extraction requires the pdf-extract crate. Install with: cargo add pdf-extract");
    None
}

/// Common helper: extract text from XML inside a ZIP archive
fn extract_ooxml_text(zip_data: &[u8], _xml_path: &str) -> Option<String> {
    // Check if this looks like a ZIP (starts with PK)
    if zip_data.len() < 4 || zip_data[0] != b'P' || zip_data[1] != b'K' {
        warn!("Not a valid ZIP archive");
        return None;
    }

    #[cfg(feature = "doc-p1")]
    {
        use std::io::{Read, Cursor};
        match zip::ZipArchive::new(Cursor::new(zip_data)) {
            Ok(mut archive) => {
                if let Ok(mut file) = archive.by_name(xml_path) {
                    let mut xml_content = String::new();
                    if file.read_to_string(&mut xml_content).is_ok() {
                        let text = crate::document::text_chunker::strip_xml_tags(&xml_content);
                        let trimmed = text.trim();
                        if trimmed.is_empty() {
                            None
                        } else {
                            Some(trimmed.to_string())
                        }
                    } else {
                        None
                    }
                } else {
                    warn!("File {} not found in archive", xml_path);
                    None
                }
            }
            Err(e) => {
                warn!("Failed to open ZIP archive: {}", e);
                None
            }
        }
    }

    #[cfg(not(feature = "doc-p1"))]
    {
        warn!("P1 document features not enabled. Add 'doc-p1' feature to coderag-core.");
        None
    }
}

/// Extract data from xlsx files
fn extract_xlsx_data(_zip_data: &[u8]) -> Option<String> {
    #[cfg(feature = "doc-p1")]
    {
        use std::io::{Read, Cursor};
        match zip::ZipArchive::new(Cursor::new(_zip_data)) {
            Ok(mut archive) => {
                // Try to read shared strings first
                let mut shared_strings: Vec<String> = Vec::new();
                if let Ok(mut file) = archive.by_name("xl/sharedStrings.xml") {
                    let mut xml = String::new();
                    if file.read_to_string(&mut xml).is_ok() {
                        // Extract <t>...</t> elements
                        for cap in simple_find_all(&xml, "<t", "</t>") {
                            shared_strings.push(cap);
                        }
                    }
                }

                // Read first sheet
                let mut result = String::new();
                if let Ok(mut file) = archive.by_name("xl/worksheets/sheet1.xml") {
                    let mut xml = String::new();
                    if file.read_to_string(&mut xml).is_ok() {
                        // Extract all <v>...</v> values
                        for cap in simple_find_all(&xml, "<v>", "</v>") {
                            if let Ok(idx) = cap.parse::<usize>() {
                                if idx < shared_strings.len() {
                                    result.push_str(&shared_strings[idx]);
                                } else {
                                    result.push_str(&cap);
                                }
                                result.push('\t');
                            } else {
                                result.push_str(&cap);
                                result.push('\t');
                            }
                        }
                        result.push('\n');
                    }
                }

                let trimmed = result.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            }
            Err(e) => {
                warn!("Failed to open xlsx: {}", e);
                None
            }
        }
    }

    #[cfg(not(feature = "doc-p1"))]
    {
        warn!("P1 document features not enabled. Add 'doc-p1' feature to coderag-core.");
        None
    }
}

/// Simple substring finder (no regex dependency)
#[cfg(feature = "doc-p1")]
fn simple_find_all(text: &str, start: &str, end: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut pos = 0;

    while let Some(start_idx) = text[pos..].find(start) {
        let content_start = pos + start_idx + start.len();
        if let Some(end_idx) = text[content_start..].find(end) {
            let content = &text[content_start..content_start + end_idx];
            results.push(content.to_string());
            pos = content_start + end_idx + end.len();
        } else {
            break;
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_extraction() {
        let html = r#"<html><head><title>Test</title></head>
<body>
<h1>Hello</h1>
<p>This is <b>bold</b> text.</p>
<script>alert('ignore me');</script>
<style>.hidden { display: none; }</style>
</body></html>"#;

        let result = extract_html(html.as_bytes()).unwrap();
        assert!(result.contains("Hello"));
        assert!(result.contains("This is bold text."));
        assert!(!result.contains("alert"));
        assert!(!result.contains(".hidden"));
    }

    #[test]
    fn test_html_entities() {
        let html = "<p>Tom &amp; Jerry &lt;cat&gt;</p>";
        let result = extract_html(html.as_bytes()).unwrap();
        assert!(result.contains("Tom & Jerry <cat>"));
    }

    #[test]
    fn test_unsupported_format() {
        let extractor = BinaryExtractor::new();
        assert!(!extractor.is_supported(DocFormat::Text));
        assert!(extractor.is_supported(DocFormat::Docx));
        assert!(extractor.is_supported(DocFormat::Pdf));
    }

    #[test]
    fn test_empty_html() {
        let html = "<html><body></body></html>";
        let result = extract_html(html.as_bytes());
        assert!(result.is_none());
    }
}
