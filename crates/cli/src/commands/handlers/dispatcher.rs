//! Command dispatcher

use super::super::types::{Commands, ConfigCommands};
use super::config_handler::{config_init, config_path_info, config_set, config_show};
use super::index_handler::index_repository;
use super::info_handler::{eval_quality, show_info, show_languages};
use super::mcp_handler::serve_mcp;
use super::search_handler::search_code;
use super::serve_handler::serve_api;
use super::web_handler::serve_web;
use crate::config::Config;
use anyhow::Result as AnyResult;

pub async fn handle_command(command: &Commands, repo_path: &str, config: &Config) -> AnyResult<()> {
    match command {
        Commands::Index {
            full,
            branch,
            local,
        } => index_repository(repo_path, branch, *full, *local, config).await,
        Commands::Search {
            query,
            language,
            limit,
            local,
            graph,
        } => {
            search_code(
                repo_path,
                query,
                language.as_deref(),
                *limit,
                *local,
                *graph,
                config,
            )
            .await
        }
        Commands::Info { tree } => show_info(repo_path, *tree),
        Commands::Languages => show_languages(),
        Commands::Eval {
            language,
            qdrant_url,
        } => {
            eval_quality(
                repo_path,
                language.as_deref(),
                qdrant_url.as_deref(),
                config,
            )
            .await
        }
        Commands::Serve { bind } => serve_api(bind, config),
        Commands::Mcp { bind } => serve_mcp(bind.clone(), config),
        Commands::Web { api_bind, mcp_bind } => serve_web(api_bind, mcp_bind, config),
        Commands::Config(sub) => match sub {
            ConfigCommands::Show => config_show(config),
            ConfigCommands::Init => config_init(),
            ConfigCommands::Set { key, value } => config_set(key, value),
            ConfigCommands::Path => config_path_info(),
        },
    }
}
