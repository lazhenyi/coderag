//! Document format types

use std::path::Path;

/// Supported document file types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocFormat {
    // P0: Pure text formats
    Markdown,
    Text,
    Xml,
    Json,
    Toml,
    Ini,
    Csv,
    Log,

    // P1: Binary / structured formats
    Html,
    Xlsx,
    Xls,
    Ods,
    Docx,
    Odt,
    Pdf,

    // P2: Archives
    Zip,
    Tar,
    Gz,

    // Not a supported document format
    Unknown,
}

impl DocFormat {
    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "md" | "markdown" | "mdx" => Self::Markdown,
            "txt" | "text" => Self::Text,
            "xml" => Self::Xml,
            "json" | "jsonl" | "ndjson" => Self::Json,
            "toml" => Self::Toml,
            "ini" | "cfg" | "conf" | "properties" => Self::Ini,
            "csv" | "tsv" => Self::Csv,
            "log" => Self::Log,

            "html" | "htm" => Self::Html,
            "xlsx" => Self::Xlsx,
            "xls" => Self::Xls,
            "ods" => Self::Ods,
            "docx" => Self::Docx,
            "odt" => Self::Odt,
            "pdf" => Self::Pdf,

            "zip" => Self::Zip,
            "tar" => Self::Tar,
            "gz" | "gzip" | "tgz" => Self::Gz,

            _ => Self::Unknown,
        }
    }

    /// Get the language label for embedding
    pub fn language_label(&self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::Text => "text",
            Self::Xml => "xml",
            Self::Json => "json",
            Self::Toml => "toml",
            Self::Ini => "ini",
            Self::Csv => "csv",
            Self::Log => "log",
            Self::Html => "html",
            Self::Xlsx => "xlsx",
            Self::Xls => "xls",
            Self::Ods => "ods",
            Self::Docx => "docx",
            Self::Odt => "odt",
            Self::Pdf => "pdf",
            Self::Zip => "zip",
            Self::Tar => "tar",
            Self::Gz => "gz",
            Self::Unknown => "unknown",
        }
    }

    /// Check if this format is supported for P0 (pure text)
    pub fn is_p0_supported(&self) -> bool {
        matches!(self, Self::Markdown | Self::Text | Self::Xml | Self::Json | Self::Toml | Self::Ini | Self::Csv | Self::Log)
    }

    /// Check if this format is supported for P1 (binary)
    pub fn is_p1_supported(&self) -> bool {
        matches!(self, Self::Html | Self::Xlsx | Self::Xls | Self::Ods | Self::Docx | Self::Odt | Self::Pdf)
    }

    /// Check if this is an archive format (P2)
    pub fn is_archive(&self) -> bool {
        matches!(self, Self::Zip | Self::Tar | Self::Gz)
    }

    /// Check if this format is supported at all
    pub fn is_supported(&self) -> bool {
        self.is_p0_supported() || self.is_p1_supported() || self.is_archive()
    }
}

/// Detect document format from a file path
pub fn detect_doc_format(path: &Path) -> DocFormat {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        DocFormat::from_extension(ext)
    } else {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            match name {
                "LICENSE" | "LICENSE-MIT" | "LICENSE-APACHE" | "COPYING"
                | "README" | "CHANGELOG" | "CHANGES" | "AUTHORS" => DocFormat::Text,
                "Makefile" | "makefile" | "Dockerfile" | "Containerfile"
                | "Vagrantfile" | "Jenkinsfile" => DocFormat::Text,
                _ => DocFormat::Unknown,
            }
        } else {
            DocFormat::Unknown
        }
    }
}
