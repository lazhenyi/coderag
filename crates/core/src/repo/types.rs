//! Repository types

use git2::Oid;

/// Oid wrapper that supports serialization
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SerializableOid(Oid);

impl SerializableOid {
    pub fn new(oid: Oid) -> Self {
        Self(oid)
    }
    pub fn inner(&self) -> Oid {
        self.0
    }
}

impl From<Oid> for SerializableOid {
    fn from(oid: Oid) -> Self {
        Self(oid)
    }
}

impl std::fmt::Display for SerializableOid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl serde::Serialize for SerializableOid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for SerializableOid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let oid = Oid::from_str(&s).map_err(serde::de::Error::custom)?;
        Ok(Self(oid))
    }
}

/// Commit information extracted from git2
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommitInfo {
    pub oid: SerializableOid,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
    pub time: i64,
}

/// File entry representing a blob in a tree
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub blob_id: SerializableOid,
    pub file_mode: i32,
}

/// Result of comparing two commits
pub struct DiffCommitsResult {
    pub old_commit: Oid,
    pub new_commit: Oid,
    pub files: Vec<git2::DiffDelta<'static>>,
    pub insertions: usize,
    pub deletions: usize,
}
