//! CodeRAG CLI
//!
//! Command-line interface for the CodeRAG code indexing system.

use anyhow::{Context, Result as AnyResult};
use clap::{Parser, Subcommand};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use coderag_core::repo::GitRepo;
use coderag_indexer::{FullIndexer, IncrementalIndexer, IndexConfig, evaluate, default_queries, print_report};

mod config;
use config::Config;

#[derive(Parser)]
#[command(name = "coderag")]
#[command(about = "AST-based code RAG system with multi-language support")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Repository path (default: current directory)
    #[arg(short, long, default_value = ".")]
    repo: String,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Index a repository (full or incremental)
    Index {
        /// Run full indexing (reindex everything)
        #[arg(short, long)]
        full: bool,

        /// Branch to index
        #[arg(short, long, default_value = "main")]
        branch: String,
    },

    /// Search for code
    Search {
        /// Search query
        query: String,

        /// Language filter
        #[arg(short, long)]
        language: Option<String>,

        /// Maximum results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Show repository information
    Info {
        /// Show file tree
        #[arg(short, long)]
        tree: bool,
    },

    /// Show supported languages
    Languages,

    /// Run quality evaluation (Top1/Top5/MRR)
    Eval {
        /// Language to evaluate (default: all languages with queries)
        #[arg(short, long)]
        language: Option<String>,

        /// Qdrant URL (default: QDRANT_URL env or localhost:6333)
        #[arg(short, long)]
        qdrant_url: Option<String>,
    },
}

#[tokio::main]
async fn main() -> AnyResult<()> {
    let cli = Cli::parse();

    // Initialize logging
    let level = match cli.log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    // Load configuration (env vars > .env > ~/.config/coderag/settings.json)
    let config = Config::load()?;

    match cli.command {
        Commands::Index { full, branch } => {
            index_repository(&cli.repo, &branch, full, &config).await?;
        }
        Commands::Search { query, language, limit } => {
            search_code(&cli.repo, &query, language.as_deref(), limit, &config).await?;
        }
        Commands::Info { tree } => {
            show_info(&cli.repo, tree)?;
        }
        Commands::Languages => {
            show_languages()?;
        }
        Commands::Eval { language, qdrant_url } => {
            eval_quality(&cli.repo, language.as_deref(), qdrant_url.as_deref(), &config).await?;
        }
    }

    Ok(())
}

async fn index_repository(repo_path: &str, branch: &str, full: bool, cli_config: &Config) -> AnyResult<()> {
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

        // Update state after successful full index
        if let Ok(state) = coderag_indexer::StateManager::new(&state_file_path) {
            let repo = GitRepo::open(repo_path)?;
            let head = repo.head_commit()?;
            if state.set_last_commit(repo_path, head.oid.inner()).is_ok() {
                println!("  State updated: {}", head.oid);
            }
        }
    } else {
        // Incremental indexing - load state
        let repo = GitRepo::open(repo_path)
            .context("Failed to open repository")?;

        let head = repo.head_commit()
            .context("Failed to get HEAD commit")?;

        // Use StateManager to track last indexed commit
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

        // Update state after successful incremental index
        if state.set_last_commit(repo_path, head.oid.inner()).is_ok() {
            println!("  State updated: {}", head.oid);
        }
    }

    Ok(())
}

async fn search_code(
    repo_path: &str,
    query: &str,
    language: Option<&str>,
    limit: usize,
    config: &Config,
) -> AnyResult<()> {
    println!("Searching for: {}", query);

    let qdrant_url = config.qdrant_url.clone()
        .unwrap_or_else(|| "http://localhost:6333".to_string());
    let qdrant_api_key = config.qdrant_api_key.clone();
    let openai_api_key = config.openai_api_key.clone();

    // Check Qdrant connection
    let storage_config = coderag_storage::QdrantConfig {
        url: qdrant_url.clone(),
        api_key: qdrant_api_key.clone(),
        collection_name: config.collection_name.clone()
            .unwrap_or_else(|| "coderag".to_string()),
        ..Default::default()
    };

    let storage = coderag_storage::QdrantClient::new(storage_config)
        .context("Failed to create Qdrant client")?;

    // Check if collection exists
    if !storage.health_check().await {
        println!("\nError: Cannot connect to Qdrant at {}", qdrant_url);
        println!("Make sure Qdrant is running (docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant)");
        return Ok(());
    }

    // Generate embedding for query
    let embedder_config = coderag_core::embedder::EmbedderConfig {
        api_url: config.embed_url.clone()
            .unwrap_or_else(|| "https://dashscope.aliyuncs.com/compatible-mode/v1/embeddings".to_string()),
        model: config.embed_model.clone()
            .unwrap_or_else(|| "text-embedding-v4".to_string()),
        dimension: config.embed_dimension.unwrap_or(1024),
        api_key: openai_api_key,
        ..Default::default()
    };

    let embedder = coderag_core::embedder::Embedder::new(embedder_config)
        .context("Failed to create embedder")?;

    println!("\nGenerating query embedding...");

    // Create a dummy chunk for embedding the query
    let query_chunk = coderag_core::chunker::Chunk {
        id: "query".to_string(),
        content_hash: "".to_string(),
        repo: repo_path.to_string(),
        branch: "main".to_string(),
        commit: "".to_string(),
        language: language.unwrap_or("").to_string(),
        file: "".to_string(),
        module: "".to_string(),
        symbol: "".to_string(),
        kind: "query".to_string(),
        signature: query.to_string(),
        doc: None,
        code: query.to_string(),
        start_line: 0,
        end_line: 0,
    };

    let embedding = match embedder.embed_chunk(&query_chunk).await {
        Ok(e) => e,
        Err(e) => {
            println!("\nError: Failed to generate embedding: {}", e);
            println!("Check your OPENAI_API_KEY environment variable.");
            return Ok(());
        }
    };

    // Search Qdrant
    let filter = coderag_storage::SearchFilter {
        language: language.map(String::from),
        ..Default::default()
    };

    let options = coderag_storage::SearchOptions {
        limit,
        score_threshold: Some(0.5),
        filter: if filter.language.is_some() || filter.file.is_some() || filter.kind.is_some() {
            Some(filter)
        } else {
            None
        },
        ..Default::default()
    };

    let results = storage.search(&embedding.vector, options).await?;

    if results.is_empty() {
        println!("\nNo results found. Try indexing the repository first:");
        println!("  coderag index --repo {} --full", repo_path);
        return Ok(());
    }

    // Display results
    println!("\nFound {} results:\n", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("{}. [{}] {:.2}", i + 1, result.payload.kind, result.score);
        println!("   Symbol: {}", result.payload.symbol);
        println!("   File: {}", result.payload.file);
        if let Some(ref doc) = result.payload.doc {
            if !doc.is_empty() {
                let doc_short = if doc.len() > 100 { &doc[..100] } else { doc };
                println!("   Doc: {}", doc_short);
            }
        }
        if !result.payload.signature.is_empty() && result.payload.signature != result.payload.symbol {
            println!("   Signature: {}", result.payload.signature);
        }
        println!("   Code: {}", result.payload.code.lines().next().unwrap_or(""));
        println!();
    }

    Ok(())
}

fn show_info(repo_path: &str, show_tree: bool) -> AnyResult<()> {
    let repo = GitRepo::open(repo_path)
        .context("Failed to open repository")?;

    let head = repo.head_commit()?;
    println!("Repository: {}", repo_path);
    println!("Current branch: {}", repo.current_branch()?);
    println!("HEAD: {} ({})", head.oid, head.message);

    if show_tree {
        let tree = repo.commit_to_tree(head.oid.inner())?;
        let files = repo.walk_tree(&tree)?;

        println!("\nFile tree ({} files):", files.len());
        for file in files.iter().take(50) {
            println!("  {}", file.path);
        }
        if files.len() > 50 {
            println!("  ... and {} more", files.len() - 50);
        }
    }

    Ok(())
}

async fn eval_quality(
    repo_path: &str,
    language: Option<&str>,
    qdrant_url: Option<&str>,
    config: &Config,
) -> AnyResult<()> {
    let qdrant_url = qdrant_url
        .map(String::from)
        .or_else(|| config.qdrant_url.clone())
        .unwrap_or_else(|| "http://localhost:6333".to_string());

    let openai_api_key = config.openai_api_key.clone();

    // Check Qdrant connection
    let storage_config = coderag_storage::QdrantConfig {
        url: qdrant_url.clone(),
        api_key: config.qdrant_api_key.clone(),
        collection_name: config.collection_name.clone()
            .unwrap_or_else(|| "coderag".to_string()),
        ..Default::default()
    };

    let storage = coderag_storage::QdrantClient::new(storage_config)
        .context("Failed to create Qdrant client")?;

    if !storage.health_check().await {
        println!("Error: Cannot connect to Qdrant at {}", qdrant_url);
        println!("Make sure Qdrant is running:");
        println!("  docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant");
        return Ok(());
    }

    // Create embedder
    let embedder_config = coderag_core::embedder::EmbedderConfig {
        api_url: config.embed_url.clone()
            .unwrap_or_else(|| "https://dashscope.aliyuncs.com/compatible-mode/v1/embeddings".to_string()),
        model: config.embed_model.clone()
            .unwrap_or_else(|| "text-embedding-v4".to_string()),
        dimension: config.embed_dimension.unwrap_or(1024),
        api_key: openai_api_key,
        ..Default::default()
    };

    let embedder = coderag_core::embedder::Embedder::new(embedder_config)
        .context("Failed to create embedder")?;

    // Load queries
    let all_queries = default_queries();
    let queries: Vec<_> = if let Some(lang) = language {
        let lang_lower = lang.to_lowercase();
        all_queries.into_iter()
            .filter(|q| q.language.to_lowercase() == lang_lower)
            .collect()
    } else {
        all_queries
    };

    if queries.is_empty() {
        println!("No evaluation queries for language: {:?}", language);
        return Ok(());
    }

    println!("Running quality evaluation ({} queries)...", queries.len());

    let metrics = evaluate(&queries, &storage, &embedder).await?;
    print_report(&metrics);

    println!();
    println!("To improve results:");
    println!("  1. Index your repository: coderag index --repo {} --full", repo_path);
    println!("  2. Use a real OpenAI API key (export OPENAI_API_KEY=...)");
    println!("  3. Run: coderag eval --repo {} --language <lang>", repo_path);

    Ok(())
}

fn show_languages() -> AnyResult<()> {
    println!("Supported languages (AST extraction):");
    println!();
    println!("  Rust           (.rs)              - function, struct, enum, trait, impl, module");
    println!("  Python         (.py, .pyw)        - function, class, method");
    println!("  JavaScript     (.js, .jsx, .mjs)  - function, class, method");
    println!("  TypeScript     (.ts, .tsx, .mts)  - function, class, interface, method");
    println!("  Java           (.java)            - class, interface, method, enum");
    println!("  Go             (.go)              - function, struct, interface, method");
    println!("  C              (.c, .h)           - function, struct, enum, union");
    println!("  C++            (.cpp, .cc, .hpp)  - function, class, struct, enum, namespace");
    println!("  C#             (.cs)              - class, interface, method, struct, enum");
    println!("  Swift          (.swift)           - function, class, struct, enum, protocol");
    println!("  Kotlin         (.kt, .kts)        - function, class, interface, enum, object");
    println!("  PHP            (.php)             - function, class, method, interface, trait");
    println!("  Ruby           (.rb)              - method, class, module");
    println!("  Shell/Bash     (.sh, .bash)       - function");
    println!("  Scala          (.scala)           - function, class, trait, object");
    println!("  Dart           (.dart)            - function, class, method, enum, mixin");
    println!("  Lua            (.lua)             - function");
    println!("  R              (.r, .R)           - function, class");
    println!("  Perl           (.pl, .pm)         - function, package, class");
    println!("  SQL            (.sql)             - table, function, procedure, view");

    Ok(())
}
