//! CodeRAG CLI

mod commands;
mod config;

use anyhow::Result as AnyResult;
use clap::Parser;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use commands::Cli;
use config::Config;

#[tokio::main]
async fn main() -> AnyResult<()> {
    let cli = Cli::parse();

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

    let config = Config::load()?;

    commands::handle_command(&cli.command, &cli.repo, &config).await?;

    Ok(())
}
