//! Index command handler

use crate::config::Config;
use anyhow::{Context, Result as AnyResult};
use coderag_core::repo::GitRepo;
use coderag_indexer::{FullIndexer, IncrementalIndexer, IndexConfig};
use tracing::info;

pub async fn index_repository(repo_path: &str, branch: &str, full: bool, use_local: bool, cli_config: &Config) -> AnyResult<()> {
    info!("Indexing repository: {} (branch: {})", repo_path, branch);

    let state_file_path = cli_config.state_file.clone()
        .unwrap_or_else(|| ".coderag/state.json".to_string());

    let config = IndexConfig {
        repo_path: repo_path.to_string(),
        branch: branch.to_string(),
        qdrant_url: cli_config.qdrant_url.clone()
            .unwrap_or_else(|| "http://localhost:6333".to_string()),
        qdrant_api_key: cli_config.qdrant_api_key.clone(),
        collection_name: cli_config.collection_name.clone()
            .unwrap_or_else(|| "coderag".to_string()),
        batch_size: cli_config.batch_size.unwrap_or(100),
        embed_api_url: cli_config.embed_url.clone(),
        embed_api_key: cli_config.openai_api_key.clone(),
        embed_model: cli_config.embed_model.clone(),
        embed_dimension: cli_config.embed_dimension,
        use_local_storage: use_local,
        ..Default::default()
    };

    if full {
        let indexer = FullIndexer::new(config)?;
        let stats = indexer.run().await?;

        println!("\nIndexing complete!");
        println!("  Files processed: {}", stats.files_processed);
        println!("  Symbols extracted: {}", stats.symbols_extracted);
        println!("  Chunks created: {}", stats.chunks_created);
        println!("  Embeddings generated: {}", stats.embeddings_generated);
        println!("  Duration: {:.2}s", stats.duration_secs);

        if let Ok(state) = coderag_indexer::StateManager::new(&state_file_path) {
            let repo = GitRepo::open(repo_path)?;
            let head = repo.head_commit()?;
            if state.set_last_commit(repo_path, head.oid.inner()).is_ok() {
                println!("  State updated: {}", head.oid);
            }
        }
    } else {
        let repo = GitRepo::open(repo_path).context("Failed to open repository")?;
        let head = repo.head_commit().context("Failed to get HEAD commit")?;

        let state = coderag_indexer::StateManager::new(&state_file_path)
            .context("Failed to create state manager")?;

        let old_commit = state.get_last_commit(repo_path);
        let old_oid = old_commit.unwrap_or(head.oid.inner());

        if let Some(ref old) = old_commit {
            println!("Incremental index from: {} -> {}", old, head.oid);
        } else {
            println!("No previous index found, running full index instead...");
            println!("  (Run with --full to force full reindex)");
            let indexer = FullIndexer::new(config)?;
            let stats = indexer.run().await?;
            println!("\nIndexing complete!");
            println!("  Symbols extracted: {}", stats.symbols_extracted);
            return Ok(());
        }

        let indexer = IncrementalIndexer::new(config)?;
        let stats = indexer.run(old_oid, head.oid.inner()).await?;

        println!("\nIncremental indexing complete!");
        println!("  Files processed: {}", stats.files_processed);
        println!("  Chunks created: {}", stats.chunks_created);
        println!("  Duration: {:.2}s", stats.duration_secs);

        if state.set_last_commit(repo_path, head.oid.inner()).is_ok() {
            println!("  State updated: {}", head.oid);
        }
    }

    Ok(())
}
