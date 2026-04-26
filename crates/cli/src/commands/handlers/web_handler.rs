//! Web command handler — launches both API server and MCP server together.

use crate::config::Config;
use anyhow::{Context, Result as AnyResult};
use std::process::{Command, Stdio};
use tokio::sync::oneshot;
use tracing::info;

pub fn serve_web(api_bind: &str, mcp_bind: &str, config: &Config) -> AnyResult<()> {
    info!("Starting web stack: API on {}, MCP on {}", api_bind, mcp_bind);

    let api_bin = which_coderag_server()?;
    let mcp_bin = which_coderag_mcp()?;

    let mut api_child = spawn_server(&api_bin, "CODERAG_BIND", api_bind, config)?;
    let mcp_child = spawn_mcp(&mcp_bin, "CODERAG_MCP_BIND", mcp_bind, config)?;

    println!("CodeRAG web stack started:");
    println!();
    println!("  Web UI:  http://{}/", api_bind);
    println!("  API:     http://{}/api", api_bind);
    println!("  Health:  http://{}/health", api_bind);
    println!("  MCP:     http://{}/mcp", mcp_bind);
    println!();
    println!("Press Ctrl+C to stop both servers.");
    println!();

    // Wait for API server; MCP runs alongside
    let api_status = api_child.wait().context("API server process failed")?;
    if !api_status.success() {
        anyhow::bail!("API server exited with status: {}", api_status);
    }
    Ok(())
}

fn spawn_server(bin: &std::path::Path, env_key: &str, bind: &str, config: &Config) -> AnyResult<std::process::Child> {
    let mut cmd = Command::new(bin);
    cmd.env(env_key, bind);

    if let Some(ref v) = config.repo_path {
        cmd.env("CODERAG_REPO", v);
    }
    if let Some(ref v) = config.embed_url {
        cmd.env("CODERAG_EMBED_URL", v);
    }
    if let Some(ref v) = config.embed_model {
        cmd.env("CODERAG_EMBED_MODEL", v);
    }
    if let Some(v) = config.embed_dimension {
        cmd.env("CODERAG_EMBED_DIMENSION", v.to_string());
    }
    if let Some(ref v) = config.openai_api_key {
        cmd.env("OPENAI_API_KEY", v);
    }
    if let Some(ref v) = config.qdrant_url {
        cmd.env("QDRANT_URL", v);
    }
    if let Some(ref v) = config.qdrant_api_key {
        cmd.env("QDRANT_API_KEY", v);
    }
    if let Some(ref v) = config.collection_name {
        cmd.env("CODERAG_COLLECTION", v);
    }
    if let Some(ref v) = config.state_file {
        cmd.env("CODERAG_STATE_FILE", v);
    }
    if let Some(v) = config.batch_size {
        cmd.env("CODERAG_BATCH_SIZE", v.to_string());
    }

    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    let child = cmd.spawn().with_context(|| format!("Failed to start server: {}", bin.display()))?;
    Ok(child)
}

fn spawn_mcp(bin: &std::path::Path, env_key: &str, bind: &str, config: &Config) -> AnyResult<std::process::Child> {
    let mut cmd = Command::new(bin);
    cmd.env(env_key, bind);

    if let Some(ref v) = config.repo_path {
        cmd.env("CODERAG_REPO", v);
    }
    if let Some(ref v) = config.embed_url {
        cmd.env("CODERAG_EMBED_URL", v);
    }
    if let Some(ref v) = config.embed_model {
        cmd.env("CODERAG_EMBED_MODEL", v);
    }
    if let Some(v) = config.embed_dimension {
        cmd.env("CODERAG_EMBED_DIMENSION", v.to_string());
    }
    if let Some(ref v) = config.openai_api_key {
        cmd.env("OPENAI_API_KEY", v);
    }
    if let Some(ref v) = config.qdrant_url {
        cmd.env("QDRANT_URL", v);
    }
    if let Some(ref v) = config.qdrant_api_key {
        cmd.env("QDRANT_API_KEY", v);
    }
    if let Some(ref v) = config.collection_name {
        cmd.env("CODERAG_COLLECTION", v);
    }
    if let Some(ref v) = config.state_file {
        cmd.env("CODERAG_STATE_FILE", v);
    }
    if let Some(v) = config.batch_size {
        cmd.env("CODERAG_BATCH_SIZE", v.to_string());
    }

    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    let child = cmd.spawn().with_context(|| format!("Failed to start MCP: {}", bin.display()))?;
    Ok(child)
}

fn which_coderag_server() -> AnyResult<std::path::PathBuf> {
    let current_exe = std::env::current_exe()
        .context("Cannot determine current executable path")?;
    let server_name = if cfg!(windows) { "coderag-server.exe" } else { "coderag-server" };

    let sibling = current_exe.parent()
        .map(|p| p.join(server_name))
        .filter(|p| p.exists());
    if let Some(p) = sibling {
        return Ok(p);
    }

    if let Ok(cargo_manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_dir = std::path::PathBuf::from(cargo_manifest_dir);
        if let Some(project_root) = manifest_dir.parent().and_then(|p| p.parent()) {
            let profile = if cfg!(debug_assertions) { "debug" } else { "release" };
            let target = project_root.join("target").join(profile).join(server_name);
            if target.exists() {
                return Ok(target);
            }
        }
    }

    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join(server_name);
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }

    anyhow::bail!("Cannot find coderag-server binary. Build it first: cargo build -p coderag-server")
}

fn which_coderag_mcp() -> AnyResult<std::path::PathBuf> {
    let current_exe = std::env::current_exe()
        .context("Cannot determine current executable path")?;
    let mcp_name = if cfg!(windows) { "coderag-mcp.exe" } else { "coderag-mcp" };

    let sibling = current_exe.parent()
        .map(|p| p.join(mcp_name))
        .filter(|p| p.exists());
    if let Some(p) = sibling {
        return Ok(p);
    }

    if let Ok(cargo_manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_dir = std::path::PathBuf::from(cargo_manifest_dir);
        if let Some(project_root) = manifest_dir.parent().and_then(|p| p.parent()) {
            let profile = if cfg!(debug_assertions) { "debug" } else { "release" };
            let target = project_root.join("target").join(profile).join(mcp_name);
            if target.exists() {
                return Ok(target);
            }
        }
    }

    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join(mcp_name);
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }

    anyhow::bail!("Cannot find coderag-mcp binary. Build it first: cargo build -p coderag-mcp")
}
