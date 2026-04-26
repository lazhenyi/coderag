//! Full indexer tests

#[cfg(test)]
mod tests {
    use crate::IndexConfig;

    #[test]
    fn test_index_config_default() {
        let config = IndexConfig::default();
        assert_eq!(config.repo_path, ".");
        assert_eq!(config.batch_size, 100);
    }
}
