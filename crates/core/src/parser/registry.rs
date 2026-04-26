//! Language Registry Module
//!
//! Manages registration and lookup of supported programming languages.

mod registry_impl;
mod types;

pub use registry_impl::LanguageRegistry;
pub use types::{CommentStyle, LanguageConfig};
