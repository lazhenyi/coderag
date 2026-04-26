//! Parser implementation

use super::languages::Language;
use super::registry::LanguageRegistry;
use anyhow::{Context, Result as AnyResult};
use std::path::Path;

/// Result of parsing a source file
pub struct ParseResult {
    pub tree: tree_sitter::Tree,
    pub language: Language,
}

/// Parser for multi-language source code
pub struct Parser {
    registry: LanguageRegistry,
}

impl Parser {
    pub fn new() -> AnyResult<Self> {
        let registry = LanguageRegistry::new()?;
        Ok(Self { registry })
    }

    pub fn with_languages(languages: Vec<Language>) -> AnyResult<Self> {
        let registry = LanguageRegistry::from_languages(languages)?;
        Ok(Self { registry })
    }

    pub fn detect_language(&self, path: &Path) -> Option<Language> {
        self.registry.detect_language(path)
    }

    pub fn detect_language_by_extension(&self, ext: &str) -> Option<Language> {
        self.registry.detect_language_by_extension(ext)
    }

    pub fn detect_language_from_content(&self, content: &[u8]) -> Option<Language> {
        self.registry.detect_language_from_content(content)
    }

    pub fn parse_auto(&self, source: &[u8], path: Option<&Path>) -> AnyResult<ParseResult> {
        let language = if let Some(p) = path {
            self.detect_language(p)
                .or_else(|| self.registry.detect_language_from_content(source))
                .context("Unable to detect language")?
        } else {
            self.registry
                .detect_language_from_content(source)
                .context("Unable to detect language from content")?
        };
        self.parse_with_language(source, language)
    }

    pub fn parse_with_language(&self, source: &[u8], language: Language) -> AnyResult<ParseResult> {
        let mut ts_parser = tree_sitter::Parser::new();
        let ts_language = self
            .registry
            .get_tree_sitter_language(&language)
            .context("Language not registered")?;

        ts_parser
            .set_language(&ts_language)
            .context("Failed to set language")?;

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ts_parser.parse(source, None)
        }));

        match result {
            Ok(Some(tree)) => Ok(ParseResult { tree, language }),
            Ok(None) => anyhow::bail!("Parser returned no tree for language '{}'", language),
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "unknown parser panic".to_string()
                };
                tracing::warn!("Tree-sitter panic for language '{}': {}", language, msg);
                anyhow::bail!("Tree-sitter panic for language '{}': {}", language, msg)
            }
        }
    }

    pub fn supported_languages(&self) -> Vec<Language> {
        self.registry.list_languages()
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new().expect("Failed to create default parser")
    }
}
