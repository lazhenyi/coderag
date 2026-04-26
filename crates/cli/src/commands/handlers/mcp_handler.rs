//! MCP command handler — launches the MCP server binary.

use crate::config::Config;
use anyhow::{Context, Result as AnyResult};
use std::process::Command;
use tracing::info;

pub fn serve_mcp(bind: Option<String>, config: &Config) -> AnyResult<()> {
    let bind = bind.unwrap_or_else(|| "127.0.0.1:8081".to_string());
    info!("Starting MCP server on {}", bind);

    let mcp_bin = which_coderag_mcp()?;

    let mut cmd = Command::new(&mcp_bin);
    cmd.env("CODERAG_MCP_BIND", &bind);

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

    let mut child = cmd
        .spawn()
        .with_context(|| format!("Failed to start MCP binary: {}", mcp_bin.display()))?;

    println!("CodeRAG MCP server listening on {}", bind);
    println!("  MCP endpoint: http://{}/mcp", bind);
    println!("  Health:       http://{}/health", bind);
    println!();
    println!("Press Ctrl+C to stop.");
    println!();

    let status = child.wait().context("MCP server process failed")?;
    if !status.success() {
        anyhow::bail!("MCP server exited with status: {}", status);
    }
    Ok(())
}

fn which_coderag_mcp() -> AnyResult<std::path::PathBuf> {
    let current_exe = std::env::current_exe()
        .context("Cannot determine current executable path")?;

    let mcp_name = if cfg!(windows) { "coderag-mcp.exe" } else { "coderag-mcp" };

    // Check same directory as this CLI binary
    let sibling = current_exe.parent()
        .map(|p| p.join(mcp_name))
        .filter(|p| p.exists());

    if let Some(p) = sibling {
        return Ok(p);
    }

    // Fallback: check workspace target directory
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

    // Last resort: search PATH
    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join(mcp_name);
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }

    anyhow::bail!(
        "Cannot find coderag-mcp binary.\n\
         Build it first: cargo build -p coderag-mcp\n\
         Then ensure it is in the same directory as this CLI or in your PATH."
    )
}
