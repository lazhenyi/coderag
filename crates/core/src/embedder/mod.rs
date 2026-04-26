//! Embedder Module

mod client_impl;
mod embedder_impl;
mod tests;
pub mod types;

pub use client_impl::EmbedderClient;
pub use embedder_impl::Embedder;
pub use types::{EmbedderBackend, EmbedderConfig, Embedding};
