//! Git Repository Module
//!
//! Provides git2-based repository access for reading commits, trees, and blobs.

mod cache;

pub use cache::LruCache;
pub use git2::{Oid, Repository, Tree};

use std::collections::HashSet;
use std::path::Path;
use parking_lot::RwLock;
use anyhow::{Context, Result as AnyResult};

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

/// Git Repository wrapper providing convenient access methods
pub struct GitRepo {
    repo: Repository,
    allowed_extensions: Option<HashSet<String>>,
}

impl GitRepo {
    /// Open a repository at the given path
    pub fn open(path: impl AsRef<Path>) -> AnyResult<Self> {
        let repo = Repository::open(path)
            .context("Failed to open git repository")?;

        Ok(Self {
            repo,
            allowed_extensions: None,
        })
    }

    /// Create a bare repository
    pub fn open_bare(path: impl AsRef<Path>) -> AnyResult<Self> {
        let repo = Repository::open_bare(path)
            .context("Failed to open bare git repository")?;

        Ok(Self {
            repo,
            allowed_extensions: None,
        })
    }

    /// Set allowed file extensions for filtering (e.g., ["rs", "py", "js"])
    pub fn with_extensions(mut self, extensions: HashSet<String>) -> Self {
        self.allowed_extensions = Some(extensions);
        self
    }

    /// Get the HEAD commit
    pub fn head_commit(&self) -> AnyResult<CommitInfo> {
        let head = self.repo.head()
            .context("Failed to get HEAD reference")?;

        let commit = head.peel_to_commit()
            .context("Failed to peel HEAD to commit")?;

        let author = commit.author();
        let author_name = author.name().unwrap_or("unknown").to_string();
        let author_email = author.email().unwrap_or("").to_string();

        Ok(CommitInfo {
            oid: SerializableOid::new(commit.id()),
            message: commit.summary().unwrap_or("").to_string(),
            author_name,
            author_email,
            time: commit.time().seconds(),
        })
    }

    /// Find a commit by OID
    pub fn find_commit(&self, oid: Oid) -> AnyResult<CommitInfo> {
        let commit = self.repo.find_commit(oid)
            .context(format!("Failed to find commit {}", oid))?;

        let author = commit.author();
        let author_name = author.name().unwrap_or("unknown").to_string();
        let author_email = author.email().unwrap_or("").to_string();

        Ok(CommitInfo {
            oid: SerializableOid::new(commit.id()),
            message: commit.summary().unwrap_or("").to_string(),
            author_name,
            author_email,
            time: commit.time().seconds(),
        })
    }

    /// Get the tree for a commit
    pub fn commit_to_tree(&self, commit_oid: Oid) -> AnyResult<Tree<'_>> {
        let commit = self.repo.find_commit(commit_oid)
            .context("Failed to find commit")?;

        let tree = commit.tree()
            .context("Failed to get commit tree")?;

        // Return borrowed tree
        Ok(tree)
    }

    /// Walk a tree and return all file entries
    pub fn walk_tree(&self, tree: &Tree) -> AnyResult<Vec<FileEntry>> {
        let mut files = Vec::new();
        self.walk_tree_recursive(tree, "", &mut files)?;
        Ok(files)
    }

    fn walk_tree_recursive(&self, tree: &Tree, prefix: &str, files: &mut Vec<FileEntry>) -> AnyResult<()> {
        let entries = tree.iter()
            .collect::<Vec<_>>();

        for entry in entries {
            let name = entry.name().unwrap_or("");
            let path = if prefix.is_empty() {
                name.to_string()
            } else {
                format!("{}/{}", prefix, name)
            };

            match entry.kind() {
                Some(git2::ObjectType::Blob) => {
                    // Filter by extension if configured
                    if let Some(ref allowed) = self.allowed_extensions {
                        if let Some(ext) = std::path::Path::new(name)
                            .extension()
                            .and_then(|e| e.to_str())
                        {
                            if !allowed.contains(ext) {
                                continue;
                            }
                        } else {
                            // No extension, skip if we have extension filter
                            continue;
                        }
                    }

                    files.push(FileEntry {
                        path: path.clone(),
                        blob_id: SerializableOid::new(entry.id()),
                        file_mode: entry.filemode() as i32,
                    });
                }
                Some(git2::ObjectType::Tree) => {
                    let sub_tree = entry.to_object(&self.repo)?
                        .peel_to_tree()?;
                    self.walk_tree_recursive(&sub_tree, &path, files)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Read blob content as bytes
    pub fn read_blob(&self, oid: Oid) -> AnyResult<Vec<u8>> {
        let blob = self.repo.find_blob(oid)
            .context("Failed to find blob")?;

        let content = blob.content().to_vec();

        // Check size limit (1MB)
        if content.len() > 1024 * 1024 {
            anyhow::bail!("Blob {} exceeds 1MB size limit", oid);
        }

        Ok(content)
    }

    /// Read blob content as string (UTF-8)
    pub fn read_blob_as_string(&self, oid: Oid) -> AnyResult<String> {
        let bytes = self.read_blob(oid)?;
        String::from_utf8(bytes)
            .context("Blob is not valid UTF-8")
    }

    /// Get the repository path
    pub fn path(&self) -> &Path {
        self.repo.path()
    }

    /// Get the workdir path (None for bare repos)
    pub fn workdir(&self) -> Option<&Path> {
        self.repo.workdir()
    }

    /// List all branch names
    pub fn list_branches(&self) -> AnyResult<Vec<String>> {
        let mut branches = Vec::new();

        for branch_result in self.repo.branches(Some(git2::BranchType::Local))? {
            let (branch, _) = branch_result?;
            if let Some(name) = branch.name()? {
                branches.push(name.to_string());
            }
        }

        Ok(branches)
    }

    /// Get the default branch name (usually "main" or "master")
    pub fn get_default_branch(&self) -> AnyResult<String> {
        // Try common names first
        for name in ["main", "master"] {
            if self.repo.find_branch(name, git2::BranchType::Local).is_ok() {
                return Ok(name.to_string());
            }
        }

        // Fall back to HEAD
        Ok(self.head()?.shorthand().unwrap_or("main").to_string())
    }

    /// Get the HEAD reference
    pub fn head(&self) -> AnyResult<git2::Reference> {
        self.repo.head()
            .context("Failed to get HEAD reference")
    }

    /// Get the current branch name
    pub fn current_branch(&self) -> AnyResult<String> {
        let head = self.head()?;
        match head.shorthand() {
            Some(name) => Ok(name.to_string()),
            None => Ok("HEAD".to_string()),
        }
    }

    /// Compare two commits and return diff information
    pub fn diff_commits(&self, old_commit_oid: Oid, new_commit_oid: Oid) -> AnyResult<DiffCommitsResult> {
        let old_commit = self.repo.find_commit(old_commit_oid)
            .context("Failed to find old commit")?;
        let new_commit = self.repo.find_commit(new_commit_oid)
            .context("Failed to find new commit")?;

        let old_tree = old_commit.tree()
            .context("Failed to get old tree")?;
        let new_tree = new_commit.tree()
            .context("Failed to get new tree")?;

        let mut diff_options = git2::DiffOptions::new();
        diff_options.ignore_filemode(true);

        let mut diff = self.repo.diff_tree_to_tree(
            Some(&old_tree),
            Some(&new_tree),
            Some(&mut diff_options),
        ).context("Failed to compute diff")?;

        let mut files = Vec::new();
        let mut insertions = 0;
        let mut deletions = 0;

        diff.foreach(
            &mut |delta, _hunk| {
                files.push(unsafe {
                    std::mem::transmute(delta)
                });
                true
            },
            None,
            Some(&mut |_delta, _hunk| true),
            None,
        )?;

        let stats = diff.stats()?;
        insertions = stats.insertions() as usize;
        deletions = stats.deletions() as usize;

        Ok(DiffCommitsResult {
            old_commit: old_commit_oid,
            new_commit: new_commit_oid,
            files,
            insertions,
            deletions,
        })
    }

    /// Get diff between two trees
    pub fn diff_trees(&self, old_tree: &Tree, new_tree: &Tree) -> AnyResult<git2::Diff> {
        let mut diff_options = git2::DiffOptions::new();
        diff_options.ignore_filemode(true);

        self.repo.diff_tree_to_tree(
            Some(old_tree),
            Some(new_tree),
            Some(&mut diff_options),
        ).context("Failed to compute tree diff")
    }

    /// Get the raw git2 repository (for advanced operations)
    pub fn raw(&self) -> &Repository {
        &self.repo
    }
}

/// Result of comparing two commits
pub struct DiffCommitsResult {
    pub old_commit: Oid,
    pub new_commit: Oid,
    pub files: Vec<git2::DiffDelta<'static>>,
    pub insertions: usize,
    pub deletions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_open_current_repo() {
        // Open this very repository for testing
        let repo = GitRepo::open(".").unwrap();

        // Should be able to get head commit
        let commit = repo.head_commit().unwrap();
        println!("Current HEAD: {} - {}", commit.oid, commit.message);
    }

    #[test]
    fn test_walk_tree() {
        let repo = GitRepo::open(".").unwrap();
        let head = repo.head_commit().unwrap();
        let tree = repo.commit_to_tree(head.oid.inner()).unwrap();

        let files = repo.walk_tree(&tree).unwrap();
        println!("Found {} files", files.len());

        for file in files.iter().take(10) {
            println!("  {}", file.path);
        }
    }
}
