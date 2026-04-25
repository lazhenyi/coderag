//! Multi-Language AST Parser Module
//!
//! Provides tree-sitter based parsing for Top 20 programming languages.

mod registry;
mod languages;

pub use registry::{LanguageRegistry, LanguageConfig};
pub use languages::Language;

use std::path::Path;
use anyhow::{Context, Result as AnyResult};

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
    /// Create a new parser with default languages registered
    pub fn new() -> AnyResult<Self> {
        let registry = LanguageRegistry::new()?;
        Ok(Self { registry })
    }

    /// Create a parser with specific languages
    pub fn with_languages(languages: Vec<Language>) -> AnyResult<Self> {
        let registry = LanguageRegistry::from_languages(languages)?;
        Ok(Self { registry })
    }

    /// Detect language from file path
    pub fn detect_language(&self, path: &Path) -> Option<Language> {
        self.registry.detect_language(path)
    }

    /// Detect language from file extension
    pub fn detect_language_by_extension(&self, ext: &str) -> Option<Language> {
        self.registry.detect_language_by_extension(ext)
    }

    /// Detect language from file content (using shebang or heuristics)
    pub fn detect_language_from_content(&self, content: &[u8]) -> Option<Language> {
        self.registry.detect_language_from_content(content)
    }

    /// Parse source code with auto language detection
    pub fn parse_auto(&self, source: &[u8], path: Option<&Path>) -> AnyResult<ParseResult> {
        let language = if let Some(p) = path {
            self.detect_language(p)
                .or_else(|| self.registry.detect_language_from_content(source))
                .context("Unable to detect language")?
        } else {
            self.registry.detect_language_from_content(source)
                .context("Unable to detect language from content")?
        };

        self.parse_with_language(source, language)
    }

    /// Parse source code with explicit language
    pub fn parse_with_language(&self, source: &[u8], language: Language) -> AnyResult<ParseResult> {
        let mut parser = tree_sitter::Parser::new();
        let ts_language = self.registry.get_tree_sitter_language(&language)
            .context("Language not registered")?;

        parser.set_language(&ts_language)
            .context("Failed to set language")?;

        let result = std::panic::catch_unwind(
            std::panic::AssertUnwindSafe(|| parser.parse(source, None))
        );

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

    /// List all supported languages
    pub fn supported_languages(&self) -> Vec<Language> {
        self.registry.list_languages()
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new().expect("Failed to create default parser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_parsing() {
        let parser = Parser::new().unwrap();

        let rust_code = r#"
fn main() {
    println!("Hello, world!");
}

struct MyStruct {
    value: i32,
}

impl MyStruct {
    fn new() -> Self {
        Self { value: 42 }
    }
}
"#;

        let result = parser.parse_auto(rust_code.as_bytes(), Some(Path::new("test.rs")))
            .unwrap();

        println!("Parsed language: {:?}", result.language);
        println!("Root node: {:?}", result.tree.root_node().kind());
    }

    #[test]
    fn test_python_parsing() {
        let parser = Parser::new().unwrap();

        let python_code = r#"
def hello():
    print("Hello, world!")

class MyClass:
    def __init__(self):
        self.value = 42

    def method(self):
        return self.value
"#;

        let result = parser.parse_auto(python_code.as_bytes(), Some(Path::new("test.py")))
            .unwrap();

        println!("Parsed language: {:?}", result.language);
        println!("Root node: {:?}", result.tree.root_node().kind());
    }

    #[test]
    fn test_javascript_parsing() {
        let parser = Parser::new().unwrap();

        let js_code = r#"
function hello() {
    console.log("Hello!");
}

class MyClass {
    constructor() {
        this.value = 42;
    }

    method() {
        return this.value;
    }
}
"#;

        let result = parser.parse_auto(js_code.as_bytes(), Some(Path::new("test.js")))
            .unwrap();

        println!("Parsed language: {:?}", result.language);
        println!("Root node: {:?}", result.tree.root_node().kind());
    }

    #[test]
    fn test_language_detection() {
        let parser = Parser::new().unwrap();

        // Test extension detection
        assert_eq!(parser.detect_language_by_extension("rs"), Some(Language::Rust));
        assert_eq!(parser.detect_language_by_extension("py"), Some(Language::Python));
        assert_eq!(parser.detect_language_by_extension("js"), Some(Language::JavaScript));
        assert_eq!(parser.detect_language_by_extension("java"), Some(Language::Java));
        assert_eq!(parser.detect_language_by_extension("go"), Some(Language::Go));

        // Test shebang detection
        let python_shebang = b"#!/usr/bin/env python\nprint('hello')";
        let lang = parser.registry.detect_language_from_content(python_shebang);
        assert_eq!(lang, Some(Language::Python));

        let rust_shebang = b"#!/usr/bin/rust-run\nfn main() {}";
        let lang = parser.registry.detect_language_from_content(rust_shebang);
        assert_eq!(lang, Some(Language::Rust));
    }
}
