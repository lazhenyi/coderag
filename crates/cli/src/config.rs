//! Configuration Module
//!
//! Loads configuration from environment variables, .env files,
//! and ~/.config/coderag/settings.json.

use anyhow::{Context, Result as AnyResult};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// CodeRAG configuration
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Qdrant server URL
    pub qdrant_url: Option<String>,
    /// Qdrant API key
    pub qdrant_api_key: Option<String>,
    /// OpenAI API key
    pub openai_api_key: Option<String>,
    /// Embedding model
    pub embed_model: Option<String>,
    /// Embedding API URL
    pub embed_url: Option<String>,
    /// Embedding dimension
    pub embed_dimension: Option<usize>,
    /// Default repository path
    pub repo_path: Option<String>,
    /// Default branch
    pub branch: Option<String>,
    /// State file path
    pub state_file: Option<String>,
    /// Collection name
    pub collection_name: Option<String>,
    /// Batch size
    pub batch_size: Option<usize>,
    /// Additional raw key-value pairs
    pub extra: HashMap<String, String>,
}

impl Config {
    /// Load configuration from all sources.
    /// Priority (highest to lowest):
    /// 1. Environment variables
    /// 2. .env file in current directory
    /// 3. ~/.config/coderag/settings.json
    pub fn load() -> AnyResult<Self> {
        let mut config = Config::default();

        // 1. Load from ~/.config/coderag/settings.json
        if let Some(json_config) = Self::load_json_config()? {
            Self::apply_json(&mut config, json_config);
        }

        // 2. Load from .env in current directory
        Self::load_dotenv()?;

        // 3. Environment variables override everything
        Self::load_env(&mut config);

        Ok(config)
    }

    /// Load from ~/.config/coderag/settings.json
    fn load_json_config() -> AnyResult<Option<JsonConfig>> {
        let config_path = Self::json_config_path()?;

        if !config_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config: {}", config_path.display()))?;

        let json: JsonConfig = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse config: {}", config_path.display()))?;

        tracing::debug!("Loaded config from {}", config_path.display());
        Ok(Some(json))
    }

    /// Get the JSON config path: ~/.config/coderag/settings.json
    fn json_config_path() -> AnyResult<PathBuf> {
        let home = dirs::home_dir()
            .context("Cannot determine home directory")?;
        Ok(home.join(".config").join("coderag").join("settings.json"))
    }

    /// Load from .env file in current directory
    fn load_dotenv() -> AnyResult<()> {
        // Try current directory first
        let local_path = PathBuf::from(".env");
        if local_path.exists() {
            dotenvy::from_path(&local_path).ok();
            tracing::debug!("Loaded .env from {}", local_path.display());
        }

        // Also try dotenvy::dotenv() which searches parent directories
        dotenvy::dotenv().ok();

        Ok(())
    }

    /// Apply JSON config values to Config
    fn apply_json(config: &mut Config, json: JsonConfig) {
        if let Some(v) = json.qdrant_url {
            config.qdrant_url = Some(v);
        }
        if let Some(v) = json.qdrant_api_key {
            config.qdrant_api_key = Some(v);
        }
        if let Some(v) = json.openai_api_key {
            config.openai_api_key = Some(v);
        }
        if let Some(v) = json.embed_model {
            config.embed_model = Some(v);
        }
        if let Some(v) = json.embed_url {
            config.embed_url = Some(v);
        }
        if let Some(v) = json.embed_dimension {
            config.embed_dimension = Some(v);
        }
        if let Some(v) = json.repo_path {
            config.repo_path = Some(v);
        }
        if let Some(v) = json.branch {
            config.branch = Some(v);
        }
        if let Some(v) = json.state_file {
            config.state_file = Some(v);
        }
        if let Some(v) = json.collection_name {
            config.collection_name = Some(v);
        }
        if let Some(v) = json.batch_size {
            config.batch_size = Some(v);
        }
        if let Some(extra) = json.extra {
            config.extra = extra;
        }
    }

    /// Apply environment variables to Config
    fn load_env(config: &mut Config) {
        if let Ok(v) = std::env::var("QDRANT_URL") {
            config.qdrant_url = Some(v);
        }
        if let Ok(v) = std::env::var("QDRANT_API_KEY") {
            config.qdrant_api_key = Some(v);
        }
        if let Ok(v) = std::env::var("OPENAI_API_KEY") {
            config.openai_api_key = Some(v);
        }
        if let Ok(v) = std::env::var("CODERAG_EMBED_MODEL") {
            config.embed_model = Some(v);
        }
        if let Ok(v) = std::env::var("CODERAG_EMBED_URL") {
            config.embed_url = Some(v);
        }
        if let Ok(v) = std::env::var("CODERAG_EMBED_DIMENSION") {
            config.embed_dimension = v.parse().ok();
        }
        if let Ok(v) = std::env::var("CODERAG_REPO") {
            config.repo_path = Some(v);
        }
        if let Ok(v) = std::env::var("CODERAG_BRANCH") {
            config.branch = Some(v);
        }
        if let Ok(v) = std::env::var("CODERAG_STATE_FILE") {
            config.state_file = Some(v);
        }
        if let Ok(v) = std::env::var("CODERAG_COLLECTION") {
            config.collection_name = Some(v);
        }
        if let Ok(v) = std::env::var("CODERAG_BATCH_SIZE") {
            config.batch_size = v.parse().ok();
        }
    }

    /// Get a string config value, with CLI override > env > json
    #[allow(dead_code)]
    pub fn get_str(&self, cli: Option<&str>, env_key: &str, json_key: Option<&str>) -> String {
        cli.map(String::from)
            .or_else(|| std::env::var(env_key).ok())
            .or_else(|| {
                json_key.and_then(|k| {
                    self.extra.get(k).cloned().or_else(|| {
                        match k {
                            "qdrant_url" => self.qdrant_url.clone(),
                            "qdrant_api_key" => self.qdrant_api_key.clone(),
                            "openai_api_key" => self.openai_api_key.clone(),
                            "embed_model" => self.embed_model.clone(),
                            "embed_url" => self.embed_url.clone(),
                            "repo_path" => self.repo_path.clone(),
                            "branch" => self.branch.clone(),
                            "state_file" => self.state_file.clone(),
                            "collection_name" => self.collection_name.clone(),
                            _ => None,
                        }
                    })
                })
            })
            .unwrap_or_default()
    }

    /// Get optional string config
    #[allow(dead_code)]
    pub fn get_optional_str(&self, env_key: &str) -> Option<String> {
        std::env::var(env_key).ok()
    }
}

/// JSON configuration structure (matches settings.json)
#[derive(Debug, Clone, Deserialize)]
struct JsonConfig {
    #[serde(rename = "qdrantUrl", default)]
    pub qdrant_url: Option<String>,

    #[serde(rename = "qdrantApiKey", default)]
    pub qdrant_api_key: Option<String>,

    #[serde(rename = "openaiApiKey", default)]
    pub openai_api_key: Option<String>,

    #[serde(rename = "embedModel", default)]
    pub embed_model: Option<String>,

    #[serde(rename = "embedUrl", default)]
    pub embed_url: Option<String>,

    #[serde(rename = "embedDimension", default)]
    pub embed_dimension: Option<usize>,

    #[serde(rename = "repoPath", default)]
    pub repo_path: Option<String>,

    #[serde(rename = "branch", default)]
    pub branch: Option<String>,

    #[serde(rename = "stateFile", default)]
    pub state_file: Option<String>,

    #[serde(rename = "collectionName", default)]
    pub collection_name: Option<String>,

    #[serde(rename = "batchSize", default)]
    pub batch_size: Option<usize>,

    /// Additional key-value pairs
    #[serde(flatten)]
    pub extra: Option<HashMap<String, String>>,
}

impl JsonConfig {}

/// Example settings.json content
#[allow(dead_code)]
pub const EXAMPLE_SETTINGS_JSON: &str = r#"{
  // Qdrant connection
  "qdrantUrl": "http://localhost:6333",
  "qdrantApiKey": "your-qdrant-api-key",

  // OpenAI embedding
  "openaiApiKey": "sk-...",
  "embedModel": "text-embedding-ada-002",
  "embedUrl": "https://api.openai.com/v1/embeddings",
  "embedDimension": 1536,

  // Indexing defaults
  "repoPath": ".",
  "branch": "main",
  "collectionName": "coderag",
  "stateFile": ".coderag/state.json",
  "batchSize": 100
}
"#;

/// Example .env content
#[allow(dead_code)]
pub const EXAMPLE_DOTENV: &str = r#"# CodeRAG Configuration
# Copy to .env in your project root

# Qdrant
QDRANT_URL=http://localhost:6333
QDRANT_API_KEY=

# OpenAI
OPENAI_API_KEY=sk-...

# Embedding
CODERAG_EMBED_MODEL=text-embedding-ada-002
CODERAG_EMBED_URL=https://api.openai.com/v1/embeddings
CODERAG_EMBED_DIMENSION=1536

# Indexing
CODERAG_REPO=.
CODERAG_BRANCH=main
CODERAG_COLLECTION=coderag
CODERAG_STATE_FILE=.coderag/state.json
CODERAG_BATCH_SIZE=100
"#;

/// Show configuration sources and their paths
#[allow(dead_code)]
pub fn show_config_info() {
    println!("Configuration sources (loaded in order):");
    println!();
    println!("  1. Environment variables (highest priority)");
    println!("  2. .env file in current directory");
    println!("  3. ~/.config/coderag/settings.json (lowest priority)");
    println!();

    // Show JSON path
    if let Some(home) = dirs::home_dir() {
        let json_path = home.join(".config").join("coderag").join("settings.json");
        println!("  JSON config: {}", json_path.display());
        if json_path.exists() {
            println!("    [exists]");
        } else {
            println!("    [not found — will be created on first write]");
        }
    }

    let dotenv_path = PathBuf::from(".env");
    println!("  .env file:   {}", dotenv_path.display());
    if dotenv_path.exists() {
        println!("    [exists]");
    } else {
        println!("    [not found]");
    }
}
