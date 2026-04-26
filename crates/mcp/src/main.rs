//! CodeRAG MCP Server — Streamable HTTP transport
//!
//! This server implements the Model Context Protocol (MCP) with Streamable HTTP transport,
//! which supports both modern MCP clients and legacy SSE clients (via fallback).
//!
//! Usage:
//!   coderag-mcp                    # Default: 127.0.0.1:8081
//!   CODERAG_MCP_BIND=0.0.0.0:3000 coderag-mcp
//!
//! Endpoint: POST/GET /mcp

mod config;
mod server;

use axum::{Router, routing::get};
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager,
    StreamableHttpService,
};
use server::CodeRagServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    dotenvy::dotenv().ok();

    let config = config::load_config();
    let bind = config.bind.clone();

    tracing::info!("Starting CodeRAG MCP Server on {}", bind);
    tracing::info!("  Repository: {}", config.repo_path);
    tracing::info!("  Embed model: {}", config.embed_model);
    tracing::info!("  MCP endpoint: http://{}/mcp", bind);

    let http_service = StreamableHttpService::new(
        move || Ok(CodeRagServer::new(config.clone())),
        LocalSessionManager::default().into(),
        Default::default(),
    );

    let app = Router::new()
        .nest_service("/mcp", http_service)
        .route("/health", get(|| async { "ok" }));

    let listener = tokio::net::TcpListener::bind(&bind).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    tracing::info!("Shutting down MCP server...");
}
