//! Diff types

/// Type of file change
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FileChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
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

/// Represents a file change with old and new blob IDs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileChange {
    pub path: String,
    pub status: FileChangeType,
    pub old_blob_id: Option<crate::repo::SerializableOid>,
    pub new_blob_id: Option<crate::repo::SerializableOid>,
}

/// Result of comparing two commits
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiffResult {
    pub old_commit: crate::repo::SerializableOid,
    pub new_commit: crate::repo::SerializableOid,
    pub changes: Vec<FileChange>,
    pub stats: DiffStats,
}
