//! Full indexer implementation

use super::types::FullIndexer;
use crate::{IndexConfig, IndexStats};
use anyhow::{Context, Result as AnyResult};
use coderag_core::analyzer::Analyzer;
use coderag_core::chunker::{Chunk, Chunker};
use coderag_core::document::{detect_doc_format, TextChunker};
use coderag_core::embedder::{Embedder, EmbedderConfig};
use coderag_core::parser::Parser;
use coderag_core::repo::GitRepo;
use coderag_storage::{ChunkPayload, StorageBackend, StorageConfig, QdrantConfig};
use std::path::Path;
use std::time::Instant;
use tracing::{info, warn};

impl FullIndexer {
    /// Create a new full indexer
    pub fn new(config: IndexConfig) -> AnyResult<Self> {
        let parser = Parser::new().context("Failed to create parser")?;
        let analyzer = Analyzer::new();
        let chunker = Chunker::new();
        let text_chunker = TextChunker::new();

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
            text_chunker,
            embedder,
            storage,
        })
    }

    /// Run full indexing
    pub async fn run(&self) -> AnyResult<IndexStats> {
        let start = Instant::now();
        let mut stats = IndexStats::new();

        info!("Starting full index for repository: {}", self.config.repo_path);

        let repo = GitRepo::open(&self.config.repo_path)
            .context("Failed to open repository")?;

        let head = repo.head_commit().context("Failed to get HEAD commit")?;
        info!("Indexing commit: {}", head.oid);

        let tree = repo.commit_to_tree(head.oid.inner()).context("Failed to get commit tree")?;
        let files = repo.walk_tree(&tree).context("Failed to walk tree")?;
        info!("Found {} files to process", files.len());

        let mut all_chunks: Vec<Chunk> = Vec::new();

        for file in &files {
            let path = Path::new(&file.path);
            let content = match repo.read_blob(file.blob_id.inner()) {
                Ok(c) => c,
                Err(e) => {
                    warn!("Failed to read {}: {}", file.path, e);
                    continue;
                }
            };

            if let Some(language) = self.parser.detect_language(path) {
                let parse_result = match self.parser.parse_with_language(&content, language.clone()) {
                    Ok(r) => r,
                    Err(e) => {
                        warn!("Failed to parse {}: {}", file.path, e);
                        continue;
                    }
                };

                let file_symbols = self.analyzer.extract_symbols(
                    &content, &parse_result.tree, &language, &file.path,
                );
                let chunks = self.chunker.chunk_symbols(
                    &file_symbols, &self.config.repo_path, &self.config.branch, &head.oid.to_string(),
                );
                all_chunks.extend(chunks);
                continue;
            }

            let doc_format = detect_doc_format(path);
            if doc_format.is_supported() {
                if let Some(chunks) = self.text_chunker.process_file(
                    path, &content, doc_format, &self.config.repo_path, &self.config.branch, &head.oid.to_string(),
                ) {
                    all_chunks.extend(chunks);
                }
            }
        }

        let chunks = all_chunks;
        stats.add_chunks(chunks.len());
        info!("Created {} chunks", stats.chunks_created);

        if let (Some(embedder), Some(storage)) = (&self.embedder, &self.storage) {
            let _ = storage.create_collection_if_not_exists().await;
            let batch_size = self.config.batch_size;

            for chunk_batch in chunks.chunks(batch_size) {
                let embeddings: Vec<_> = embedder.embed_chunks(chunk_batch).await?;

                let storage_batch: Vec<_> = chunk_batch.iter().zip(embeddings.iter()).map(|(chunk, embedding)| {
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
                }).collect();

                if storage.upsert_batch(storage_batch).await.is_ok() {
                    stats.add_embeddings(chunk_batch.len());
                }
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

        let symbols: Vec<_> = files.iter().filter_map(|file| {
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

            Some(self.analyzer.extract_symbols(&content, &parse_result.tree, &language, &file.path))
        }).flatten().collect();

        let chunks = self.chunker.chunk_symbols(
            &symbols, &self.config.repo_path, &self.config.branch, &head.oid.to_string(),
        );

        Ok(chunks)
    }
}
