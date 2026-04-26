//! MCP server configuration

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Default)]
pub struct McpConfig {
    pub repo_path: String,
    pub embed_url: String,
    pub embed_model: String,
    pub embed_dimension: usize,
    pub embed_api_key: Option<String>,
    pub qdrant_url: String,
    pub qdrant_api_key: Option<String>,
    pub collection_name: String,
    pub batch_size: usize,
    pub state_file: String,
    pub bind: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JsonConfig {
    #[serde(rename = "qdrantUrl", default)]
    qdrant_url: Option<String>,
    #[serde(rename = "qdrantApiKey", default)]
    qdrant_api_key: Option<String>,
    #[serde(rename = "openaiApiKey", default)]
    openai_api_key: Option<String>,
    #[serde(rename = "embedModel", default)]
    embed_model: Option<String>,
    #[serde(rename = "embedUrl", default)]
    embed_url: Option<String>,
    #[serde(rename = "embedDimension", default)]
    embed_dimension: Option<usize>,
    #[serde(rename = "repoPath", default)]
    repo_path: Option<String>,
    #[serde(rename = "stateFile", default)]
    state_file: Option<String>,
    #[serde(rename = "collectionName", default)]
    collection_name: Option<String>,
    #[serde(rename = "batchSize", default)]
    batch_size: Option<usize>,
}

fn load_json_config() -> Option<JsonConfig> {
    let home = dirs::home_dir()?;
    let path = home.join(".config").join("coderag").join("settings.json");
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

fn load_dotenv() {
    dotenvy::dotenv().ok();
    let local = std::path::PathBuf::from(".env");
    if local.exists() {
        dotenvy::from_path(&local).ok();
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_opt(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

pub fn load_config() -> McpConfig {
    load_dotenv();
    let json = load_json_config();

    McpConfig {
        repo_path: json
            .as_ref()
            .and_then(|j| j.repo_path.clone())
            .or_else(|| env_opt("CODERAG_REPO"))
            .unwrap_or_else(|| ".".into()),
        embed_url: json
            .as_ref()
            .and_then(|j| j.embed_url.clone())
            .or_else(|| env_opt("CODERAG_EMBED_URL"))
            .unwrap_or_else(|| "https://dashscope.aliyuncs.com/compatible-mode/v1/embeddings".into()),
        embed_model: json
            .as_ref()
            .and_then(|j| j.embed_model.clone())
            .or_else(|| env_opt("CODERAG_EMBED_MODEL"))
            .unwrap_or_else(|| "text-embedding-v4".into()),
        embed_dimension: json
            .as_ref()
            .and_then(|j| j.embed_dimension)
            .or_else(|| env_opt("CODERAG_EMBED_DIMENSION").and_then(|v| v.parse().ok()))
            .unwrap_or(1024),
        embed_api_key: json
            .as_ref()
            .and_then(|j| j.openai_api_key.clone())
            .or_else(|| env_opt("OPENAI_API_KEY")),
        qdrant_url: json
            .as_ref()
            .and_then(|j| j.qdrant_url.clone())
            .or_else(|| env_opt("QDRANT_URL"))
            .unwrap_or_else(|| "http://localhost:6333".into()),
        qdrant_api_key: json
            .as_ref()
            .and_then(|j| j.qdrant_api_key.clone())
            .or_else(|| env_opt("QDRANT_API_KEY")),
        collection_name: json
            .as_ref()
            .and_then(|j| j.collection_name.clone())
            .or_else(|| env_opt("CODERAG_COLLECTION"))
            .unwrap_or_else(|| "coderag".into()),
        batch_size: json
            .as_ref()
            .and_then(|j| j.batch_size)
            .or_else(|| env_opt("CODERAG_BATCH_SIZE").and_then(|v| v.parse().ok()))
            .unwrap_or(100),
        state_file: json
            .as_ref()
            .and_then(|j| j.state_file.clone())
            .or_else(|| env_opt("CODERAG_STATE_FILE"))
            .unwrap_or_else(|| ".coderag/state.json".into()),
        bind: env_or("CODERAG_MCP_BIND", "127.0.0.1:8081"),
    }
}
