//! Parser types and enums

/// Style of comments for a language
#[derive(Debug, Clone, Copy)]
pub enum CommentStyle {
    CStyle,
    PythonStyle,
    SqlStyle,
    ShellStyle,
}

/// Language configuration with metadata and file associations
#[derive(Debug, Clone)]
pub struct LanguageConfig {
    pub language: crate::parser::languages::Language,
    pub name: String,
    pub extensions: Vec<String>,
    pub tree_sitter_lang: tree_sitter::Language,
    pub comment_style: CommentStyle,
}

impl crate::parser::languages::Language {
    pub fn extensions(&self) -> Vec<&'static str> {
        match self {
            Self::Rust => vec![".rs"],
            Self::Python => vec![".py", ".pyw"],
            Self::JavaScript => vec![".js", ".jsx", ".mjs"],
            Self::TypeScript => vec![".ts", ".tsx", ".mts"],
            Self::Java => vec![".java"],
            Self::Go => vec![".go"],
            Self::C => vec![".c", ".h"],
            Self::Cpp => vec![".cpp", ".cc", ".cxx", ".hpp", ".hh"],
            Self::CSharp => vec![".cs"],
            Self::Swift => vec![".swift"],
            Self::Kotlin => vec![".kt", ".kts"],
            Self::Php => vec![".php"],
            Self::Ruby => vec![".rb"],
            Self::Scala => vec![".scala"],
            Self::Dart => vec![".dart"],
            Self::Lua => vec![".lua"],
            Self::R => vec![".r", ".R"],
            Self::Perl => vec![".pl", ".pm"],
            Self::Shell => vec![".sh", ".bash"],
            Self::Sql => vec![".sql"],
            Self::Markdown => vec![".md"],
            Self::Unknown => vec![],
        }
    }
}
