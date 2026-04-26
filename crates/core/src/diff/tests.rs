//! Diff tests

#[cfg(test)]
mod tests {
    use super::super::types::{DiffResult, FileChange, FileChangeType, DiffStats};
    use crate::repo::{Oid, SerializableOid};

    #[test]
    fn test_diff_structs() {
        let change = FileChange {
            path: "test.rs".to_string(),
            status: FileChangeType::Modified,
            old_blob_id: Some(SerializableOid::new(Oid::from_bytes(&[0u8; 20]).unwrap())),
            new_blob_id: Some(SerializableOid::new(Oid::from_bytes(&[1u8; 20]).unwrap())),
        };

        let json = serde_json::to_string_pretty(&change).unwrap();
        println!("{}", json);
    }

    #[test]
    fn test_diff_result_filters() {
        let result = DiffResult {
            old_commit: SerializableOid::new(Oid::from_bytes(&[0u8; 20]).unwrap()),
            new_commit: SerializableOid::new(Oid::from_bytes(&[1u8; 20]).unwrap()),
            changes: vec![
                FileChange {
                    path: "added.rs".to_string(),
                    status: FileChangeType::Added,
                    old_blob_id: None,
                    new_blob_id: Some(SerializableOid::new(Oid::from_bytes(&[2u8; 20]).unwrap())),
                },
                FileChange {
                    path: "modified.rs".to_string(),
                    status: FileChangeType::Modified,
                    old_blob_id: Some(SerializableOid::new(Oid::from_bytes(&[0u8; 20]).unwrap())),
                    new_blob_id: Some(SerializableOid::new(Oid::from_bytes(&[1u8; 20]).unwrap())),
                },
                FileChange {
                    path: "deleted.rs".to_string(),
                    status: FileChangeType::Deleted,
                    old_blob_id: Some(SerializableOid::new(Oid::from_bytes(&[0u8; 20]).unwrap())),
                    new_blob_id: None,
                },
            ],
            stats: DiffStats {
                files_added: 1,
                files_modified: 1,
                files_deleted: 1,
                files_renamed: 0,
                insertions: 10,
                deletions: 5,
            },
        };

        assert_eq!(result.added_files().len(), 1);
        assert_eq!(result.modified_files().len(), 1);
        assert_eq!(result.deleted_files().len(), 1);
    }
}
