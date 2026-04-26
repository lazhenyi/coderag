//! Language configs: Tier 2

use super::configs_tier1::*;
use super::lang_enum::Language;
use crate::parser::registry::{CommentStyle, LanguageConfig};

pub fn swift_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Swift,
        name: "Swift".to_string(),
        extensions: vec![".swift".to_string()],
        tree_sitter_lang: tree_sitter_swift::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}

pub fn php_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Php,
        name: "PHP".to_string(),
        extensions: vec![".php".to_string()],
        tree_sitter_lang: tree_sitter_php::LANGUAGE_PHP.into(),
        comment_style: CommentStyle::CStyle,
    }
}

pub fn ruby_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Ruby,
        name: "Ruby".to_string(),
        extensions: vec![".rb".to_string()],
        tree_sitter_lang: tree_sitter_ruby::LANGUAGE.into(),
        comment_style: CommentStyle::ShellStyle,
    }
}

pub fn bash_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Shell,
        name: "Bash".to_string(),
        extensions: vec![".sh".to_string(), ".bash".to_string()],
        tree_sitter_lang: tree_sitter_bash::LANGUAGE.into(),
        comment_style: CommentStyle::ShellStyle,
    }
}

pub fn scala_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Scala,
        name: "Scala".to_string(),
        extensions: vec![".scala".to_string()],
        tree_sitter_lang: tree_sitter_scala::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}

pub fn dart_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Dart,
        name: "Dart".to_string(),
        extensions: vec![".dart".to_string()],
        tree_sitter_lang: tree_sitter_dart::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}

pub fn lua_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Lua,
        name: "Lua".to_string(),
        extensions: vec![".lua".to_string()],
        tree_sitter_lang: tree_sitter_lua::LANGUAGE.into(),
        comment_style: CommentStyle::ShellStyle,
    }
}

pub fn r_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::R,
        name: "R".to_string(),
        extensions: vec![".r".to_string(), ".R".to_string()],
        tree_sitter_lang: tree_sitter_r::LANGUAGE.into(),
        comment_style: CommentStyle::ShellStyle,
    }
}

pub fn perl_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Perl,
        name: "Perl".to_string(),
        extensions: vec![".pl".to_string(), ".pm".to_string()],
        tree_sitter_lang: tree_sitter_perl::LANGUAGE.into(),
        comment_style: CommentStyle::ShellStyle,
    }
}

pub fn kotlin_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Kotlin,
        name: "Kotlin".to_string(),
        extensions: vec![".kt".to_string(), ".kts".to_string()],
        tree_sitter_lang: tree_sitter_kotlin_sg::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}

pub fn sql_config() -> LanguageConfig {
    LanguageConfig {
        language: Language::Sql,
        name: "SQL".to_string(),
        extensions: vec![".sql".to_string()],
        tree_sitter_lang: tree_sitter_sequel::LANGUAGE.into(),
        comment_style: CommentStyle::CStyle,
    }
}

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
