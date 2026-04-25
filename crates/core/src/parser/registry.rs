//! Language Registry Module
//!
//! Manages registration and lookup of supported programming languages.

use std::collections::HashMap;
use std::path::Path;
use anyhow::{Context, Result as AnyResult};

use crate::parser::languages::Language;

/// Language configuration with metadata and file associations
#[derive(Debug, Clone)]
pub struct LanguageConfig {
    pub language: Language,
    pub name: String,
    pub extensions: Vec<String>,
    pub tree_sitter_lang: tree_sitter::Language,
    pub comment_style: CommentStyle,
}

/// Style of comments for a language
#[derive(Debug, Clone, Copy)]
pub enum CommentStyle {
    /// C-style: // and /* */
    CStyle,
    /// Python-style: # and """
    PythonStyle,
    /// SQL-style: --
    SqlStyle,
    /// Shell-style: #
    ShellStyle,
}

/// Language registry for managing supported languages
pub struct LanguageRegistry {
    configs: HashMap<Language, LanguageConfig>,
    extension_map: HashMap<String, Language>,
    shebang_map: HashMap<String, Language>,
}

impl LanguageRegistry {
    /// Create a new registry with all supported languages
    pub fn new() -> AnyResult<Self> {
        let mut registry = Self {
            configs: HashMap::new(),
            extension_map: HashMap::new(),
            shebang_map: HashMap::new(),
        };

        // Register all languages
        registry.register_language(super::languages::rust_config())?;
        registry.register_language(super::languages::python_config())?;
        registry.register_language(super::languages::javascript_config())?;
        registry.register_language(super::languages::typescript_config())?;
        registry.register_language(super::languages::java_config())?;
        registry.register_language(super::languages::go_config())?;

        Ok(registry)
    }

    /// Create a registry from specific languages
    pub fn from_languages(languages: Vec<Language>) -> AnyResult<Self> {
        let mut registry = Self {
            configs: HashMap::new(),
            extension_map: HashMap::new(),
            shebang_map: HashMap::new(),
        };

        for lang in languages {
            let config = match lang {
                Language::Rust => super::languages::rust_config(),
                Language::Python => super::languages::python_config(),
                Language::JavaScript => super::languages::javascript_config(),
                Language::TypeScript => super::languages::typescript_config(),
                Language::Java => super::languages::java_config(),
                Language::Go => super::languages::go_config(),
                _ => anyhow::bail!("Unsupported language: {:?}", lang),
            };
            registry.register_language(config)?;
        }

        Ok(registry)
    }

    fn register_language(&mut self, config: LanguageConfig) -> AnyResult<()> {
        // Register in language map
        self.configs.insert(config.language.clone(), config.clone());

        // Register extensions
        for ext in &config.extensions {
            // Strip leading dot for path.extension() compatibility
            let key = ext.strip_prefix('.').unwrap_or(ext);
            self.extension_map.insert(key.to_string(), config.language.clone());
        }

        // Register shebangs
        if let Some(shebang) = get_shebang_for_language(&config.language) {
            self.shebang_map.insert(shebang.to_string(), config.language.clone());
        }

        Ok(())
    }

    /// Detect language from file path
    pub fn detect_language(&self, path: &Path) -> Option<Language> {
        // Try file name first (for special files like Makefile)
        if let Some(file_name) = path.file_name()?.to_str() {
            if let Some(lang) = self.detect_special_file(file_name) {
                return Some(lang);
            }
        }

        // Try extension
        let ext = path.extension()?.to_str()?;
        self.detect_language_by_extension(ext)
    }

    /// Detect language from file extension
    pub fn detect_language_by_extension(&self, ext: &str) -> Option<Language> {
        self.extension_map.get(ext).cloned()
    }

    /// Detect language from file content (shebang)
    pub fn detect_language_from_content(&self, content: &[u8]) -> Option<Language> {
        // Check for shebang
        if content.starts_with(b"#!") {
            if let Ok(shebang) = std::str::from_utf8(content) {
                let line = shebang.lines().next()?;
                for (pattern, lang) in &self.shebang_map {
                    if line.contains(pattern) {
                        return Some(lang.clone());
                    }
                }
            }
        }

        // Try heuristics based on content patterns
        self.detect_by_content_heuristics(content)
    }

    fn detect_special_file(&self, file_name: &str) -> Option<Language> {
        match file_name {
            "Makefile" | "makefile" | "CMakeLists.txt" | "*.mk" => Some(Language::C),
            "BUILD" | "BUILD.bazel" => Some(Language::Python), // Bazel uses Starlark
            _ => None,
        }
    }

    fn detect_by_content_heuristics(&self, content: &[u8]) -> Option<Language> {
        let content_str = String::from_utf8_lossy(content);

        // Python indicators
        if content_str.contains("def ") && content_str.contains(":")
            || content_str.contains("import ") && content_str.contains("from ")
        {
            return Some(Language::Python);
        }

        // Rust indicators
        if content_str.contains("fn ") && content_str.contains("->")
            || content_str.contains("impl ")
            || content_str.contains("let mut")
        {
            return Some(Language::Rust);
        }

        // Go indicators
        if content_str.contains("package ")
            && content_str.contains("func ")
            && content_str.contains("import \"")
        {
            return Some(Language::Go);
        }

        // Java/TypeScript indicators
        if content_str.contains("public class ")
            || content_str.contains("public static void main")
        {
            return Some(Language::Java);
        }

        None
    }

    /// Get tree-sitter Language for the given language
    pub fn get_tree_sitter_language(&self, language: &Language) -> AnyResult<tree_sitter::Language> {
        let config = self.configs.get(language)
            .context("Language not found in registry")?;

        Ok(config.tree_sitter_lang.clone())
    }

    /// List all registered languages
    pub fn list_languages(&self) -> Vec<Language> {
        self.configs.keys().cloned().collect()
    }

    /// Get configuration for a language
    pub fn get_config(&self, language: &Language) -> Option<&LanguageConfig> {
        self.configs.get(language)
    }
}

/// Get shebang patterns for a language
fn get_shebang_for_language(language: &Language) -> Option<&'static str> {
    match language {
        Language::Python => Some("python"),
        Language::JavaScript => Some("node"),
        Language::Rust => Some("rust"),
        Language::Go => Some("go"),
        Language::Ruby => Some("ruby"),
        Language::Perl => Some("perl"),
        Language::Shell => Some("bash"),
        _ => None,
    }
}

impl Language {
    /// Get all extensions for this language
    pub fn extensions(&self) -> Vec<&'static str> {
        match self {
            Language::Rust => vec![".rs"],
            Language::Python => vec![".py", ".pyw"],
            Language::JavaScript => vec![".js", ".jsx", ".mjs"],
            Language::TypeScript => vec![".ts", ".tsx", ".mts"],
            Language::Java => vec![".java"],
            Language::Go => vec![".go"],
            Language::C => vec![".c", ".h"],
            Language::Cpp => vec![".cpp", ".cc", ".cxx", ".hpp", ".hh"],
            Language::CSharp => vec![".cs"],
            Language::Swift => vec![".swift"],
            Language::Kotlin => vec![".kt", ".kts"],
            Language::Php => vec![".php"],
            Language::Ruby => vec![".rb"],
            Language::Scala => vec![".scala"],
            Language::Dart => vec![".dart"],
            Language::Lua => vec![".lua"],
            Language::R => vec![".r", ".R"],
            Language::Perl => vec![".pl", ".pm"],
            Language::Shell => vec![".sh", ".bash"],
            Language::Sql => vec![".sql"],
            Language::Markdown => vec![".md"],
            Language::Unknown => vec![],
        }
    }
}
