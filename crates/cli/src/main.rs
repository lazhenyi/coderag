//! CodeRAG CLI
//!
//! Command-line interface for the CodeRAG code indexing system.

use anyhow::{Context, Result as AnyResult};
use clap::{Parser, Subcommand};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use coderag_core::repo::GitRepo;
use coderag_indexer::{FullIndexer, IncrementalIndexer, IndexConfig};
use coderag_storage::{QdrantClient, QdrantConfig, SearchOptions, SearchFilter};

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

    match cli.command {
        Commands::Index { full, branch } => {
            index_repository(&cli.repo, &branch, full).await?;
        }
        Commands::Search { query, language, limit } => {
            search_code(&cli.repo, &query, language.as_deref(), limit).await?;
        }
        Commands::Info { tree } => {
            show_info(&cli.repo, tree)?;
        }
        Commands::Languages => {
            show_languages()?;
        }
    }

    Ok(())
}

async fn index_repository(repo_path: &str, branch: &str, full: bool) -> AnyResult<()> {
    info!("Indexing repository: {} (branch: {})", repo_path, branch);

    let config = IndexConfig {
        repo_path: repo_path.to_string(),
        branch: branch.to_string(),
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
    } else {
        // Incremental indexing
        let repo = GitRepo::open(repo_path)
            .context("Failed to open repository")?;

        let head = repo.head_commit()
            .context("Failed to get HEAD commit")?;

        let indexer = IncrementalIndexer::new(config)?;
        let stats = indexer.run(head.oid.inner(), head.oid.inner()).await?;

        println!("\nIncremental indexing complete!");
        println!("  Files processed: {}", stats.files_processed);
        println!("  Chunks created: {}", stats.chunks_created);
        println!("  Duration: {:.2}s", stats.duration_secs);
    }

    Ok(())
}

async fn search_code(
    _repo_path: &str,
    query: &str,
    language: Option<&str>,
    limit: usize,
) -> AnyResult<()> {
    println!("Searching for: {}", query);

    if let Some(lang) = language {
        println!("Language filter: {}", lang);
    }
    println!("Limit: {}", limit);

    // Note: Full search implementation would require:
    // 1. Generating embedding for the query
    // 2. Searching Qdrant
    // 3. Returning formatted results

    println!("\nSearch functionality requires Qdrant connection.");
    println!("Run 'coderag index --full' first to index the repository.");

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

fn show_languages() -> AnyResult<()> {
    println!("Supported languages:");

    let languages = vec![
        ("Rust", ".rs", "function, struct, enum, trait, impl, module"),
        ("Python", ".py", "function, class, method"),
        ("JavaScript", ".js, .jsx", "function, class, method"),
        ("TypeScript", ".ts, .tsx", "function, class, interface, method"),
        ("Java", ".java", "class, interface, method, enum"),
        ("Go", ".go", "function, struct, interface, method"),
    ];

    for (name, ext, symbols) in languages {
        println!("  {} ({})", name, ext);
        println!("    Symbols: {}", symbols);
    }

    Ok(())
}
