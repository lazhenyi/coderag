//! Local vector store implementation

use super::types::{VectorPoint, LocalStoreStats};
use anyhow::{Context, Result as AnyResult};
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::{Arc, RwLock};

/// Local file-based vector store
pub struct LocalStore {
    store_path: PathBuf,
    pub(crate) points: Arc<RwLock<Vec<VectorPoint>>>,
    pub(crate) dirty: Arc<RwLock<bool>>,
}

impl LocalStore {
    /// Create or open a local store at the given path
    pub fn open(path: impl AsRef<Path>) -> AnyResult<Self> {
        let path = path.as_ref().to_path_buf();
        let data_file = path.join("vectors.json");

        let points = if data_file.exists() {
            let content = fs::read_to_string(&data_file)
                .with_context(|| format!("Failed to read store file: {}", data_file.display()))?;
            serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse store file: {}", data_file.display()))?
        } else {
            Vec::new()
        };

        Ok(Self {
            store_path: path,
            points: Arc::new(RwLock::new(points)),
            dirty: Arc::new(RwLock::new(false)),
        })
    }

    /// Create store for a git repo: uses .git/embed/ for worktree, embed/ for bare
    pub fn for_repo(repo_path: &str, is_bare: bool) -> AnyResult<Self> {
        let embed_path = if is_bare {
            PathBuf::from(repo_path).join("embed")
        } else {
            PathBuf::from(repo_path).join(".git").join("embed")
        };

        if !embed_path.exists() {
            fs::create_dir_all(&embed_path)
                .with_context(|| format!("Failed to create embed directory: {}", embed_path.display()))?;
        }

        Self::open(embed_path)
    }

    /// Get the store path
    pub fn path(&self) -> &Path { &self.store_path }

    /// Upsert a single point
    pub fn upsert(&self, point: VectorPoint) -> AnyResult<()> {
        let mut points = self.points.write().unwrap();

        // Find existing point by id and update
        if let Some(existing) = points.iter_mut().find(|p| p.id == point.id) {
            *existing = point;
        } else {
            points.push(point);
        }

        let mut dirty = self.dirty.write().unwrap();
        *dirty = true;

        Ok(())
    }

    /// Upsert multiple points in batch
    pub fn upsert_batch(&self, points: Vec<VectorPoint>) -> AnyResult<()> {
        let mut all = self.points.write().unwrap();

        for new_point in points {
            if let Some(existing) = all.iter_mut().find(|p| p.id == new_point.id) {
                *existing = new_point;
            } else {
                all.push(new_point);
            }
        }

        let mut dirty = self.dirty.write().unwrap();
        *dirty = true;

        Ok(())
    }

    /// Persist to disk if dirty
    pub fn flush(&self) -> AnyResult<()> {
        let dirty = self.dirty.read().unwrap();
        if !*dirty {
            return Ok(());
        }
        drop(dirty);

        let points = self.points.read().unwrap();
        let content = serde_json::to_string_pretty(&*points)
            .context("Failed to serialize points")?;

        let data_file = self.store_path.join("vectors.json");
        fs::write(&data_file, content)
            .with_context(|| format!("Failed to write store file: {}", data_file.display()))?;

        let mut dirty = self.dirty.write().unwrap();
        *dirty = false;

        Ok(())
    }

    /// Clear all points
    pub fn clear(&self) -> AnyResult<()> {
        let mut points = self.points.write().unwrap();
        points.clear();

        let mut dirty = self.dirty.write().unwrap();
        *dirty = true;

        Ok(())
    }

    /// Get store statistics
    pub fn stats(&self) -> LocalStoreStats {
        let points = self.points.read().unwrap();
        let dim = points.first().map(|p| p.vector.len()).unwrap_or(0);
        LocalStoreStats {
            total_points: points.len(),
            vector_dimension: dim,
            file_size_bytes: self.store_path.join("vectors.json").metadata().map(|m| m.len()).unwrap_or(0),
        }
    }

    /// Get total number of points
    pub fn len(&self) -> usize {
        self.points.read().unwrap().len()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.points.read().unwrap().is_empty()
    }

    /// Check if a point with given id exists
    pub fn contains(&self, id: &str) -> bool {
        self.points.read().unwrap().iter().any(|p| p.id == id)
    }
}

impl Drop for LocalStore {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

/// Compute cosine similarity between two vectors
pub fn cosine_similarity_fn(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a * norm_b)
}
