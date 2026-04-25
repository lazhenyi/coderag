//! Full Indexer
//!
//! Performs full repository indexing.

use crate::IndexConfig;
use crate::IndexStats;
use anyhow::{Context, Result as AnyResult};
use coderag_core::analyzer::Analyzer;
use coderag_core::chunker::{Chunk, Chunker};
use coderag_core::embedder::Embedder;
use coderag_core::embedder::EmbedderConfig;
use coderag_core::embedder::Embedding;
use coderag_core::parser::Parser;
use coderag_core::repo::GitRepo;
use coderag_storage::{QdrantClient, QdrantConfig, ChunkPayload, SearchOptions};
use std::path::Path;
use std::time::Instant;
use tracing::{info, warn};

/// Full indexer for complete repository indexing
pub struct FullIndexer {
    config: IndexConfig,
    parser: Parser,
    analyzer: Analyzer,
    chunker: Chunker,
    embedder: Option<Embedder>,
    storage: Option<QdrantClient>,
}

impl FullIndexer {
    /// Create a new full indexer
    pub fn new(config: IndexConfig) -> AnyResult<Self> {
        let parser = Parser::new()
            .context("Failed to create parser")?;

        let analyzer = Analyzer::new();
        let chunker = Chunker::new();

        let embedder = if config.qdrant_url != "" {
            let embedder_config = EmbedderConfig {
                api_url: "https://api.openai.com/v1/embeddings".to_string(),
                model: "text-embedding-ada-002".to_string(),
                dimension: 1536,
                ..Default::default()
            };
            Some(Embedder::new(embedder_config)?)
        } else {
            None
        };

        let storage = if config.qdrant_url != "" {
            let storage_config = QdrantConfig {
                url: config.qdrant_url.clone(),
                api_key: config.qdrant_api_key.clone(),
                collection_name: config.collection_name.clone(),
                ..Default::default()
            };
            Some(QdrantClient::new(storage_config)?)
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

    /// Run full indexing
    pub async fn run(&self) -> AnyResult<IndexStats> {
        let start = Instant::now();
        let mut stats = IndexStats::new();

        info!("Starting full index for repository: {}", self.config.repo_path);

        // Open repository
        let repo = GitRepo::open(&self.config.repo_path)
            .context("Failed to open repository")?;

        let head = repo.head_commit()
            .context("Failed to get HEAD commit")?;

        info!("Indexing commit: {}", head.oid);

        // Walk the tree
        let tree = repo.commit_to_tree(head.oid.inner())
            .context("Failed to get commit tree")?;

        let files = repo.walk_tree(&tree)
            .context("Failed to walk tree")?;

        info!("Found {} files to process", files.len());

        // Process files sequentially
        let symbols: Vec<_> = files
            .iter()
            .filter_map(|file| {
                // Try to detect language
                let path = Path::new(&file.path);
                let language = self.parser.detect_language(path)?;

                // Read file content
                let content = match repo.read_blob(file.blob_id.inner()) {
                    Ok(c) => c,
                    Err(e) => {
                        warn!("Failed to read {}: {}", file.path, e);
                        return None;
                    }
                };

                // Parse
                let parse_result = match self.parser.parse_with_language(&content, language.clone()) {
                    Ok(r) => r,
                    Err(e) => {
                        warn!("Failed to parse {}: {}", file.path, e);
                        return None;
                    }
                };

                // Extract symbols
                let file_symbols = self.analyzer.extract_symbols(
                    &content,
                    &parse_result.tree,
                    &language,
                    &file.path,
                );

                Some(file_symbols)
            })
            .flatten()
            .collect();

        stats.add_symbols(symbols.len());
        info!("Extracted {} symbols", stats.symbols_extracted);

        // Create chunks
        let chunks: Vec<Chunk> = self.chunker.chunk_symbols(
            &symbols,
            &self.config.repo_path,
            &self.config.branch,
            &head.oid.to_string(),
        );

        stats.add_chunks(chunks.len());
        info!("Created {} chunks", stats.chunks_created);

        // Generate embeddings and store
        if let (Some(embedder), Some(storage)) = (&self.embedder, &self.storage) {
            // Initialize collection
            let _ = storage.create_collection_if_not_exists().await;

            // Process in batches
            let batch_size = self.config.batch_size;

            for chunk_batch in chunks.chunks(batch_size) {
                // Generate embeddings
                let embeddings: Vec<Embedding> = embedder.embed_chunks(chunk_batch).await?;

                // Prepare storage batch
                let storage_batch: Vec<_> = chunk_batch
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

                // Store batch
                                let _ = storage.upsert_points_batch(storage_batch).await;

                stats.add_embeddings(chunk_batch.len());
            }

            info!("Stored {} embeddings", stats.embeddings_generated);
        }

        stats.duration_secs = start.elapsed().as_secs_f64();
        info!("Full indexing completed in {:.2}s", stats.duration_secs);

        Ok(stats)
    }

    /// Run without embedding (just extract and chunk)
    pub fn run_extract_only(&self) -> AnyResult<Vec<Chunk>> {
        let repo = GitRepo::open(&self.config.repo_path)?;

        let head = repo.head_commit()?;
        let tree = repo.commit_to_tree(head.oid.inner())?;
        let files = repo.walk_tree(&tree)?;

        let symbols: Vec<_> = files
            .iter()
            .filter_map(|file| {
                let path = Path::new(&file.path);
                let language = self.parser.detect_language(path)?;

                let content = match repo.read_blob(file.blob_id.inner()) {
                    Ok(c) => c,
                    Err(e) => {
                        warn!("Failed to read {}: {}", file.path, e);
                        return None;
                    }
                };

                let parse_result = match self.parser.parse_with_language(&content, language.clone()) {
                    Ok(r) => r,
                    Err(e) => {
                        warn!("Failed to parse {}: {}", file.path, e);
                        return None;
                    }
                };

                Some(self.analyzer.extract_symbols(
                    &content,
                    &parse_result.tree,
                    &language,
                    &file.path,
                ))
            })
            .flatten()
            .collect();

        let chunks = self.chunker.chunk_symbols(
            &symbols,
            &self.config.repo_path,
            &self.config.branch,
            &head.oid.to_string(),
        );

        Ok(chunks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_config_default() {
        let config = IndexConfig::default();
        assert_eq!(config.repo_path, ".");
        assert_eq!(config.batch_size, 100);
    }
}
