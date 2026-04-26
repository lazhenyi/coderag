//! Incremental Indexer
//!
//! Performs incremental repository indexing based on git diffs.

use crate::{IndexConfig, IndexStats};
use anyhow::{Context, Result as AnyResult};
use coderag_core::analyzer::Analyzer;
use coderag_core::chunker::{Chunk, Chunker};
use coderag_core::diff::{DiffResult, FileChangeType};
use coderag_core::embedder::Embedder;
use coderag_core::embedder::EmbedderConfig;
use coderag_core::embedder::Embedding;
use coderag_core::parser::Parser;
use coderag_core::repo::{GitRepo, Oid};
use coderag_storage::{ChunkPayload, QdrantConfig, SearchFilter, StorageBackend, StorageConfig};
use std::path::Path;
use std::time::Instant;
use tracing::{info, warn};

/// Incremental indexer for delta indexing
pub struct IncrementalIndexer {
    config: IndexConfig,
    parser: Parser,
    analyzer: Analyzer,
    chunker: Chunker,
    embedder: Option<Embedder>,
    storage: Option<StorageBackend>,
}

impl IncrementalIndexer {
    /// Create a new incremental indexer
    pub fn new(config: IndexConfig) -> AnyResult<Self> {
        let parser = Parser::new()
            .context("Failed to create parser")?;

        let analyzer = Analyzer::new();
        let chunker = Chunker::new();

        let embedder = if config.qdrant_url != "" || config.use_local_storage {
            let embedder_config = EmbedderConfig {
                api_url: config.embed_api_url.clone()
                    .unwrap_or_else(|| "https://dashscope.aliyuncs.com/compatible-mode/v1/embeddings".to_string()),
                model: config.embed_model.clone()
                    .unwrap_or_else(|| "text-embedding-v4".to_string()),
                dimension: config.embed_dimension.unwrap_or(1024),
                api_key: config.embed_api_key.clone(),
                ..Default::default()
            };
            Some(Embedder::new(embedder_config)?)
        } else {
            None
        };

        let storage = if config.use_local_storage {
            Some(StorageBackend::from_config(&StorageConfig::Local {
                store_path: config.repo_path.clone(),
                is_bare: false,
            })?)
        } else if config.qdrant_url != "" {
            let storage_config = QdrantConfig {
                url: config.qdrant_url.clone(),
                api_key: config.qdrant_api_key.clone(),
                collection_name: config.collection_name.clone(),
                ..Default::default()
            };
            Some(StorageBackend::from_config(&StorageConfig::Qdrant(storage_config))?)
        } else {
            None
        };

        Ok(Self {
            config,
            parser,
            analyzer,
            chunker,
            embedder,
            storage,
        })
    }

    /// Run incremental indexing between two commits
    pub async fn run(&self, old_commit: Oid, new_commit: Oid) -> AnyResult<IndexStats> {
        let start = Instant::now();
        let mut stats = IndexStats::new();

        info!("Starting incremental index: {} -> {}", old_commit, new_commit);

        // Open repository
        let repo = GitRepo::open(&self.config.repo_path)
            .context("Failed to open repository")?;

        // Compute diff
        let diff = DiffResult::diff_commits(&repo, old_commit, new_commit)
            .context("Failed to compute diff")?;

        info!(
            "Diff: {} added, {} modified, {} deleted",
            diff.stats.files_added,
            diff.stats.files_modified,
            diff.stats.files_deleted
        );

        // Process modified and added files
        let mut chunks = Vec::new();

        for change in diff.changes.iter() {
            match change.status {
                FileChangeType::Added | FileChangeType::Modified => {
                    if let Some(new_blob) = &change.new_blob_id {
                        if let Some(chunk) = self.process_file(&repo, &change.path, new_blob.inner()) {
                            chunks.extend(chunk);
                        }
                    }
                }
                FileChangeType::Deleted => {
                    // Delete chunks for this file from storage
                    if let Some(ref storage) = self.storage {
                        let _ = storage.delete_by_filter(SearchFilter {
                            file: Some(change.path.clone()),
                            ..Default::default()
                        }).await;
                        info!("Deleted chunks for deleted file: {}", change.path);
                    }
                }
                _ => {}
            }
        }

        stats.files_processed = diff.changes.iter()
            .filter(|c| matches!(c.status, FileChangeType::Added | FileChangeType::Modified))
            .count();

        stats.add_chunks(chunks.len());

        // Store new chunks
        if let (Some(embedder), Some(storage)) = (&self.embedder, &self.storage) {
            let embeddings: Vec<Embedding> = embedder.embed_chunks(&chunks).await?;

            let storage_batch: Vec<_> = chunks
                .iter()
                .zip(embeddings.iter())
                .map(|(chunk, embedding)| {
                    let payload = ChunkPayload {
                        id: chunk.id.clone(),
                        content_hash: chunk.content_hash.clone(),
                        repo: chunk.repo.clone(),
                        branch: chunk.branch.clone(),
                        commit: chunk.commit.clone(),
                        language: chunk.language.clone(),
                        file: chunk.file.clone(),
                        module: chunk.module.clone(),
                        symbol: chunk.symbol.clone(),
                        kind: chunk.kind.clone(),
                        signature: chunk.signature.clone(),
                        doc: chunk.doc.clone(),
                        code: chunk.code.clone(),
                        start_line: chunk.start_line,
                        end_line: chunk.end_line,
                    };

                    (chunk.id.clone(), embedding.vector.clone(), payload)
                })
                .collect();

            if storage.upsert_batch(storage_batch).await.is_ok() {
                stats.add_embeddings(chunks.len());
            }
        }

        stats.duration_secs = start.elapsed().as_secs_f64();
        info!("Incremental indexing completed in {:.2}s", stats.duration_secs);

        Ok(stats)
    }

    /// Process a single file
    fn process_file(&self, repo: &GitRepo, path: &str, blob_id: Oid) -> Option<Vec<Chunk>> {
        let file_path = Path::new(path);

        // Check if file has a supported language
        let language = self.parser.detect_language(file_path)?;

        // Read content
        let content = match repo.read_blob(blob_id) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to read {}: {}", path, e);
                return None;
            }
        };

        // Parse
        let parse_result = match self.parser.parse_with_language(&content, language.clone()) {
            Ok(r) => r,
            Err(e) => {
                warn!("Failed to parse {}: {}", path, e);
                return None;
            }
        };

        // Extract symbols
        let symbols = self.analyzer.extract_symbols(
            &content,
            &parse_result.tree,
            &language,
            path,
        );

        // Create chunks
        let head = repo.head_commit().ok()?;
        let chunks = self.chunker.chunk_symbols(
            &symbols,
            &self.config.repo_path,
            &self.config.branch,
            &head.oid.to_string(),
        );

        Some(chunks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incremental_config() {
        let config = IndexConfig::default();
        assert_eq!(config.batch_size, 100);
    }
}
