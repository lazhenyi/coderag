//! CLI types

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "coderag")]
#[command(about = "AST-based code RAG system with multi-language support")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Repository path (default: current directory)
    #[arg(short, long, default_value = ".")]
    pub repo: String,

    /// Log level
    #[arg(short, long, default_value = "info")]
    pub log_level: String,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Index a repository (full or incremental)
    Index {
        /// Run full indexing (reindex everything)
        #[arg(short, long)]
        full: bool,

        /// Branch to index
        #[arg(short, long, default_value = "main")]
        branch: String,

        /// Use local file storage instead of Qdrant
        #[arg(short, long)]
        local: bool,
    },

    /// Search for code
    Search {
        /// Search query
        query: String,

        /// Language filter
        #[arg(long)]
        language: Option<String>,

        /// Maximum results
        #[arg(short = 'n', long, default_value = "10")]
        limit: usize,

        /// Use local file storage instead of Qdrant
        #[arg(short, long)]
        local: bool,

        /// Output results as a force-graph JSON (nodes + links)
        #[arg(long)]
        graph: bool,
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

    /// Start API server with embedded web UI
    Serve {
        /// Bind address
        #[arg(short, long, default_value = "127.0.0.1:8080")]
        bind: String,
    },

    /// Start MCP (Model Context Protocol) server with Streamable HTTP transport
    Mcp {
        /// Bind address (default: 127.0.0.1:8081)
        #[arg(short, long)]
        bind: Option<String>,
    },

    /// Start both API server and MCP server for full web experience
    Web {
        /// API server bind address
        #[arg(long, default_value = "127.0.0.1:8080")]
        api_bind: String,

        /// MCP server bind address
        #[arg(long, default_value = "127.0.0.1:8081")]
        mcp_bind: String,
    },

    /// Manage configuration (show/init/set)
    #[command(subcommand)]
    Config(ConfigCommands),
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration and loaded values
    Show,
    /// Create default config file at ~/.config/coderag/settings.json
    Init,
    /// Set a configuration value
    Set {
        /// Config key (qdrant_url, qdrant_api_key, openai_api_key, embed_model, embed_url, embed_dimension, collection_name, state_file, batch_size)
        key: String,
        /// Config value
        value: String,
    },
    /// Show where configuration files are located
    Path,
}
