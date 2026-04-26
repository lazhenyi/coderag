//! Language Registry Module
//!
//! Manages registration and lookup of supported programming languages.

mod types;
mod registry_impl;

pub use types::{CommentStyle, LanguageConfig};
pub use registry_impl::LanguageRegistry;
