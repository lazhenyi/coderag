//! Analyzer
//!
//! High-level API for extracting symbols from source code.

use crate::analyzer::extractor::SymbolExtractor;
use crate::analyzer::symbol::Symbol;
use crate::parser::Language;

/// Analyzer for extracting symbols from source code
pub struct Analyzer {
    extractor: SymbolExtractor,
}

impl Analyzer {
    /// Create a new analyzer
    pub fn new() -> Self {
        Self {
            extractor: SymbolExtractor::new(),
        }
    }

    /// Extract all symbols from parsed tree
    pub fn extract_symbols(
        &self,
        source: &[u8],
        tree: &tree_sitter::Tree,
        language: &Language,
        file_path: &str,
    ) -> Vec<Symbol> {
        self.extractor.extract(source, tree, language, file_path)
    }

    /// Extract symbols with module path
    pub fn extract_symbols_with_module(
        &self,
        source: &[u8],
        tree: &tree_sitter::Tree,
        language: &Language,
        file_path: &str,
        module_path: &str,
    ) -> Vec<Symbol> {
        let mut symbols = self.extract_symbols(source, tree, language, file_path);
        for symbol in &mut symbols {
            symbol.module_path = module_path.to_string();
        }
        symbols
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}
