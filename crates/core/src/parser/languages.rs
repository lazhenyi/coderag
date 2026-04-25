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

/// Get C language configuration
pub fn c_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::C,
        name: "C".to_string(),
        extensions: vec![".c".to_string(), ".h".to_string()],
        tree_sitter_lang: tree_sitter_c::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}

/// Get C++ language configuration
pub fn cpp_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Cpp,
        name: "C++".to_string(),
        extensions: vec![".cpp".to_string(), ".cc".to_string(), ".cxx".to_string(), ".hpp".to_string(), ".hh".to_string()],
        tree_sitter_lang: tree_sitter_cpp::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}

/// Get C# language configuration
pub fn csharp_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::CSharp,
        name: "C#".to_string(),
        extensions: vec![".cs".to_string()],
        tree_sitter_lang: tree_sitter_c_sharp::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}

/// Get Swift language configuration
pub fn swift_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Swift,
        name: "Swift".to_string(),
        extensions: vec![".swift".to_string()],
        tree_sitter_lang: tree_sitter_swift::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}

/// Get PHP language configuration
pub fn php_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Php,
        name: "PHP".to_string(),
        extensions: vec![".php".to_string()],
        tree_sitter_lang: tree_sitter_php::LANGUAGE_PHP.into(),
        comment_style: CommentStyle::CStyle,
    }
}

/// Get Ruby language configuration
pub fn ruby_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Ruby,
        name: "Ruby".to_string(),
        extensions: vec![".rb".to_string()],
        tree_sitter_lang: tree_sitter_ruby::LANGUAGE.into(),
        comment_style: CommentStyle::ShellStyle,
    }
}

/// Get Bash language configuration
pub fn bash_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Shell,
        name: "Bash".to_string(),
        extensions: vec![".sh".to_string(), ".bash".to_string()],
        tree_sitter_lang: tree_sitter_bash::LANGUAGE.into(),
        comment_style: CommentStyle::ShellStyle,
    }
}

/// Get Scala language configuration
pub fn scala_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Scala,
        name: "Scala".to_string(),
        extensions: vec![".scala".to_string()],
        tree_sitter_lang: tree_sitter_scala::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}

/// Get Dart language configuration
pub fn dart_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Dart,
        name: "Dart".to_string(),
        extensions: vec![".dart".to_string()],
        tree_sitter_lang: tree_sitter_dart::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}

/// Get Lua language configuration
pub fn lua_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Lua,
        name: "Lua".to_string(),
        extensions: vec![".lua".to_string()],
        tree_sitter_lang: tree_sitter_lua::LANGUAGE.into(),
        comment_style: CommentStyle::ShellStyle,
    }
}

/// Get R language configuration
pub fn r_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::R,
        name: "R".to_string(),
        extensions: vec![".r".to_string(), ".R".to_string()],
        tree_sitter_lang: tree_sitter_r::LANGUAGE.into(),
        comment_style: CommentStyle::ShellStyle,
    }
}

/// Get Perl language configuration
pub fn perl_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Perl,
        name: "Perl".to_string(),
        extensions: vec![".pl".to_string(), ".pm".to_string()],
        tree_sitter_lang: tree_sitter_perl::LANGUAGE.into(),
        comment_style: CommentStyle::ShellStyle,
    }
}

/// Get Kotlin language configuration
pub fn kotlin_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Kotlin,
        name: "Kotlin".to_string(),
        extensions: vec![".kt".to_string(), ".kts".to_string()],
        tree_sitter_lang: tree_sitter_kotlin_sg::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}

/// Get SQL language configuration
pub fn sql_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Sql,
        name: "SQL".to_string(),
        extensions: vec![".sql".to_string()],
        tree_sitter_lang: tree_sitter_sequel::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}

/// Get all language configurations (Top 20)
#[allow(dead_code)]
pub fn all_configs() -> Vec<LanguageConfig> {
    vec![
        rust_config(),
        python_config(),
        javascript_config(),
        typescript_config(),
        java_config(),
        go_config(),
        c_config(),
        cpp_config(),
        csharp_config(),
        swift_config(),
        php_config(),
        ruby_config(),
        bash_config(),
        scala_config(),
        dart_config(),
        lua_config(),
        r_config(),
        perl_config(),
        kotlin_config(),
        sql_config(),
    ]
}
