//! Config subcommand handlers

use crate::config::{Config, EXAMPLE_SETTINGS_JSON};
use anyhow::{Context, Result as AnyResult};
use std::fs;
use std::path::PathBuf;

fn config_path() -> AnyResult<PathBuf> {
    let home = dirs::home_dir().context("Cannot determine home directory")?;
    Ok(home.join(".config").join("coderag").join("settings.json"))
}

pub fn config_show(config: &Config) -> AnyResult<()> {
    println!("CodeRAG Configuration");
    println!("=====================");
    println!();

    // Determine source for each value by checking env first, then config
    let env_vars = [
        ("qdrant_url", "QDRANT_URL"),
        ("qdrant_api_key", "QDRANT_API_KEY"),
        ("openai_api_key", "OPENAI_API_KEY"),
        ("embed_model", "CODERAG_EMBED_MODEL"),
        ("embed_url", "CODERAG_EMBED_URL"),
        ("embed_dimension", "CODERAG_EMBED_DIMENSION"),
        ("repo_path", "CODERAG_REPO"),
        ("branch", "CODERAG_BRANCH"),
        ("state_file", "CODERAG_STATE_FILE"),
        ("collection_name", "CODERAG_COLLECTION"),
        ("batch_size", "CODERAG_BATCH_SIZE"),
    ];

    let get_config_val = |key: &str| -> Option<String> {
        match key {
            "qdrant_url" => config.qdrant_url.clone(),
            "qdrant_api_key" => config.qdrant_api_key.clone(),
            "openai_api_key" => config.openai_api_key.clone(),
            "embed_model" => config.embed_model.clone(),
            "embed_url" => config.embed_url.clone(),
            "repo_path" => config.repo_path.clone(),
            "branch" => config.branch.clone(),
            "state_file" => config.state_file.clone(),
            "collection_name" => config.collection_name.clone(),
            "batch_size" => config.batch_size.map(|v| v.to_string()),
            "embed_dimension" => config.embed_dimension.map(|v| v.to_string()),
            _ => None,
        }
    };

    for (key, env_key) in env_vars {
        let env_val = std::env::var(env_key).ok();
        let cfg_val = get_config_val(key);
        match (env_val, cfg_val) {
            (Some(v), _) => println!("  {:25} {}  (env:{})", key, v, env_key),
            (None, Some(v)) => println!("  {:25} {}  (config)", key, v),
            (None, None) => println!("  {:25} <not set>", key),
        }
    }

    println!();
    let path = config_path()?;
    if path.exists() {
        println!("  Config file: {}", path.display());
    } else {
        println!("  Config file: not found (run `coderag config init` to create)");
    }

    Ok(())
}

pub fn config_init() -> AnyResult<()> {
    let path = config_path()?;
    if path.exists() {
        println!("Config file already exists at: {}", path.display());
        println!("Edit it directly or remove it and run `coderag config init` again.");
        return Ok(());
    }

    let parent = path.parent().unwrap();
    fs::create_dir_all(parent)
        .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;

    fs::write(&path, EXAMPLE_SETTINGS_JSON)
        .with_context(|| format!("Failed to write config file: {}", path.display()))?;

    println!("Created default config file at: {}", path.display());
    println!("Edit it with your values, then run `coderag config show` to verify.");
    Ok(())
}

pub fn config_set(key: &str, value: &str) -> AnyResult<()> {
    let path = config_path()?;

    let mut json: serde_json::Value = if path.exists() {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config: {}", path.display()))?;
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse config: {}", path.display()))?
    } else {
        serde_json::from_str(EXAMPLE_SETTINGS_JSON)?
    };

    let obj = json
        .as_object_mut()
        .context("Config file is not a JSON object")?;

    match key {
        "qdrant_url" | "qdrantUrl" => obj.insert("qdrantUrl".into(), value.into()),
        "qdrant_api_key" | "qdrantApiKey" => obj.insert("qdrantApiKey".into(), value.into()),
        "openai_api_key" | "openaiApiKey" => obj.insert("openaiApiKey".into(), value.into()),
        "embed_model" | "embedModel" => obj.insert("embedModel".into(), value.into()),
        "embed_url" | "embedUrl" => obj.insert("embedUrl".into(), value.into()),
        "collection_name" | "collectionName" => obj.insert("collectionName".into(), value.into()),
        "state_file" | "stateFile" => obj.insert("stateFile".into(), value.into()),
        "repo_path" | "repoPath" => obj.insert("repoPath".into(), value.into()),
        "branch" => obj.insert("branch".into(), value.into()),
        "batch_size" | "batchSize" => {
            value
                .parse::<usize>()
                .with_context(|| format!("batch_size must be a number, got: {}", value))?;
            obj.insert("batchSize".into(), value.into())
        }
        "embed_dimension" | "embedDimension" => {
            value
                .parse::<usize>()
                .with_context(|| format!("embed_dimension must be a number, got: {}", value))?;
            obj.insert("embedDimension".into(), value.into())
        }
        _ => {
            anyhow::bail!(
                "Unknown config key: {}\n\
                 Valid keys: qdrant_url, qdrant_api_key, openai_api_key, embed_model, \
                 embed_url, embed_dimension, collection_name, state_file, batch_size, repo_path, branch",
                key
            );
        }
    };

    let formatted = serde_json::to_string_pretty(&json).context("Failed to serialize config")?;

    fs::write(&path, formatted)
        .with_context(|| format!("Failed to write config: {}", path.display()))?;

    println!("Set {} = {}", key, value);
    println!("Config saved to: {}", path.display());
    Ok(())
}

pub fn config_path_info() -> AnyResult<()> {
    let path = config_path()?;
    println!("Config file location:");
    println!("  {}", path.display());
    if path.exists() {
        println!("  [exists]");
    } else {
        println!("  [not found — run `coderag config init` to create]");
    }
    Ok(())
}
