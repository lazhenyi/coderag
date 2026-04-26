//! Rust symbol extraction

use crate::analyzer::extractors::node_helpers::node_to_symbol;
use crate::analyzer::symbol::SymbolKind;

/// Extract Rust symbols
pub fn extract_rust(
    source: &[u8],
    node: tree_sitter::Node,
    file_path: &str,
    symbols: &mut Vec<crate::analyzer::Symbol>,
) {
    let symbol_kinds = [
        ("function_item", SymbolKind::Function),
        ("struct_item", SymbolKind::Struct),
        ("enum_item", SymbolKind::Enum),
        ("trait_item", SymbolKind::Trait),
        ("type_alias_item", SymbolKind::TypeAlias),
        ("mod_item", SymbolKind::Module),
        ("const_item", SymbolKind::Constant),
        ("static_item", SymbolKind::Variable),
    ];

    let node_kind = node.kind();

    if node_kind == "impl_item" {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "declaration_list" {
                let mut decl_cursor = child.walk();
                for decl in child.children(&mut decl_cursor) {
                    if decl.kind() == "function_item" {
                        let symbol = node_to_symbol(source, decl, SymbolKind::Method, file_path);
                        if !symbol.name.is_empty() {
                            symbols.push(symbol);
                        }
                    }
                }
            }
        }
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_rust(source, child, file_path, symbols);
    }

    for (kind_str, kind) in &symbol_kinds {
        if node_kind == *kind_str {
            let symbol = node_to_symbol(source, node, kind.clone(), file_path);
            if !symbol.name.is_empty() {
                symbols.push(symbol);
            }
            break;
        }
    }
}
