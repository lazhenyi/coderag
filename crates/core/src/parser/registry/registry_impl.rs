//! Language Registry implementation

use super::types::{CommentStyle, LanguageConfig};
use crate::parser::languages::Language;
use anyhow::{Context, Result as AnyResult};
use std::collections::HashMap;
use std::path::Path;

/// Language registry for managing supported languages
pub struct LanguageRegistry {
    configs: HashMap<Language, LanguageConfig>,
    extension_map: HashMap<String, Language>,
    shebang_map: HashMap<String, Language>,
}

impl LanguageRegistry {
    pub fn new() -> AnyResult<Self> {
        let mut registry = Self {
            configs: HashMap::new(),
            extension_map: HashMap::new(),
            shebang_map: HashMap::new(),
        };
        for config in crate::parser::languages::all_configs() {
            registry.register_language(config)?;
        }
        Ok(registry)
    }

    pub fn from_languages(languages: Vec<Language>) -> AnyResult<Self> {
        let mut registry = Self {
            configs: HashMap::new(),
            extension_map: HashMap::new(),
            shebang_map: HashMap::new(),
        };
        for lang in languages {
            let config = match &lang {
                Language::Rust => crate::parser::languages::rust_config(),
                Language::Python => crate::parser::languages::python_config(),
                Language::JavaScript => crate::parser::languages::javascript_config(),
                Language::TypeScript => crate::parser::languages::typescript_config(),
                Language::Java => crate::parser::languages::java_config(),
                Language::Go => crate::parser::languages::go_config(),
                Language::C => crate::parser::languages::c_config(),
                Language::Cpp => crate::parser::languages::cpp_config(),
                Language::CSharp => crate::parser::languages::csharp_config(),
                Language::Swift => crate::parser::languages::swift_config(),
                Language::Php => crate::parser::languages::php_config(),
                Language::Ruby => crate::parser::languages::ruby_config(),
                Language::Shell => crate::parser::languages::bash_config(),
                Language::Scala => crate::parser::languages::scala_config(),
                Language::Dart => crate::parser::languages::dart_config(),
                Language::Lua => crate::parser::languages::lua_config(),
                Language::R => crate::parser::languages::r_config(),
                Language::Perl => crate::parser::languages::perl_config(),
                Language::Kotlin => crate::parser::languages::kotlin_config(),
                Language::Sql => crate::parser::languages::sql_config(),
                _ => anyhow::bail!("Unsupported language: {:?}", lang),
            };
            registry.register_language(config)?;
        }
        Ok(registry)
    }

    fn register_language(&mut self, config: LanguageConfig) -> AnyResult<()> {
        self.configs.insert(config.language.clone(), config.clone());
        for ext in &config.extensions {
            let key = ext.strip_prefix('.').unwrap_or(ext);
            self.extension_map
                .insert(key.to_string(), config.language.clone());
        }
        if let Some(shebang) = get_shebang_for_language(&config.language) {
            self.shebang_map
                .insert(shebang.to_string(), config.language.clone());
        }
        Ok(())
    }

    pub fn detect_language(&self, path: &Path) -> Option<Language> {
        if let Some(file_name) = path.file_name()?.to_str() {
            if let Some(lang) = detect_special_file(file_name) {
                return Some(lang);
            }
        }
        let ext = path.extension()?.to_str()?;
        self.detect_language_by_extension(ext)
    }

    pub fn detect_language_by_extension(&self, ext: &str) -> Option<Language> {
        self.extension_map.get(ext).cloned()
    }

    pub fn detect_language_from_content(&self, content: &[u8]) -> Option<Language> {
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
        detect_by_content_heuristics(content)
    }

    pub fn get_tree_sitter_language(
        &self,
        language: &Language,
    ) -> AnyResult<tree_sitter::Language> {
        let config = self.configs.get(language).context("Language not found")?;
        Ok(config.tree_sitter_lang.clone())
    }

    pub fn list_languages(&self) -> Vec<Language> {
        self.configs.keys().cloned().collect()
    }

    pub fn get_config(&self, language: &Language) -> Option<&LanguageConfig> {
        self.configs.get(language)
    }
}

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

fn detect_special_file(file_name: &str) -> Option<Language> {
    match file_name {
        "Makefile" | "makefile" | "CMakeLists.txt" => Some(Language::C),
        "BUILD" | "BUILD.bazel" => Some(Language::Python),
        _ => None,
    }
}

fn detect_by_content_heuristics(content: &[u8]) -> Option<Language> {
    let s = String::from_utf8_lossy(content);
    if (s.contains("def ") && s.contains(":")) || (s.contains("import ") && s.contains("from ")) {
        return Some(Language::Python);
    }
    if (s.contains("fn ") && s.contains("->")) || s.contains("impl ") || s.contains("let mut") {
        return Some(Language::Rust);
    }
    if s.contains("package ") && s.contains("func ") && s.contains("import \"") {
        return Some(Language::Go);
    }
    if s.contains("public class ") || s.contains("public static void main") {
        return Some(Language::Java);
    }
    None
}
