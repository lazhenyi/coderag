//! Git Repository Module
//!
//! Provides git2-based repository access for reading commits, trees, and blobs.

mod cache;
mod repo_impl;
mod tests;
mod types;

pub use cache::LruCache;
pub use git2::{Oid, Repository, Tree};
pub use repo_impl::GitRepo;
pub use types::{CommitInfo, DiffCommitsResult, FileEntry, SerializableOid};
