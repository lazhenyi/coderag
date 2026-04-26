//! Search and data operations

use super::types::{ChunkPayload, QdrantClient, SearchFilter, SearchOptions, SearchResult};
use anyhow::{Context, Result as AnyResult};
use serde::{Deserialize, Serialize};

impl QdrantClient {
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
            points: points
                .into_iter()
                .map(|(id, vector, payload)| Point {
                    id,
                    vector,
                    payload,
                })
                .collect(),
        };

        let resp = self
            .add_auth(self.http.put(&self.collection_url("points")).json(&request))
            .send()
            .await;

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
                tracing::warn!(
                    "Qdrant not reachable: {}. Run 'docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant' to start.",
                    e
                );
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

        let resp = self
            .add_auth(
                self.http
                    .post(&self.collection_url("points/search"))
                    .json(&request),
            )
            .send()
            .await;

        let resp = match resp {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Qdrant not reachable: {}", e);
                return Ok(Vec::new());
            }
        };

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
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

        Ok(results
            .into_iter()
            .filter_map(|r| {
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
            })
            .collect())
    }

    /// Delete a point by ID
    pub async fn delete_point(&self, id: &str) -> AnyResult<()> {
        let url = format!("{}/{}", self.collection_url("points"), id);

        let resp = self
            .add_auth(self.http.delete(&url))
            .send()
            .await
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

        let resp = self
            .add_auth(
                self.http
                    .post(format!("{}/points/delete", self.collection_url("")))
                    .json(&request),
            )
            .send()
            .await
            .context("Failed to delete by filter")?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            tracing::warn!("Delete by filter failed: {}", body);
        }

        Ok(())
    }
}
