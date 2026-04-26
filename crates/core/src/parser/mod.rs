//! Multi-Language AST Parser Module
//!
//! Provides tree-sitter based parsing for Top 20 programming languages.

mod registry;
mod languages;
mod parser_impl;
mod tests;

pub use languages::Language;
pub use parser_impl::{Parser, ParseResult};
pub use registry::{LanguageRegistry, LanguageConfig};
