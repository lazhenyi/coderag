//! Serve command handler — launches the API server with embedded frontend.

use crate::config::Config;
use anyhow::{Context, Result as AnyResult};
use std::process::Command;
use tracing::info;

pub fn serve_api(bind: &str, config: &Config) -> AnyResult<()> {
    info!("Starting API server on {}", bind);

    let server_bin = which_coderag_server()?;

    let mut cmd = Command::new(&server_bin);
    cmd.env("CODERAG_BIND", bind);

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
        .with_context(|| format!("Failed to start server binary: {}", server_bin.display()))?;

    println!("CodeRAG server listening on {}", bind);
    println!("  Web UI: http://{}", bind);
    println!("  API:    http://{}/api", bind);
    println!("  Health: http://{}/health", bind);
    println!();
    println!("Press Ctrl+C to stop.");
    println!();

    // Wait for child process; on Ctrl+C the signal propagates
    let status = child.wait().context("Server process failed")?;
    if !status.success() {
        anyhow::bail!("Server exited with status: {}", status);
    }
    Ok(())
}

fn which_coderag_server() -> AnyResult<std::path::PathBuf> {
    // Prefer the sibling binary from the same workspace build
    let current_exe =
        std::env::current_exe().context("Cannot determine current executable path")?;

    let server_name = if cfg!(windows) {
        "coderag-server.exe"
    } else {
        "coderag-server"
    };

    // Check same directory as this CLI binary
    let sibling = current_exe
        .parent()
        .map(|p| p.join(server_name))
        .filter(|p| p.exists());

    if let Some(p) = sibling {
        return Ok(p);
    }

    // Fallback: check workspace target directory
    if let Ok(cargo_manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_dir = std::path::PathBuf::from(cargo_manifest_dir);
        if let Some(project_root) = manifest_dir.parent().and_then(|p| p.parent()) {
            let profile = if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            };
            let target = project_root.join("target").join(profile).join(server_name);
            if target.exists() {
                return Ok(target);
            }
        }
    }

    // Last resort: search PATH
    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join(server_name);
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }

    anyhow::bail!(
        "Cannot find coderag-server binary.\n\
         Build it first: cargo build -p coderag-server\n\
         Then ensure it is in the same directory as this CLI or in your PATH."
    )
}
