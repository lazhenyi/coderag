//! Text chunker tests

#[cfg(test)]
mod tests {
    use crate::document::DocFormat;
    use crate::document::text_chunker::{
        TextChunkConfig, TextChunker, strip_markdown, strip_xml_tags,
    };
    use std::path::Path;

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

        assert!(
            chunks.len() > 1,
            "Expected multiple chunks, got {}",
            chunks.len()
        );
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
