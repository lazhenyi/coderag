//! Java, JavaScript, PHP, Ruby, Bash symbol extraction

use crate::analyzer::extractors::node_helpers::extract_symbols_by_kind;
use crate::analyzer::symbol::SymbolKind;

/// Extract JavaScript/TypeScript symbols
pub fn extract_javascript(
    source: &[u8],
    node: tree_sitter::Node,
    file_path: &str,
    symbols: &mut Vec<crate::analyzer::Symbol>,
) {
    let symbol_kinds = [
        ("function_declaration", SymbolKind::Function),
        ("class_declaration", SymbolKind::Class),
        ("method_definition", SymbolKind::Method),
        ("interface_declaration", SymbolKind::Interface),
    ];
    extract_symbols_by_kind(source, node, &symbol_kinds, file_path, symbols);
}

/// Extract Java symbols
pub fn extract_java(
    source: &[u8],
    node: tree_sitter::Node,
    file_path: &str,
    symbols: &mut Vec<crate::analyzer::Symbol>,
) {
    let symbol_kinds = [
        ("class_declaration", SymbolKind::Class),
        ("interface_declaration", SymbolKind::Interface),
        ("method_declaration", SymbolKind::Method),
        ("constructor_declaration", SymbolKind::Method),
        ("enum_declaration", SymbolKind::Enum),
    ];
    extract_symbols_by_kind(source, node, &symbol_kinds, file_path, symbols);
}

/// Extract PHP symbols
pub fn extract_php(
    source: &[u8],
    node: tree_sitter::Node,
    file_path: &str,
    symbols: &mut Vec<crate::analyzer::Symbol>,
) {
    let symbol_kinds = [
        ("function_definition", SymbolKind::Function),
        ("class_declaration", SymbolKind::Class),
        ("method_declaration", SymbolKind::Method),
        ("interface_declaration", SymbolKind::Interface),
        ("trait_declaration", SymbolKind::Trait),
        ("enum_declaration", SymbolKind::Enum),
    ];
    extract_symbols_by_kind(source, node, &symbol_kinds, file_path, symbols);
}

/// Extract Ruby symbols
pub fn extract_ruby(
    source: &[u8],
    node: tree_sitter::Node,
    file_path: &str,
    symbols: &mut Vec<crate::analyzer::Symbol>,
) {
    let symbol_kinds = [
        ("method", SymbolKind::Method),
        ("class", SymbolKind::Class),
        ("module", SymbolKind::Module),
    ];
    extract_symbols_by_kind(source, node, &symbol_kinds, file_path, symbols);
}

/// Extract Bash symbols
pub fn extract_bash(
    source: &[u8],
    node: tree_sitter::Node,
    file_path: &str,
    symbols: &mut Vec<crate::analyzer::Symbol>,
) {
    let symbol_kinds = [("function_definition", SymbolKind::Function)];
    extract_symbols_by_kind(source, node, &symbol_kinds, file_path, symbols);
}
