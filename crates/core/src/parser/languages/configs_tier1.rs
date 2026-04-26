//! Language configs: Tier 1 (most popular)

use super::lang_enum::Language;
use crate::parser::registry::{CommentStyle, LanguageConfig};

pub fn rust_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Rust,
        name: "Rust".to_string(),
        extensions: vec![".rs".to_string()],
        tree_sitter_lang: tree_sitter_rust::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}

pub fn python_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Python,
        name: "Python".to_string(),
        extensions: vec![".py".to_string(), ".pyw".to_string()],
        tree_sitter_lang: tree_sitter_python::LANGUAGE.into(),
        comment_style: CommentStyle::PythonStyle,
    }
}

pub fn javascript_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::JavaScript,
        name: "JavaScript".to_string(),
        extensions: vec![".js".to_string(), ".jsx".to_string(), ".mjs".to_string()],
        tree_sitter_lang: tree_sitter_javascript::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}

pub fn typescript_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::TypeScript,
        name: "TypeScript".to_string(),
        extensions: vec![".ts".to_string(), ".tsx".to_string(), ".mts".to_string()],
        tree_sitter_lang: tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        comment_style: CommentStyle::CStyle,
    }
}

pub fn java_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Java,
        name: "Java".to_string(),
        extensions: vec![".java".to_string()],
        tree_sitter_lang: tree_sitter_java::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}

pub fn go_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Go,
        name: "Go".to_string(),
        extensions: vec![".go".to_string()],
        tree_sitter_lang: tree_sitter_go::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}

pub fn c_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::C,
        name: "C".to_string(),
        extensions: vec![".c".to_string(), ".h".to_string()],
        tree_sitter_lang: tree_sitter_c::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}

pub fn cpp_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Cpp,
        name: "C++".to_string(),
        extensions: vec![
            ".cpp".to_string(),
            ".cc".to_string(),
            ".cxx".to_string(),
            ".hpp".to_string(),
            ".hh".to_string(),
        ],
        tree_sitter_lang: tree_sitter_cpp::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}

pub fn csharp_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::CSharp,
        name: "C#".to_string(),
        extensions: vec![".cs".to_string()],
        tree_sitter_lang: tree_sitter_c_sharp::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}
