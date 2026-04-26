//! C# and Swift symbol extraction

use crate::analyzer::extractors::node_helpers::{extract_symbols_by_kind, node_to_symbol};
use crate::analyzer::symbol::SymbolKind;

/// Extract C# symbols
pub fn extract_csharp(
    source: &[u8],
    node: tree_sitter::Node,
    file_path: &str,
    symbols: &mut Vec<crate::analyzer::Symbol>,
) {
    match node.kind() {
        "class_declaration"
        | "interface_declaration"
        | "method_declaration"
        | "struct_declaration"
        | "enum_declaration"
        | "namespace_declaration" => {
            let kind = match node.kind() {
                "class_declaration" => SymbolKind::Class,
                "interface_declaration" => SymbolKind::Interface,
                "method_declaration" => SymbolKind::Method,
                "struct_declaration" => SymbolKind::Struct,
                "enum_declaration" => SymbolKind::Enum,
                "namespace_declaration" => SymbolKind::Module,
                _ => return,
            };
            let symbol = node_to_symbol(source, node, kind, file_path);
            if !symbol.name.is_empty() {
                symbols.push(symbol);
            }
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_csharp(source, child, file_path, symbols);
    }
}

/// Extract Swift symbols
pub fn extract_swift(
    source: &[u8],
    node: tree_sitter::Node,
    file_path: &str,
    symbols: &mut Vec<crate::analyzer::Symbol>,
) {
    let symbol_kinds = [
        ("function_declaration", SymbolKind::Function),
        ("enum_declaration", SymbolKind::Enum),
        ("protocol_declaration", SymbolKind::Interface),
    ];
    extract_symbols_by_kind(source, node, &symbol_kinds, file_path, symbols);
}
