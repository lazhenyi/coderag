//! Chunker tests

#[cfg(test)]
mod tests {
    use crate::analyzer::symbol::{Symbol, SymbolKind};
    use crate::chunker::{Chunker, Chunk, chunk_to_embedding_text};

    #[test]
    fn test_chunk_creation() {
        let chunker = Chunker::new();
        let symbol = Symbol::new("hello".to_string(), SymbolKind::Function, 1, 5, "rust".to_string())
            .with_file_path("src/lib.rs".to_string())
            .with_module_path("crate".to_string())
            .with_signature("fn hello()".to_string())
            .with_code("fn hello() {\n    println!(\"Hello!\");\n}".to_string())
            .with_doc(Some("Says hello".to_string()));
        let chunks = chunker.chunk_symbol(&symbol, "test-repo", "main", "abc123");
        assert_eq!(chunks.len(), 2);
        let code_chunk = &chunks[0];
        assert_eq!(code_chunk.symbol, "hello");
        assert_eq!(code_chunk.kind, "function");
        assert_eq!(code_chunk.language, "rust");
        assert!(!code_chunk.content_hash.is_empty());
    }

    #[test]
    fn test_content_hash() {
        let chunker = Chunker::new();
        let symbol = Symbol::new("test".to_string(), SymbolKind::Function, 1, 10, "rust".to_string())
            .with_code("fn test() {}".to_string())
            .with_signature("fn test()".to_string())
            .with_file_path("test.rs".to_string())
            .with_module_path("".to_string());
        let chunks = chunker.chunk_symbol(&symbol, "repo", "main", "commit");
        let chunks2 = chunker.chunk_symbol(&symbol, "repo", "main", "commit");
        assert_eq!(chunks[0].content_hash, chunks2[0].content_hash);
        let mut symbol2 = symbol.clone();
        symbol2.code = "fn test2() {}".to_string();
        let chunks3 = chunker.chunk_symbol(&symbol2, "repo", "main", "commit");
        assert_ne!(chunks[0].content_hash, chunks3[0].content_hash);
    }

    #[test]
    fn test_embedding_text() {
        let chunk = Chunk {
            id: "test".to_string(), content_hash: "abc".to_string(),
            repo: "test-repo".to_string(), branch: "main".to_string(), commit: "123".to_string(),
            language: "rust".to_string(), file: "lib.rs".to_string(), module: "crate".to_string(),
            symbol: "hello".to_string(), kind: "function".to_string(),
            signature: "fn hello() -> String".to_string(), doc: Some("Returns greeting".to_string()),
            code: "fn hello() -> String { \"Hello\".to_string() }".to_string(),
            start_line: 1, end_line: 2,
        };
        let text = chunk_to_embedding_text(&chunk);
        assert!(text.starts_with("[rust]"));
        assert!(text.contains("fn hello()"));
        assert!(text.contains("Returns greeting"));
    }
}
