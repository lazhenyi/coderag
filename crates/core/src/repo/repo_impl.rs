//! Git Repository implementation

use super::types::{CommitInfo, DiffCommitsResult, FileEntry, SerializableOid};
use anyhow::{Context, Result as AnyResult};
use git2::{Oid, Repository, Tree};
use std::collections::HashSet;
use std::path::Path;

/// Git Repository wrapper
pub struct GitRepo {
    repo: Repository,
    allowed_extensions: Option<HashSet<String>>,
}

impl GitRepo {
    pub fn open(path: impl AsRef<Path>) -> AnyResult<Self> {
        let repo = Repository::open(path).context("Failed to open git repository")?;
        Ok(Self {
            repo,
            allowed_extensions: None,
        })
    }

    pub fn open_bare(path: impl AsRef<Path>) -> AnyResult<Self> {
        let repo = Repository::open_bare(path).context("Failed to open bare git repository")?;
        Ok(Self {
            repo,
            allowed_extensions: None,
        })
    }

    pub fn with_extensions(mut self, extensions: HashSet<String>) -> Self {
        self.allowed_extensions = Some(extensions);
        self
    }

    pub fn head_commit(&self) -> AnyResult<CommitInfo> {
        let head = self.repo.head().context("Failed to get HEAD reference")?;
        let commit = head
            .peel_to_commit()
            .context("Failed to peel HEAD to commit")?;
        let author = commit.author();
        Ok(CommitInfo {
            oid: SerializableOid::new(commit.id()),
            message: commit.summary().unwrap_or("").to_string(),
            author_name: author.name().unwrap_or("unknown").to_string(),
            author_email: author.email().unwrap_or("").to_string(),
            time: commit.time().seconds(),
        })
    }

    pub fn find_commit(&self, oid: Oid) -> AnyResult<CommitInfo> {
        let commit = self
            .repo
            .find_commit(oid)
            .context(format!("Failed to find commit {}", oid))?;
        let author = commit.author();
        Ok(CommitInfo {
            oid: SerializableOid::new(commit.id()),
            message: commit.summary().unwrap_or("").to_string(),
            author_name: author.name().unwrap_or("unknown").to_string(),
            author_email: author.email().unwrap_or("").to_string(),
            time: commit.time().seconds(),
        })
    }

    pub fn commit_to_tree(&self, commit_oid: Oid) -> AnyResult<Tree<'_>> {
        let commit = self
            .repo
            .find_commit(commit_oid)
            .context("Failed to find commit")?;
        commit.tree().context("Failed to get commit tree")
    }

    pub fn walk_tree(&self, tree: &Tree) -> AnyResult<Vec<FileEntry>> {
        let mut files = Vec::new();
        self.walk_tree_recursive(tree, "", &mut files)?;
        Ok(files)
    }

    fn walk_tree_recursive(
        &self,
        tree: &Tree,
        prefix: &str,
        files: &mut Vec<FileEntry>,
    ) -> AnyResult<()> {
        for entry in tree.iter() {
            let name = entry.name().unwrap_or("");
            let path = if prefix.is_empty() {
                name.to_string()
            } else {
                format!("{}/{}", prefix, name)
            };
            match entry.kind() {
                Some(git2::ObjectType::Blob) => {
                    if let Some(ref allowed) = self.allowed_extensions {
                        if let Some(ext) = Path::new(name).extension().and_then(|e| e.to_str()) {
                            if !allowed.contains(ext) {
                                continue;
                            }
                        } else {
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
                    let sub_tree = entry.to_object(&self.repo)?.peel_to_tree()?;
                    self.walk_tree_recursive(&sub_tree, &path, files)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn read_blob(&self, oid: Oid) -> AnyResult<Vec<u8>> {
        let blob = self.repo.find_blob(oid).context("Failed to find blob")?;
        let content = blob.content().to_vec();
        if content.len() > 1024 * 1024 {
            anyhow::bail!("Blob {} exceeds 1MB size limit", oid);
        }
        Ok(content)
    }

    pub fn read_blob_as_string(&self, oid: Oid) -> AnyResult<String> {
        let bytes = self.read_blob(oid)?;
        String::from_utf8(bytes).context("Blob is not valid UTF-8")
    }

    pub fn path(&self) -> &Path {
        self.repo.path()
    }
    pub fn workdir(&self) -> Option<&Path> {
        self.repo.workdir()
    }

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

    pub fn get_default_branch(&self) -> AnyResult<String> {
        for name in ["main", "master"] {
            if self.repo.find_branch(name, git2::BranchType::Local).is_ok() {
                return Ok(name.to_string());
            }
        }
        Ok(self.head()?.shorthand().unwrap_or("main").to_string())
    }

    pub fn head(&self) -> AnyResult<git2::Reference<'_>> {
        self.repo.head().context("Failed to get HEAD reference")
    }

    pub fn current_branch(&self) -> AnyResult<String> {
        let head = self.head()?;
        Ok(head.shorthand().unwrap_or("HEAD").to_string())
    }

    pub fn diff_commits(
        &self,
        old_commit_oid: Oid,
        new_commit_oid: Oid,
    ) -> AnyResult<DiffCommitsResult> {
        let old_commit = self
            .repo
            .find_commit(old_commit_oid)
            .context("Failed to find old commit")?;
        let new_commit = self
            .repo
            .find_commit(new_commit_oid)
            .context("Failed to find new commit")?;
        let old_tree = old_commit.tree().context("Failed to get old tree")?;
        let new_tree = new_commit.tree().context("Failed to get new tree")?;
        let mut diff_options = git2::DiffOptions::new();
        diff_options.ignore_filemode(true);
        let diff = self
            .repo
            .diff_tree_to_tree(Some(&old_tree), Some(&new_tree), Some(&mut diff_options))
            .context("Failed to compute diff")?;
        let mut files = Vec::new();
        diff.foreach(
            &mut |delta, _hunk| {
                files.push(unsafe { std::mem::transmute(delta) });
                true
            },
            None,
            Some(&mut |_delta, _hunk| true),
            None,
        )?;
        let stats = diff.stats()?;
        Ok(DiffCommitsResult {
            old_commit: old_commit_oid,
            new_commit: new_commit_oid,
            files,
            insertions: stats.insertions() as usize,
            deletions: stats.deletions() as usize,
        })
    }

    pub fn diff_trees(&self, old_tree: &Tree, new_tree: &Tree) -> AnyResult<git2::Diff<'_>> {
        let mut diff_options = git2::DiffOptions::new();
        diff_options.ignore_filemode(true);
        self.repo
            .diff_tree_to_tree(Some(old_tree), Some(new_tree), Some(&mut diff_options))
            .context("Failed to compute tree diff")
    }

    pub fn raw(&self) -> &Repository {
        &self.repo
    }
}
