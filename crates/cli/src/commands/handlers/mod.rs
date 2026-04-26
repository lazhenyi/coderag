//! Command handlers

mod config_handler;
mod dispatcher;
mod index_handler;
mod info_handler;
mod mcp_handler;
mod search_handler;
mod serve_handler;
mod web_handler;

pub use dispatcher::handle_command;
