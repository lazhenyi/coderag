//! Binary extractor implementation

use super::html::extract_html;
use super::ooxml::{extract_docx, extract_ods, extract_odt};
use super::stubs::extract_pdf;
use super::xls::extract_xls;
use super::xlsx::extract_xlsx;
use crate::document::DocFormat;
use std::path::Path;
use tracing::warn;

/// Extract text from binary document formats
pub struct BinaryExtractor;

impl BinaryExtractor {
    pub fn new() -> Self {
        Self
    }

    pub fn extract_text(&self, path: &Path, content: &[u8], format: DocFormat) -> Option<String> {
        match format {
            DocFormat::Html => extract_html(content),
            DocFormat::Docx => extract_docx(content),
            DocFormat::Odt => extract_odt(content),
            DocFormat::Xlsx => extract_xlsx(content),
            DocFormat::Xls => extract_xls(content),
            DocFormat::Ods => extract_ods(content),
            DocFormat::Pdf => extract_pdf(content),
            _ => {
                warn!(
                    "Unsupported binary format: {:?} for {}",
                    format,
                    path.display()
                );
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
