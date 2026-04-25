//! Qdrant Client Module
//!
//! Client for interacting with Qdrant vector database via REST API.

use anyhow::{Context, Result as AnyResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for Qdrant connection
#[derive(Debug, Clone)]
pub struct QdrantConfig {
    pub url: String,
    pub api_key: Option<String>,
    pub timeout_secs: u64,
    pub collection_name: String,
    pub vector_dimension: usize,
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:6333".to_string(),
            api_key: None,
            timeout_secs: 60,
            collection_name: "coderag".to_string(),
            vector_dimension: 1536,
        }
    }
}

/// Search result from Qdrant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub payload: ChunkPayload,
}

/// Chunk payload stored in Qdrant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkPayload {
    pub id: String,
    pub content_hash: String,
    pub repo: String,
    pub branch: String,
    pub commit: String,
    pub language: String,
    pub file: String,
    pub module: String,
    pub symbol: String,
    pub kind: String,
    pub signature: String,
    pub doc: Option<String>,
    pub code: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// Search options
#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub limit: usize,
    pub offset: Option<usize>,
    pub score_threshold: Option<f32>,
    pub filter: Option<SearchFilter>,
}

/// Search filter
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilter {
    pub language: Option<String>,
    pub file: Option<String>,
    pub kind: Option<String>,
    pub repo: Option<String>,
}

/// Qdrant client wrapper using REST API
pub struct QdrantClient {
    config: QdrantConfig,
    http: Client,
}

impl QdrantClient {
    /// Create a new Qdrant client
    pub fn new(config: QdrantConfig) -> AnyResult<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { config, http })
    }

    /// Build base URL for collection
    fn collection_url(&self, path: &str) -> String {
        format!(
            "{}/collections/{}/{}",
            self.config.url.trim_end_matches('/'),
            self.config.collection_name,
            path
        )
    }

    /// Add authorization header if API key is set
    fn add_auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(ref key) = self.config.api_key {
            req.header("api-key", key)
        } else {
            req
        }
    }

    /// Create a collection if it doesn't exist
    pub async fn create_collection_if_not_exists(&self) -> AnyResult<()> {
        // Check if collection exists
        let url = format!(
            "{}/collections/{}",
            self.config.url.trim_end_matches('/'),
            self.config.collection_name
        );

        let resp = self.add_auth(self.http.get(&url)).send().await?;

        if resp.status().is_success() {
            tracing::info!("Collection '{}' already exists", self.config.collection_name);
            return Ok(());
        }

        // Create collection
        self.create_collection().await
    }

    /// Create the collection
    pub async fn create_collection(&self) -> AnyResult<()> {
        #[derive(Serialize)]
        struct CreateCollectionRequest {
            vectors: VectorsConfig,
        }

        #[derive(Serialize)]
        struct VectorsConfig {
            size: usize,
            distance: String,
        }

        let request = CreateCollectionRequest {
            vectors: VectorsConfig {
                size: self.config.vector_dimension,
                distance: "Cosine".to_string(),
            },
        };

        let url = format!(
            "{}/collections/{}",
            self.config.url.trim_end_matches('/'),
            self.config.collection_name
        );

        let resp = self.add_auth(
            self.http.put(&url)
                .json(&request)
        ).send().await
            .context("Failed to create collection")?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to create collection: {}", body);
        }

        tracing::info!("Created collection: {}", self.config.collection_name);
        Ok(())
    }

    /// Delete a collection
    pub async fn delete_collection(&self) -> AnyResult<()> {
        let url = format!(
            "{}/collections/{}",
            self.config.url.trim_end_matches('/'),
            self.config.collection_name
        );

        let resp = self.add_auth(self.http.delete(&url)).send().await
            .context("Failed to delete collection")?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to delete collection: {}", body);
        }

        tracing::info!("Deleted collection: {}", self.config.collection_name);
        Ok(())
    }

    /// Upsert multiple points in batch
    pub async fn upsert_points_batch(
        &self,
        points: Vec<(String, Vec<f32>, ChunkPayload)>,
    ) -> AnyResult<()> {
        if points.is_empty() {
            return Ok(());
        }

        #[derive(Serialize)]
        struct UpsertRequest {
            points: Vec<Point>,
        }

        #[derive(Serialize)]
        struct Point {
            id: String,
            vector: Vec<f32>,
            payload: ChunkPayload,
        }

        let request = UpsertRequest {
            points: points.into_iter().map(|(id, vector, payload)| Point {
                id,
                vector,
                payload,
            }).collect(),
        };

        let resp = self.add_auth(
            self.http.put(&self.collection_url("points"))
                .json(&request)
        ).send().await;

        match resp {
            Ok(resp) if resp.status().is_success() => {
                tracing::debug!("Upserted batch successfully");
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                tracing::error!("Upsert failed ({}): {}", status, body);
            }
            Err(e) => {
                tracing::warn!("Qdrant not reachable: {}. Run 'docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant' to start.", e);
            }
        }

        Ok(())
    }

    /// Search for similar vectors
    pub async fn search(
        &self,
        vector: &[f32],
        options: SearchOptions,
    ) -> AnyResult<Vec<SearchResult>> {
        #[derive(Serialize)]
        struct SearchRequest<'a> {
            vector: &'a [f32],
            limit: usize,
            offset: Option<usize>,
            score_threshold: Option<f32>,
            filter: Option<SearchFilter>,
            with_payload: bool,
        }

        let request = SearchRequest {
            vector,
            limit: options.limit,
            offset: options.offset,
            score_threshold: options.score_threshold,
            filter: options.filter,
            with_payload: true,
        };

        let resp = self.add_auth(
            self.http.post(&self.collection_url("points/search"))
                .json(&request)
        ).send().await;

        let resp = match resp {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Qdrant not reachable: {}", e);
                return Ok(Vec::new());
            }
        };

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            // If collection doesn't exist, return empty results
            if body.contains("not found") || body.contains("404") {
                tracing::debug!("Collection not found, returning empty results");
                return Ok(Vec::new());
            }
            anyhow::bail!("Search failed: {}", body);
        }

        #[derive(Deserialize)]
        struct QdrantResult {
            id: serde_json::Value,
            score: f32,
            payload: Option<ChunkPayload>,
        }

        let results: Vec<QdrantResult> = resp.json().await?;

        Ok(results.into_iter().filter_map(|r| {
            let id = match r.id {
                serde_json::Value::String(s) => s,
                serde_json::Value::Number(n) => n.to_string(),
                _ => return None,
            };
            Some(SearchResult {
                id,
                score: r.score,
                payload: r.payload?,
            })
        }).collect())
    }

    /// Delete a point by ID
    pub async fn delete_point(&self, id: &str) -> AnyResult<()> {
        let url = format!("{}/{}", self.collection_url("points"), id);

        let resp = self.add_auth(self.http.delete(&url)).send().await
            .context("Failed to delete point")?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            tracing::warn!("Delete point failed: {}", body);
        }

        Ok(())
    }

    /// Delete points by filter
    pub async fn delete_by_filter(&self, filter: SearchFilter) -> AnyResult<()> {
        #[derive(Serialize)]
        struct DeleteRequest {
            filter: SearchFilter,
        }

        let request = DeleteRequest { filter };

        let resp = self.add_auth(
            self.http.post(format!("{}/points/delete", self.collection_url("")))
                .json(&request)
        ).send().await
            .context("Failed to delete by filter")?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            tracing::warn!("Delete by filter failed: {}", body);
        }

        Ok(())
    }

    /// Get collection info
    pub async fn collection_info(&self) -> AnyResult<CollectionInfo> {
        let url = format!(
            "{}/collections/{}",
            self.config.url.trim_end_matches('/'),
            self.config.collection_name
        );

        let resp = self.add_auth(self.http.get(&url)).send().await
            .context("Failed to get collection info")?;

        if !resp.status().is_success() {
            return Ok(CollectionInfo {
                points_count: 0,
                vectors_count: 0,
                status: "unknown".to_string(),
            });
        }

        #[derive(Deserialize)]
        struct CollectionResponse {
            result: CollectionResult,
        }

        #[derive(Deserialize)]
        struct CollectionResult {
            points_count: Option<u64>,
            vectors_count: Option<u64>,
            status: String,
        }

        let collection: CollectionResponse = resp.json().await?;

        Ok(CollectionInfo {
            points_count: collection.result.points_count.unwrap_or(0),
            vectors_count: collection.result.vectors_count.unwrap_or(0),
            status: collection.result.status,
        })
    }

    /// Check if client is connected and collection exists
    pub async fn health_check(&self) -> bool {
        let url = format!("{}/collections/{}", self.config.url, self.config.collection_name);
        self.add_auth(self.http.get(&url)).send().await.is_ok()
    }
}

/// Collection information
#[derive(Debug, Clone)]
pub struct CollectionInfo {
    pub points_count: u64,
    pub vectors_count: u64,
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_options_default() {
        let options = SearchOptions::default();
        assert_eq!(options.limit, 0);
        assert!(options.offset.is_none());
        assert!(options.score_threshold.is_none());
    }

    #[test]
    fn test_chunk_payload_serialization() {
        let payload = ChunkPayload {
            id: "test".to_string(),
            content_hash: "abc".to_string(),
            repo: "test-repo".to_string(),
            branch: "main".to_string(),
            commit: "123".to_string(),
            language: "rust".to_string(),
            file: "lib.rs".to_string(),
            module: "crate".to_string(),
            symbol: "hello".to_string(),
            kind: "function".to_string(),
            signature: "fn hello()".to_string(),
            doc: Some("Test doc".to_string()),
            code: "fn hello() {}".to_string(),
            start_line: 1,
            end_line: 2,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("rust"));
        assert!(json.contains("hello"));
    }
}
