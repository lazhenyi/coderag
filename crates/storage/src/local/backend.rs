//! Unified storage backend supporting both Qdrant and local file storage

use super::super::client::{QdrantClient, QdrantConfig, SearchOptions, SearchFilter, SearchResult, ChunkPayload};
use super::store::LocalStore;
use super::types::VectorPoint;
use anyhow::{Context, Result as AnyResult};

/// Storage backend configuration
#[derive(Debug, Clone)]
pub enum StorageConfig {
    Qdrant(QdrantConfig),
    Local {
        store_path: String,
        is_bare: bool,
    },
}

/// Unified storage backend
pub enum StorageBackend {
    Qdrant(QdrantClient),
    Local(LocalStore),
}

impl StorageBackend {
    /// Create a storage backend from config
    pub fn from_config(config: &StorageConfig) -> AnyResult<Self> {
        match config {
            StorageConfig::Qdrant(qdrant_config) => {
                let client = QdrantClient::new(qdrant_config.clone())?;
                Ok(Self::Qdrant(client))
            }
            StorageConfig::Local { store_path, is_bare } => {
                let store = LocalStore::for_repo(store_path, *is_bare)?;
                Ok(Self::Local(store))
            }
        }
    }

    /// Create local store for a specific path
    pub fn local(path: impl AsRef<std::path::Path>) -> AnyResult<Self> {
        let store = LocalStore::open(path)?;
        Ok(Self::Local(store))
    }

    /// Upsert multiple points
    pub async fn upsert_batch(&self, points: Vec<(String, Vec<f32>, ChunkPayload)>) -> AnyResult<()> {
        match self {
            Self::Qdrant(client) => client.upsert_points_batch(points).await,
            Self::Local(store) => {
                let vector_points: Vec<VectorPoint> = points.into_iter()
                    .map(|(id, vector, payload)| VectorPoint { id, vector, payload })
                    .collect();
                store.upsert_batch(vector_points)
            }
        }
    }

    /// Search for similar vectors
    pub async fn search(&self, vector: &[f32], options: SearchOptions) -> AnyResult<Vec<SearchResult>> {
        match self {
            Self::Qdrant(client) => client.search(vector, options).await,
            Self::Local(store) => {
                let results = store.search(vector, options)?;
                Ok(results.into_iter().map(|r| SearchResult {
                    id: r.id,
                    score: r.score,
                    payload: r.payload,
                }).collect())
            }
        }
    }

    /// Delete points by filter
    pub async fn delete_by_filter(&self, filter: SearchFilter) -> AnyResult<()> {
        match self {
            Self::Qdrant(client) => client.delete_by_filter(filter).await,
            Self::Local(store) => store.delete_by_filter(&filter),
        }
    }

    /// Delete a point by ID
    pub async fn delete_point(&self, id: &str) -> AnyResult<()> {
        match self {
            Self::Qdrant(client) => client.delete_point(id).await,
            Self::Local(store) => store.delete(id),
        }
    }

    /// Create collection if not exists (Qdrant only, no-op for local)
    pub async fn create_collection_if_not_exists(&self) -> AnyResult<()> {
        match self {
            Self::Qdrant(client) => client.create_collection_if_not_exists().await,
            Self::Local(_) => Ok(()),
        }
    }

    /// Health check
    pub async fn health_check(&self) -> bool {
        match self {
            Self::Qdrant(client) => client.health_check().await,
            Self::Local(store) => store.path().exists(),
        }
    }

    /// Flush pending changes (local store only)
    pub fn flush(&self) -> AnyResult<()> {
        match self {
            Self::Qdrant(_) => Ok(()),
            Self::Local(store) => store.flush(),
        }
    }

    /// Get number of stored points
    pub fn len(&self) -> usize {
        match self {
            Self::Qdrant(_) => 0,
            Self::Local(store) => store.len(),
        }
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Qdrant(_) => false,
            Self::Local(store) => store.is_empty(),
        }
    }
}
