//! Language Definitions
//!
//! Provides tree-sitter language configurations for supported languages.

use super::registry::{CommentStyle, LanguageConfig};

/// Supported programming languages
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Java,
    Go,
    C,
    Cpp,
    CSharp,
    Swift,
    Kotlin,
    Php,
    Ruby,
    Scala,
    Dart,
    Lua,
    R,
    Perl,
    Shell,
    Sql,
    Markdown,
    Unknown,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Rust => write!(f, "rust"),
            Language::Python => write!(f, "python"),
            Language::JavaScript => write!(f, "javascript"),
            Language::TypeScript => write!(f, "typescript"),
            Language::Java => write!(f, "java"),
            Language::Go => write!(f, "go"),
            Language::C => write!(f, "c"),
            Language::Cpp => write!(f, "cpp"),
            Language::CSharp => write!(f, "csharp"),
            Language::Swift => write!(f, "swift"),
            Language::Kotlin => write!(f, "kotlin"),
            Language::Php => write!(f, "php"),
            Language::Ruby => write!(f, "ruby"),
            Language::Scala => write!(f, "scala"),
            Language::Dart => write!(f, "dart"),
            Language::Lua => write!(f, "lua"),
            Language::R => write!(f, "r"),
            Language::Perl => write!(f, "perl"),
            Language::Shell => write!(f, "shell"),
            Language::Sql => write!(f, "sql"),
            Language::Markdown => write!(f, "markdown"),
            Language::Unknown => write!(f, "unknown"),
        }
    }
}

/// Get Rust language configuration
pub fn rust_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Rust,
        name: "Rust".to_string(),
        extensions: vec![".rs".to_string()],
        tree_sitter_lang: tree_sitter_rust::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}

/// Get Python language configuration
pub fn python_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Python,
        name: "Python".to_string(),
        extensions: vec![".py".to_string(), ".pyw".to_string()],
        tree_sitter_lang: tree_sitter_python::LANGUAGE.into(),
        comment_style: CommentStyle::PythonStyle,
    }
}

/// Get JavaScript language configuration
pub fn javascript_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::JavaScript,
        name: "JavaScript".to_string(),
        extensions: vec![".js".to_string(), ".jsx".to_string(), ".mjs".to_string()],
        tree_sitter_lang: tree_sitter_javascript::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}

/// Get TypeScript language configuration
pub fn typescript_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::TypeScript,
        name: "TypeScript".to_string(),
        extensions: vec![".ts".to_string(), ".tsx".to_string(), ".mts".to_string()],
        tree_sitter_lang: tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        comment_style: CommentStyle::CStyle,
    }
}

/// Get Java language configuration
pub fn java_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Java,
        name: "Java".to_string(),
        extensions: vec![".java".to_string()],
        tree_sitter_lang: tree_sitter_java::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}

/// Get Go language configuration
pub fn go_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Go,
        name: "Go".to_string(),
        extensions: vec![".go".to_string()],
        tree_sitter_lang: tree_sitter_go::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}

/// Get all Top 5 language configurations (for quick setup)
pub fn top5_configs() -> Vec<LanguageConfig> {
    vec![
        rust_config(),
        python_config(),
        javascript_config(),
        java_config(),
        go_config(),
    ]
}

/// Get all language configurations (Top 20)
pub fn all_configs() -> Vec<LanguageConfig> {
    let mut configs = top5_configs();

    // Add more languages as they're implemented
    // C, C++, C#, Swift, Kotlin, PHP, Ruby, Scala, Dart, Lua, R, Perl, Shell, SQL

    configs
}
