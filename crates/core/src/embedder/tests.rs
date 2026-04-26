//! Embedder tests

#[cfg(test)]
mod tests {
    use crate::embedder::types::{Embedding, EmbedderConfig};

    #[test]
    fn test_embedding_creation() {
        let embedding = Embedding::new(vec![0.1, 0.2, 0.3], "test".to_string());
        assert_eq!(embedding.dimension(), 3);
        assert_eq!(embedding.model, "test");
    }

    #[test]
    fn test_cosine_similarity() {
        let e1 = Embedding::new(vec![1.0, 0.0, 0.0], "test".to_string());
        let e2 = Embedding::new(vec![1.0, 0.0, 0.0], "test".to_string());
        let e3 = Embedding::new(vec![0.0, 1.0, 0.0], "test".to_string());
        assert!((e1.cosine_similarity(&e2) - 1.0).abs() < 0.001);
        assert!((e1.cosine_similarity(&e3) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_default_config() {
        let config = EmbedderConfig::default();
        assert_eq!(config.model, "text-embedding-ada-002");
        assert_eq!(config.dimension, 1536);
    }
}
