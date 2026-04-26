//! Language enum definition

use crate::parser::registry::CommentStyle;

/// Supported programming languages
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Language {
    Rust, Python, JavaScript, TypeScript, Java, Go,
    C, Cpp, CSharp, Swift, Kotlin, Php, Ruby,
    Scala, Dart, Lua, R, Perl, Shell, Sql, Markdown, Unknown,
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
