//! Local store tests

#[cfg(test)]
mod tests {
    use super::super::super::client::{ChunkPayload, SearchFilter, SearchOptions};
    use super::super::store::{LocalStore, cosine_similarity_fn as cosine_similarity};
    use super::super::types::VectorPoint;
    use tempfile::TempDir;

    fn make_payload(id: &str, language: &str, file: &str) -> ChunkPayload {
        ChunkPayload {
            id: id.to_string(),
            content_hash: "hash".to_string(),
            repo: "test-repo".to_string(),
            branch: "main".to_string(),
            commit: "abc123".to_string(),
            language: language.to_string(),
            file: file.to_string(),
            module: "mod".to_string(),
            symbol: id.to_string(),
            kind: "function".to_string(),
            signature: format!("fn {}()", id),
            doc: None,
            code: format!("fn {}() {{}}", id),
            start_line: 1,
            end_line: 2,
        }
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_empty() {
        let sim = cosine_similarity(&[], &[]);
        assert_eq!(sim, 0.0);
    }

    #[test]
    fn test_cosine_similarity_different_length() {
        let sim = cosine_similarity(&[1.0], &[1.0, 0.0]);
        assert_eq!(sim, 0.0);
    }

    #[test]
    fn test_local_store_create() {
        let temp_dir = TempDir::new().unwrap();
        let store = LocalStore::open(temp_dir.path()).unwrap();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_local_store_upsert_and_search() {
        let temp_dir = TempDir::new().unwrap();
        let store = LocalStore::open(temp_dir.path()).unwrap();

        // Insert a point
        let point = VectorPoint {
            id: "point1".to_string(),
            vector: vec![1.0, 0.0, 0.0],
            payload: make_payload("point1", "rust", "lib.rs"),
        };
        store.upsert(point).unwrap();
        store.flush().unwrap();

        assert_eq!(store.len(), 1);
        assert!(store.contains("point1"));
        assert!(!store.contains("nonexistent"));
    }

    #[test]
    fn test_local_store_batch_upsert() {
        let temp_dir = TempDir::new().unwrap();
        let store = LocalStore::open(temp_dir.path()).unwrap();

        let points: Vec<VectorPoint> = (0..5)
            .map(|i| VectorPoint {
                id: format!("point{}", i),
                vector: vec![i as f32 / 5.0, (5 - i) as f32 / 5.0, 0.0],
                payload: make_payload(&format!("point{}", i), "rust", "lib.rs"),
            })
            .collect();

        store.upsert_batch(points).unwrap();
        store.flush().unwrap();

        assert_eq!(store.len(), 5);
    }

    #[test]
    fn test_local_store_search_cosine() {
        let temp_dir = TempDir::new().unwrap();
        let store = LocalStore::open(temp_dir.path()).unwrap();

        // Insert points with known vectors
        let points = vec![
            VectorPoint {
                id: "a".to_string(),
                vector: vec![1.0, 0.0, 0.0],
                payload: make_payload("a", "rust", "lib.rs"),
            },
            VectorPoint {
                id: "b".to_string(),
                vector: vec![0.9, 0.1, 0.0],
                payload: make_payload("b", "rust", "lib.rs"),
            },
            VectorPoint {
                id: "c".to_string(),
                vector: vec![0.0, 1.0, 0.0],
                payload: make_payload("c", "rust", "lib.rs"),
            },
        ];
        store.upsert_batch(points).unwrap();
        store.flush().unwrap();

        // Search for [1.0, 0.0, 0.0]
        let query = vec![1.0, 0.0, 0.0];
        let results = store
            .search(
                &query,
                SearchOptions {
                    limit: 3,
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(results.len(), 3);
        // "a" should be first (perfect match)
        assert_eq!(results[0].id, "a");
        assert!((results[0].score - 1.0).abs() < 1e-6);
        // "b" should be second (close)
        assert_eq!(results[1].id, "b");
        // "c" should be third (orthogonal)
        assert_eq!(results[2].id, "c");
        assert!(results[2].score < 0.01);
    }

    #[test]
    fn test_local_store_search_with_filter() {
        let temp_dir = TempDir::new().unwrap();
        let store = LocalStore::open(temp_dir.path()).unwrap();

        let points = vec![
            VectorPoint {
                id: "rust_fn".to_string(),
                vector: vec![1.0, 0.0],
                payload: make_payload("rust_fn", "rust", "lib.rs"),
            },
            VectorPoint {
                id: "py_fn".to_string(),
                vector: vec![0.9, 0.1],
                payload: make_payload("py_fn", "python", "main.py"),
            },
        ];
        store.upsert_batch(points).unwrap();
        store.flush().unwrap();

        let query = vec![1.0, 0.0];
        let results = store
            .search(
                &query,
                SearchOptions {
                    limit: 10,
                    filter: Some(SearchFilter {
                        language: Some("rust".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "rust_fn");
    }

    #[test]
    fn test_local_store_search_score_threshold() {
        let temp_dir = TempDir::new().unwrap();
        let store = LocalStore::open(temp_dir.path()).unwrap();

        let points = vec![
            VectorPoint {
                id: "close".to_string(),
                vector: vec![0.95, 0.05],
                payload: make_payload("close", "rust", "lib.rs"),
            },
            VectorPoint {
                id: "far".to_string(),
                vector: vec![0.1, 0.9],
                payload: make_payload("far", "rust", "lib.rs"),
            },
        ];
        store.upsert_batch(points).unwrap();
        store.flush().unwrap();

        let query = vec![1.0, 0.0];
        let results = store
            .search(
                &query,
                SearchOptions {
                    limit: 10,
                    score_threshold: Some(0.8),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "close");
    }

    #[test]
    fn test_local_store_delete() {
        let temp_dir = TempDir::new().unwrap();
        let store = LocalStore::open(temp_dir.path()).unwrap();

        let points = vec![
            VectorPoint {
                id: "keep".to_string(),
                vector: vec![1.0],
                payload: make_payload("keep", "rust", "lib.rs"),
            },
            VectorPoint {
                id: "delete".to_string(),
                vector: vec![0.5],
                payload: make_payload("delete", "rust", "lib.rs"),
            },
        ];
        store.upsert_batch(points).unwrap();
        store.flush().unwrap();

        store.delete("delete").unwrap();
        store.flush().unwrap();

        assert_eq!(store.len(), 1);
        assert!(store.contains("keep"));
        assert!(!store.contains("delete"));
    }

    #[test]
    fn test_local_store_delete_by_filter() {
        let temp_dir = TempDir::new().unwrap();
        let store = LocalStore::open(temp_dir.path()).unwrap();

        let points = vec![
            VectorPoint {
                id: "rust1".to_string(),
                vector: vec![1.0],
                payload: make_payload("rust1", "rust", "lib.rs"),
            },
            VectorPoint {
                id: "rust2".to_string(),
                vector: vec![0.9],
                payload: make_payload("rust2", "rust", "mod.rs"),
            },
            VectorPoint {
                id: "py1".to_string(),
                vector: vec![0.8],
                payload: make_payload("py1", "python", "main.py"),
            },
        ];
        store.upsert_batch(points).unwrap();
        store.flush().unwrap();

        store
            .delete_by_filter(&SearchFilter {
                language: Some("python".to_string()),
                ..Default::default()
            })
            .unwrap();
        store.flush().unwrap();

        assert_eq!(store.len(), 2);
    }

    #[test]
    fn test_local_store_persistence() {
        let temp_dir = TempDir::new().unwrap();

        // Create store, add data, flush
        {
            let store = LocalStore::open(temp_dir.path()).unwrap();
            let point = VectorPoint {
                id: "persisted".to_string(),
                vector: vec![1.0, 0.5],
                payload: make_payload("persisted", "rust", "lib.rs"),
            };
            store.upsert(point).unwrap();
            store.flush().unwrap();
        }

        // Create new store, verify data persisted
        let store2 = LocalStore::open(temp_dir.path()).unwrap();
        assert_eq!(store2.len(), 1);
        assert!(store2.contains("persisted"));
    }

    #[test]
    fn test_local_store_upsert_updates_existing() {
        let temp_dir = TempDir::new().unwrap();
        let store = LocalStore::open(temp_dir.path()).unwrap();

        let point1 = VectorPoint {
            id: "same".to_string(),
            vector: vec![1.0],
            payload: make_payload("same", "rust", "lib.rs"),
        };
        store.upsert(point1).unwrap();

        let point2 = VectorPoint {
            id: "same".to_string(),
            vector: vec![2.0],
            payload: make_payload("same", "python", "main.py"),
        };
        store.upsert(point2).unwrap();
        store.flush().unwrap();

        // Should still have only 1 point
        assert_eq!(store.len(), 1);

        // Search to verify updated vector
        let results = store
            .search(
                &vec![2.0],
                SearchOptions {
                    limit: 1,
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "same");
    }
}
