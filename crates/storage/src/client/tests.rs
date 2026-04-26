//! Client tests

#[cfg(test)]
mod tests {
    use super::super::types::*;

    #[test]
    fn test_search_options_default() {
        let options = SearchOptions::default();
        assert_eq!(options.limit, 0);
        assert!(options.offset.is_none());
        assert!(options.score_threshold.is_none());
    }

    #[test]
    fn test_chunk_payload_serialization() {
        let payload = ChunkPayload {
            id: "test".to_string(),
            content_hash: "abc".to_string(),
            repo: "test-repo".to_string(),
            branch: "main".to_string(),
            commit: "123".to_string(),
            language: "rust".to_string(),
            file: "lib.rs".to_string(),
            module: "crate".to_string(),
            symbol: "hello".to_string(),
            kind: "function".to_string(),
            signature: "fn hello()".to_string(),
            doc: Some("Test doc".to_string()),
            code: "fn hello() {}".to_string(),
            start_line: 1,
            end_line: 2,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("rust"));
        assert!(json.contains("hello"));
    }
}
