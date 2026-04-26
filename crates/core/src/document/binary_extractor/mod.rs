//! Binary Document Extractor for P1 Formats

mod extractor_impl;
mod html;
mod ooxml;
mod stubs;
mod xls;
mod xlsx;

pub use extractor_impl::BinaryExtractor;
pub use html::extract_html;
pub use ooxml::{extract_docx, extract_ods, extract_odt};
pub use stubs::extract_pdf;
pub use xls::extract_xls;
pub use xlsx::extract_xlsx;
