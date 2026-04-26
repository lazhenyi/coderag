//! Text Chunker for P0 Formats
//!
//! Handles: markdown, text, xml, json, toml, ini, csv, log

use crate::chunker::Chunk;
use crate::document::DocFormat;
use sha2::{Digest, Sha256};
use std::path::Path;

/// Configuration for text chunking
#[derive(Debug, Clone)]
pub struct TextChunkConfig {
    /// Maximum chunk size in characters
    pub max_chunk_size: usize,
    /// Overlap between chunks for context continuity
    pub overlap: usize,
    /// Strip markdown formatting for cleaner text
    pub strip_markdown: bool,
}

impl Default for TextChunkConfig {
    fn default() -> Self {
        Self {
            max_chunk_size: 2000,
            overlap: 200,
            strip_markdown: true,
        }
    }
}

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
            DocFormat::Json => text.to_string(), // JSON is already structured
            DocFormat::Toml => text.to_string(),
            DocFormat::Ini => text.to_string(),
            DocFormat::Csv => text.to_string(), // CSV parsed as-is
            DocFormat::Log => text.to_string(),
            _ => text.to_string(),
        };

        let chunks = self.chunk_text(
            &processed,
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

    /// Split text into chunks with overlap
    fn chunk_text(
        &self,
        text: &str,
        language: &str,
        file_path: &str,
        repo: &str,
        branch: &str,
        commit: &str,
    ) -> Vec<Chunk> {
        let text = text.trim();
        if text.is_empty() {
            return vec![];
        }

        let mut chunks = Vec::new();

        // If text fits in one chunk, emit a single chunk
        if text.len() <= self.config.max_chunk_size {
            chunks.push(make_chunk(
                text, 1, 1, language, file_path, repo, branch, commit,
            ));
            return chunks;
        }

        // Split by paragraphs first
        let paragraphs: Vec<&str> = text
            .split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .collect();

        let mut current_chunk = String::new();
        let mut chunk_start_line = 1;
        let mut current_line = 1;

        for para in &paragraphs {
            let para_lines = para.lines().count();

            // If paragraph alone exceeds limit, split it into character-based chunks
            if para.len() > self.config.max_chunk_size {
                // First, emit any accumulated chunk
                if !current_chunk.is_empty() {
                    let chunk_lines = current_chunk.lines().count();
                    chunks.push(make_chunk(
                        &current_chunk,
                        chunk_start_line,
                        chunk_start_line + chunk_lines - 1,
                        language,
                        file_path,
                        repo,
                        branch,
                        commit,
                    ));
                    current_chunk.clear();
                }

                // Split the long paragraph into fixed-size chunks
                let mut remaining: &str = para;
                let mut line_offset = current_line;
                while remaining.len() > self.config.max_chunk_size {
                    let mut split_at = self.config.max_chunk_size;
                    // Try to find word boundary
                    if let Some(idx) = remaining[..split_at].rfind(char::is_whitespace) {
                        if idx > 10 {
                            split_at = idx;
                        }
                    }
                    let part = &remaining[..split_at];
                    chunks.push(make_chunk(
                        part.trim(),
                        line_offset,
                        line_offset + part.lines().count(),
                        language,
                        file_path,
                        repo,
                        branch,
                        commit,
                    ));
                    remaining = &remaining[split_at..];
                    line_offset += part.lines().count();
                }
                // Start new chunk with the tail
                current_chunk = remaining.trim().to_string();
                chunk_start_line = line_offset;
            } else if current_chunk.len() + para.len() + 2 > self.config.max_chunk_size && !current_chunk.is_empty() {
                // Emit current chunk
                let chunk_lines = current_chunk.lines().count();
                chunks.push(make_chunk(
                    &current_chunk,
                    chunk_start_line,
                    chunk_start_line + chunk_lines - 1,
                    language,
                    file_path,
                    repo,
                    branch,
                    commit,
                ));

                // Start new chunk with overlap
                let overlap_text = if current_chunk.len() > self.config.overlap {
                    let overlap_start = current_chunk.len() - self.config.overlap;
                    let idx = current_chunk[overlap_start..]
                        .find(char::is_whitespace)
                        .map(|i| overlap_start + i)
                        .unwrap_or(overlap_start);
                    current_chunk[idx..].to_string()
                } else {
                    current_chunk.clone()
                };

                current_chunk = overlap_text.trim().to_string();
                chunk_start_line = current_line - current_chunk.lines().count();
                if chunk_start_line < 1 {
                    chunk_start_line = 1;
                }

                if !current_chunk.is_empty() {
                    current_chunk.push_str("\n\n");
                }
                current_chunk.push_str(para);
            } else {
                if !current_chunk.is_empty() {
                    current_chunk.push_str("\n\n");
                }
                current_chunk.push_str(para);
            }

            current_line += para_lines;
        }

        // Emit final chunk
        if !current_chunk.is_empty() {
            let chunk_lines = current_chunk.lines().count();
            chunks.push(make_chunk(
                &current_chunk,
                chunk_start_line,
                chunk_start_line + chunk_lines - 1,
                language,
                file_path,
                repo,
                branch,
                commit,
            ));
        }

        // If paragraph splitting produced nothing, fall back to character-based splitting
        if chunks.is_empty() {
            let lines: Vec<&str> = text.lines().collect();
            let mut buf = String::new();
            let mut start = 1;
            let mut line_num = 1;

            for line in &lines {
                if buf.len() + line.len() + 1 > self.config.max_chunk_size && !buf.is_empty() {
                    chunks.push(make_chunk(
                        &buf, start, start + buf.lines().count() - 1,
                        language, file_path, repo, branch, commit,
                    ));
                    buf = String::new();
                    start = line_num;
                }
                if !buf.is_empty() {
                    buf.push('\n');
                }
                buf.push_str(line);
                line_num += 1;
            }
            if !buf.is_empty() {
                chunks.push(make_chunk(
                    &buf, start, start + buf.lines().count() - 1,
                    language, file_path, repo, branch, commit,
                ));
            }
        }

        chunks
    }
}

impl Default for TextChunker {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a Chunk from text content
fn make_chunk(
    text: &str,
    start_line: usize,
    end_line: usize,
    language: &str,
    file_path: &str,
    repo: &str,
    branch: &str,
    commit: &str,
) -> Chunk {
    let id = uuid::Uuid::new_v4().to_string();
    let content_hash = compute_hash(text);

    // Extract a signature: first meaningful line or first 80 chars
    let signature = text
        .lines()
        .find(|l| !l.trim().is_empty())
        .map(|l| {
            let trimmed = l.trim();
            if trimmed.len() > 80 {
                format!("{}...", &trimmed[..80])
            } else {
                trimmed.to_string()
            }
        })
        .unwrap_or_else(|| "(empty)".to_string());

    Chunk {
        id,
        content_hash,
        repo: repo.to_string(),
        branch: branch.to_string(),
        commit: commit.to_string(),
        language: language.to_string(),
        file: file_path.to_string(),
        module: String::new(),
        symbol: String::new(),
        kind: "document".to_string(),
        signature,
        doc: None,
        code: text.to_string(),
        start_line,
        end_line,
    }
}

fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

/// Strip markdown formatting, leaving plain text
fn strip_markdown(text: &str) -> String {
    let mut result = String::with_capacity(text.len());

    for line in text.lines() {
        // Strip headers
        let stripped = if line.starts_with('#') {
            line.trim_start_matches('#').trim_start().to_string()
        } else {
            line.to_string()
        };

        // Strip bold/italic markers
        let stripped = stripped
            .replace("**", "")
            .replace("__", "")
            .replace('*', "")
            .replace('_', "");

        // Strip links: [text](url) -> text
        let stripped = strip_links(&stripped);

        // Strip images
        let stripped = strip_images(&stripped);

        // Strip code blocks markers
        let stripped = if stripped.starts_with("```") || stripped.starts_with("~~~") {
            String::new()
        } else {
            stripped
        };

        // Strip list markers
        let stripped = strip_list_marker(&stripped);

        if !stripped.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&stripped);
        }
    }

    result
}

fn strip_links(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '[' {
            // Collect link text
            let mut link_text = String::new();
            let mut found_close = false;
            while let Some(c) = chars.next() {
                if c == ']' {
                    found_close = true;
                    break;
                }
                link_text.push(c);
            }
            // Skip the URL part if it exists
            if found_close {
                if chars.peek() == Some(&'(') {
                    chars.next(); // skip (
                    let mut depth = 1;
                    while let Some(c) = chars.next() {
                        if c == '(' {
                            depth += 1;
                        } else if c == ')' {
                            depth -= 1;
                        }
                        if depth == 0 {
                            break;
                        }
                    }
                }
            }
            result.push_str(&link_text);
        } else {
            result.push(ch);
        }
    }

    result
}

fn strip_images(text: &str) -> String {
    // Remove ![alt](url) patterns
    let mut result = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '!' && chars.peek() == Some(&'[') {
            chars.next(); // skip [
            // Skip to ]
            let mut depth = 1;
            while let Some(c) = chars.next() {
                if c == '[' {
                    depth += 1;
                } else if c == ']' {
                    depth -= 1;
                }
                if depth == 0 {
                    break;
                }
            }
            // Skip (url)
            if chars.peek() == Some(&'(') {
                chars.next();
                let mut depth = 1;
                while let Some(c) = chars.next() {
                    if c == '(' {
                        depth += 1;
                    } else if c == ')' {
                        depth -= 1;
                    }
                    if depth == 0 {
                        break;
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

fn strip_list_marker(line: &str) -> String {
    let trimmed = line.trim_start();
    // Match patterns like "- ", "* ", "+ ", "1. ", etc.
    if trimmed.starts_with("- ")
        || trimmed.starts_with("* ")
        || trimmed.starts_with("+ ")
    {
        trimmed[2..].to_string()
    } else {
        line.to_string()
    }
}

/// Strip XML tags, leaving text content
fn strip_xml_tags(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut in_tag = false;

    for ch in text.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ => {
                if !in_tag {
                    result.push(ch);
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_strip() {
        let md = r#"# Header
Some **bold** and *italic* text.

## Another Header
A [link](https://example.com) here.

- Item 1
- Item 2

```rust
fn main() {}
```

Plain text at end."#;

        let result = strip_markdown(md);
        assert!(result.contains("Header"));
        assert!(result.contains("bold and italic text"));
        assert!(result.contains("link"));
        assert!(result.contains("Item 1"));
        assert!(result.contains("Plain text at end"));
        assert!(!result.contains("**"));
        assert!(!result.contains("https://example.com"));
    }

    #[test]
    fn test_xml_tag_stripping() {
        let xml = r#"<root>
  <item>Hello</item>
  <item>World</item>
</root>"#;

        let result = strip_xml_tags(xml);
        assert!(result.contains("Hello"));
        assert!(result.contains("World"));
        assert!(!result.contains("<root>"));
        assert!(!result.contains("</item>"));
    }

    #[test]
    fn test_text_chunker_single_chunk() {
        let chunker = TextChunker::new();
        let content = b"Hello, this is a short document.";

        let chunks = chunker
            .process_file(
                Path::new("test.txt"),
                content,
                DocFormat::Text,
                "test-repo",
                "main",
                "abc123",
            )
            .unwrap();

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].language, "text");
        assert_eq!(chunks[0].kind, "document");
        assert!(!chunks[0].content_hash.is_empty());
    }

    #[test]
    fn test_text_chunker_multi_chunk() {
        let chunker = TextChunker::with_config(TextChunkConfig {
            max_chunk_size: 50,
            overlap: 10,
            strip_markdown: true,
        });

        // Create a long text with multiple paragraphs
        let mut long_text = String::new();
        for i in 0..30 {
            if i > 0 {
                long_text.push_str("\n\n");
            }
            long_text.push_str(&format!("Paragraph {}: This is paragraph number {} with some additional text to make it longer.", i, i));
        }

        let chunks = chunker
            .process_file(
                Path::new("long.md"),
                long_text.as_bytes(),
                DocFormat::Markdown,
                "test-repo",
                "main",
                "abc123",
            )
            .unwrap();

        assert!(chunks.len() > 1, "Expected multiple chunks, got {}", chunks.len());
        assert_eq!(chunks[0].language, "markdown");
    }

    #[test]
    fn test_format_detection() {
        assert_eq!(DocFormat::from_extension("md"), DocFormat::Markdown);
        assert_eq!(DocFormat::from_extension("json"), DocFormat::Json);
        assert_eq!(DocFormat::from_extension("csv"), DocFormat::Csv);
        assert_eq!(DocFormat::from_extension("log"), DocFormat::Log);
        assert_eq!(DocFormat::from_extension("toml"), DocFormat::Toml);
        assert_eq!(DocFormat::from_extension("ini"), DocFormat::Ini);
        assert_eq!(DocFormat::from_extension("xml"), DocFormat::Xml);
        assert_eq!(DocFormat::from_extension("txt"), DocFormat::Text);
        assert_eq!(DocFormat::from_extension("unknown"), DocFormat::Unknown);
    }

    #[test]
    fn test_p0_support_flags() {
        assert!(DocFormat::Markdown.is_p0_supported());
        assert!(DocFormat::Json.is_p0_supported());
        assert!(!DocFormat::Markdown.is_p1_supported());
        assert!(!DocFormat::Markdown.is_archive());
        assert!(DocFormat::Markdown.is_supported());
    }

    #[test]
    fn test_json_chunking() {
        let chunker = TextChunker::new();
        let json = br#"{"name": "test", "version": "1.0", "description": "A test project"}"#;

        let chunks = chunker
            .process_file(
                Path::new("test.json"),
                json,
                DocFormat::Json,
                "test-repo",
                "main",
                "abc123",
            )
            .unwrap();

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].language, "json");
        assert!(chunks[0].code.contains("test"));
    }

    #[test]
    fn test_csv_chunking() {
        let chunker = TextChunker::new();
        let csv = b"name,age,city\nAlice,30,NYC\nBob,25,LA\n";

        let chunks = chunker
            .process_file(
                Path::new("data.csv"),
                csv,
                DocFormat::Csv,
                "test-repo",
                "main",
                "abc123",
            )
            .unwrap();

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].language, "csv");
        assert!(chunks[0].code.contains("Alice"));
    }

    #[test]
    fn test_log_chunking() {
        let chunker = TextChunker::new();
        let log = b"2024-01-01 INFO Starting server\n2024-01-01 ERROR Connection failed\n2024-01-01 INFO Retrying\n";

        let chunks = chunker
            .process_file(
                Path::new("app.log"),
                log,
                DocFormat::Log,
                "test-repo",
                "main",
                "abc123",
            )
            .unwrap();

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].language, "log");
    }

    #[test]
    fn test_unsupported_format_returns_none() {
        let chunker = TextChunker::new();
        let result = chunker.process_file(
            Path::new("test.xlsx"),
            b"",
            DocFormat::Xlsx,
            "test-repo",
            "main",
            "abc123",
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_empty_content_returns_no_chunks() {
        let chunker = TextChunker::new();
        let result = chunker.process_file(
            Path::new("empty.txt"),
            b"",
            DocFormat::Text,
            "test-repo",
            "main",
            "abc123",
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_toml_chunking() {
        let chunker = TextChunker::new();
        let toml = br#"
[package]
name = "my-crate"
version = "0.1.0"

[dependencies]
serde = "1"
"#;

        let chunks = chunker
            .process_file(
                Path::new("Cargo.toml"),
                toml,
                DocFormat::Toml,
                "test-repo",
                "main",
                "abc123",
            )
            .unwrap();

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].language, "toml");
        assert!(chunks[0].code.contains("my-crate"));
    }
}
