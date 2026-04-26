//! Qdrant client implementation

use super::types::{CollectionInfo, QdrantClient, QdrantConfig};
use anyhow::{Context, Result as AnyResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};

impl QdrantClient {
    /// Create a new Qdrant client
    pub fn new(config: QdrantConfig) -> AnyResult<Self> {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { config, http })
    }

    /// Build base URL for collection
    pub(crate) fn collection_url(&self, path: &str) -> String {
        format!(
            "{}/collections/{}/{}",
            self.config.url.trim_end_matches('/'),
            self.config.collection_name,
            path
        )
    }

    /// Add authorization header if API key is set
    pub(crate) fn add_auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(ref key) = self.config.api_key {
            req.header("api-key", key)
        } else {
            req
        }
    }

    /// Create a collection if it doesn't exist
    pub async fn create_collection_if_not_exists(&self) -> AnyResult<()> {
        let url = format!(
            "{}/collections/{}",
            self.config.url.trim_end_matches('/'),
            self.config.collection_name
        );

        let resp = self.add_auth(self.http.get(&url)).send().await?;

        if resp.status().is_success() {
            tracing::info!(
                "Collection '{}' already exists",
                self.config.collection_name
            );
            return Ok(());
        }

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

        let resp = self
            .add_auth(self.http.put(&url).json(&request))
            .send()
            .await
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

        let resp = self
            .add_auth(self.http.delete(&url))
            .send()
            .await
            .context("Failed to delete collection")?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to delete collection: {}", body);
        }

        tracing::info!("Deleted collection: {}", self.config.collection_name);
        Ok(())
    }

    /// Get collection info
    pub async fn collection_info(&self) -> AnyResult<CollectionInfo> {
        let url = format!(
            "{}/collections/{}",
            self.config.url.trim_end_matches('/'),
            self.config.collection_name
        );

        let resp = self
            .add_auth(self.http.get(&url))
            .send()
            .await
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
        let url = format!(
            "{}/collections/{}",
            self.config.url, self.config.collection_name
        );
        self.add_auth(self.http.get(&url)).send().await.is_ok()
    }
}
