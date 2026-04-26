//! PDF extraction using pdf-extract

/// Extract text from PDF
pub fn extract_pdf(content: &[u8]) -> Option<String> {
    #[cfg(feature = "doc-p1")]
    {
        match pdf_extract::extract_text_from_mem(content) {
            Ok(text) => {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            }
            Err(e) => {
                tracing::warn!("Failed to extract PDF text: {}", e);
                None
            }
        }
    }

    #[cfg(not(feature = "doc-p1"))]
    {
        let _ = content;
        tracing::warn!("P1 document features not enabled. Add 'doc-p1' feature to coderag-core.");
        None
    }
}
