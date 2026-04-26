//! Multi-Language AST Parser Module
//!
//! Provides tree-sitter based parsing for Top 20 programming languages.

mod languages;
mod parser_impl;
mod registry;
mod tests;

pub use languages::Language;
pub use parser_impl::{ParseResult, Parser};
pub use registry::{LanguageConfig, LanguageRegistry};
