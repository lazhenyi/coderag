//! Symbol Extractor
//!
//! Dispatches to language-specific extractors.

use crate::analyzer::extractors::node_helpers::extract_symbols_by_kind;
use crate::analyzer::extractors::{
    extract_bash, extract_c, extract_cpp, extract_csharp, extract_go, extract_java,
    extract_javascript, extract_php, extract_python, extract_ruby, extract_rust, extract_swift,
};
use crate::analyzer::symbol::{Symbol, SymbolKind, SymbolScope};

/// Extracts symbols from parsed source code
pub struct SymbolExtractor {
    _scope_stack: Vec<SymbolScope>,
}

impl SymbolExtractor {
    pub fn new() -> Self {
        Self {
            _scope_stack: Vec::new(),
        }
    }

    pub fn extract(
        &self,
        source: &[u8],
        tree: &tree_sitter::Tree,
        language: &crate::parser::Language,
        file_path: &str,
    ) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let root = tree.root_node();

        match language {
            crate::parser::Language::Rust => extract_rust(source, root, file_path, &mut symbols),
            crate::parser::Language::Python => {
                extract_python(source, root, file_path, &mut symbols)
            }
            crate::parser::Language::JavaScript | crate::parser::Language::TypeScript => {
                extract_javascript(source, root, file_path, &mut symbols)
            }
            crate::parser::Language::Java => extract_java(source, root, file_path, &mut symbols),
            crate::parser::Language::Go => extract_go(source, root, file_path, &mut symbols),
            crate::parser::Language::C => extract_c(source, root, file_path, &mut symbols),
            crate::parser::Language::Cpp => extract_cpp(source, root, file_path, &mut symbols),
            crate::parser::Language::CSharp => {
                extract_csharp(source, root, file_path, &mut symbols)
            }
            crate::parser::Language::Swift => extract_swift(source, root, file_path, &mut symbols),
            crate::parser::Language::Php => extract_php(source, root, file_path, &mut symbols),
            crate::parser::Language::Ruby => extract_ruby(source, root, file_path, &mut symbols),
            crate::parser::Language::Shell => extract_bash(source, root, file_path, &mut symbols),
            crate::parser::Language::Scala => extract_symbols_by_kind(
                source,
                root,
                &[
                    ("function_definition", SymbolKind::Function),
                    ("class_definition", SymbolKind::Class),
                    ("trait_definition", SymbolKind::Trait),
                    ("object_definition", SymbolKind::Class),
                ],
                file_path,
                &mut symbols,
            ),
            crate::parser::Language::Dart => extract_symbols_by_kind(
                source,
                root,
                &[
                    ("function_declaration", SymbolKind::Function),
                    ("class_declaration", SymbolKind::Class),
                    ("method_declaration", SymbolKind::Method),
                    ("enum_declaration", SymbolKind::Enum),
                    ("mixin_declaration", SymbolKind::Trait),
                ],
                file_path,
                &mut symbols,
            ),
            crate::parser::Language::Lua => extract_symbols_by_kind(
                source,
                root,
                &[("function_declaration", SymbolKind::Function)],
                file_path,
                &mut symbols,
            ),
            crate::parser::Language::R => extract_symbols_by_kind(
                source,
                root,
                &[
                    ("function_definition", SymbolKind::Function),
                    ("class_definition", SymbolKind::Class),
                ],
                file_path,
                &mut symbols,
            ),
            crate::parser::Language::Perl => extract_symbols_by_kind(
                source,
                root,
                &[
                    ("function", SymbolKind::Function),
                    ("package", SymbolKind::Module),
                    ("class", SymbolKind::Class),
                ],
                file_path,
                &mut symbols,
            ),
            crate::parser::Language::Kotlin => extract_symbols_by_kind(
                source,
                root,
                &[
                    ("function_declaration", SymbolKind::Function),
                    ("class_declaration", SymbolKind::Class),
                    ("object_declaration", SymbolKind::Class),
                    ("type_alias", SymbolKind::TypeAlias),
                ],
                file_path,
                &mut symbols,
            ),
            crate::parser::Language::Sql => extract_symbols_by_kind(
                source,
                root,
                &[
                    ("create_function", SymbolKind::Function),
                    ("create_procedure", SymbolKind::Function),
                    ("create_table", SymbolKind::Class),
                    ("create_view", SymbolKind::Struct),
                ],
                file_path,
                &mut symbols,
            ),
            _ => {}
        }

        symbols
    }
}
