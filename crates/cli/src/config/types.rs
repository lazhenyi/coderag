//! Config types

use serde::Deserialize;
use std::collections::HashMap;

/// CodeRAG configuration
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub qdrant_url: Option<String>,
    pub qdrant_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub embed_model: Option<String>,
    pub embed_url: Option<String>,
    pub embed_dimension: Option<usize>,
    pub repo_path: Option<String>,
    pub branch: Option<String>,
    pub state_file: Option<String>,
    pub collection_name: Option<String>,
    pub batch_size: Option<usize>,
    pub extra: HashMap<String, String>,
}

/// JSON configuration structure
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct JsonConfig {
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
    #[serde(flatten)]
    pub extra: Option<HashMap<String, String>>,
}
