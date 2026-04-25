//! Diff Module
//!
//! Provides git2-based diff operations for commit comparisons.

use crate::repo::{GitRepo, Oid, SerializableOid};
use anyhow::Result as AnyResult;

/// Represents a file change with old and new blob IDs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileChange {
    pub path: String,
    pub status: FileChangeType,
    pub old_blob_id: Option<SerializableOid>,
    pub new_blob_id: Option<SerializableOid>,
}

/// Type of file change
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FileChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
}

/// Result of comparing two commits
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiffResult {
    pub old_commit: SerializableOid,
    pub new_commit: SerializableOid,
    pub changes: Vec<FileChange>,
    pub stats: DiffStats,
}

/// Statistics about the diff
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiffStats {
    pub files_added: usize,
    pub files_modified: usize,
    pub files_deleted: usize,
    pub files_renamed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

impl DiffResult {
    /// Create a diff result comparing two commits
    pub fn diff_commits(repo: &GitRepo, old_commit: Oid, new_commit: Oid) -> AnyResult<Self> {
        let diff = repo.diff_commits(old_commit, new_commit)?;

        let mut changes = Vec::new();
        let mut stats = DiffStats {
            files_added: 0,
            files_modified: 0,
            files_deleted: 0,
            files_renamed: 0,
            insertions: 0,
            deletions: 0,
        };

        for file_change in diff.files.iter() {
            let change_type = match file_change.status() {
                git2::Delta::Added => FileChangeType::Added,
                git2::Delta::Modified => FileChangeType::Modified,
                git2::Delta::Deleted => FileChangeType::Deleted,
                git2::Delta::Renamed => FileChangeType::Renamed,
                git2::Delta::Copied => FileChangeType::Copied,
                _ => continue,
            };

            let old_blob = if file_change.old_file().id() != git2::Oid::zero() {
                Some(SerializableOid::new(file_change.old_file().id()))
            } else {
                None
            };

            let new_blob = if file_change.new_file().id() != git2::Oid::zero() {
                Some(SerializableOid::new(file_change.new_file().id()))
            } else {
                None
            };

            let path = file_change.new_file().path()
                .or_else(|| file_change.old_file().path())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            match change_type {
                FileChangeType::Added => stats.files_added += 1,
                FileChangeType::Modified => stats.files_modified += 1,
                FileChangeType::Deleted => stats.files_deleted += 1,
                FileChangeType::Renamed => stats.files_renamed += 1,
                FileChangeType::Copied => {}
            }

            changes.push(FileChange {
                path,
                status: change_type,
                old_blob_id: old_blob,
                new_blob_id: new_blob,
            });
        }

        stats.insertions = diff.insertions;
        stats.deletions = diff.deletions;

        Ok(Self {
            old_commit: SerializableOid::new(old_commit),
            new_commit: SerializableOid::new(new_commit),
            changes,
            stats,
        })
    }

    /// Get files that were added
    pub fn added_files(&self) -> Vec<&FileChange> {
        self.changes.iter().filter(|c| c.status == FileChangeType::Added).collect()
    }

    /// Get files that were modified
    pub fn modified_files(&self) -> Vec<&FileChange> {
        self.changes.iter().filter(|c| c.status == FileChangeType::Modified).collect()
    }

    /// Get files that were deleted
    pub fn deleted_files(&self) -> Vec<&FileChange> {
        self.changes.iter().filter(|c| c.status == FileChangeType::Deleted).collect()
    }

    /// Get all affected file paths
    pub fn affected_paths(&self) -> Vec<&str> {
        self.changes.iter().map(|c| c.path.as_str()).collect()
    }

    /// Get files matching a predicate
    pub fn filter_files<F>(&self, predicate: F) -> Vec<&FileChange>
    where
        F: Fn(&FileChange) -> bool,
    {
        self.changes.iter().filter(|c| predicate(c)).collect()
    }
}

/// Internal diff result from git2
pub struct GitDiffResult {
    pub files: Vec<git2::DiffDelta<'static>>,
    pub insertions: usize,
    pub deletions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

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
