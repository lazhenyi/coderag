//! Analyzer Module
//!
//! Extracts symbols (functions, classes, methods, etc.) from parsed AST.

mod analyzer;
mod extractor;
mod extractors;
pub mod symbol;
mod tests;

pub use analyzer::Analyzer;
pub use extractor::SymbolExtractor;
pub use symbol::{Symbol, SymbolKind, SymbolScope};
