//! Indexer State Module
//!
//! Manages persistent state for incremental indexing.

use anyhow::{Context, Result as AnyResult};
use coderag_core::repo::Oid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::RwLock;

/// Indexer state for tracking indexed commits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerState {
    /// Repository path -> last indexed commit
    pub last_indexed: HashMap<String, String>,
    /// Repository path -> last indexed time
    pub last_indexed_time: HashMap<String, u64>,
}

impl Default for IndexerState {
    fn default() -> Self {
        Self {
            last_indexed: HashMap::new(),
            last_indexed_time: HashMap::new(),
        }
    }
}

/// Manager for indexer state persistence
pub struct StateManager {
    state_file: PathBuf,
    state: RwLock<IndexerState>,
}

impl StateManager {
    /// Create a new state manager
    pub fn new(state_file: impl AsRef<Path>) -> AnyResult<Self> {
        let state_file = state_file.as_ref().to_path_buf();
        let state = if state_file.exists() {
            let content = fs::read_to_string(&state_file).context("Failed to read state file")?;
            serde_json::from_str(&content).context("Failed to parse state file")?
        } else {
            IndexerState::default()
        };

        Ok(Self {
            state_file,
            state: RwLock::new(state),
        })
    }

    /// Get the last indexed commit for a repository
    pub fn get_last_commit(&self, repo_path: &str) -> Option<Oid> {
        let state = self.state.read().unwrap();
        state
            .last_indexed
            .get(repo_path)
            .and_then(|s| Oid::from_str(s).ok())
    }

    /// Set the last indexed commit for a repository
    pub fn set_last_commit(&self, repo_path: &str, commit: Oid) -> AnyResult<()> {
        {
            let mut state = self.state.write().unwrap();
            state
                .last_indexed
                .insert(repo_path.to_string(), commit.to_string());
            state.last_indexed_time.insert(
                repo_path.to_string(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );
        }
        self.save()
    }

    /// Save state to disk
    pub fn save(&self) -> AnyResult<()> {
        let state = self.state.read().unwrap();
        let content = serde_json::to_string_pretty(&*state).context("Failed to serialize state")?;

        // Create parent directories if needed
        if let Some(parent) = self.state_file.parent() {
            fs::create_dir_all(parent).context("Failed to create state directory")?;
        }

        fs::write(&self.state_file, content).context("Failed to write state file")?;

        Ok(())
    }

    /// Clear state for a repository
    pub fn clear(&self, repo_path: &str) -> AnyResult<()> {
        {
            let mut state = self.state.write().unwrap();
            state.last_indexed.remove(repo_path);
            state.last_indexed_time.remove(repo_path);
        }
        self.save()
    }
}

use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_state_manager() -> AnyResult<()> {
        let temp_dir = TempDir::new()?;
        let state_file = temp_dir.path().join("state.json");

        let manager = StateManager::new(&state_file)?;

        // Initially no state
        assert!(manager.get_last_commit("test-repo").is_none());

        // Set a commit
        let commit = Oid::from_str("abc123def456")?;
        manager.set_last_commit("test-repo", commit)?;

        // Verify it was saved
        let _loaded_manager = StateManager::new(&state_file)?;
        assert_eq!(manager.get_last_commit("test-repo"), Some(commit));

        Ok(())
    }
}
