//! Python symbol extraction

use crate::analyzer::symbol::SymbolKind;
use crate::analyzer::extractors::node_helpers::{node_to_symbol, is_in_class};

/// Extract Python symbols
pub fn extract_python(source: &[u8], node: tree_sitter::Node, file_path: &str, symbols: &mut Vec<crate::analyzer::Symbol>) {
    let node_kind = node.kind();

    if node_kind == "class_definition" {
        let symbol = node_to_symbol(source, node, SymbolKind::Class, file_path);
        if !symbol.name.is_empty() {
            symbols.push(symbol);
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            extract_python_in_class(source, child, file_path, symbols);
        }
        return;
    }

    if node_kind == "function_definition" || node_kind == "async_function_definition" {
        let kind = if is_in_class(node) { SymbolKind::Method } else { SymbolKind::Function };
        let symbol = node_to_symbol(source, node, kind, file_path);
        if !symbol.name.is_empty() {
            symbols.push(symbol);
        }
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_python(source, child, file_path, symbols);
    }
}

fn extract_python_in_class(source: &[u8], node: tree_sitter::Node, file_path: &str, symbols: &mut Vec<crate::analyzer::Symbol>) {
    let node_kind = node.kind();

    if node_kind == "function_definition" || node_kind == "async_function_definition" {
        let symbol = node_to_symbol(source, node, SymbolKind::Method, file_path);
        if !symbol.name.is_empty() {
            symbols.push(symbol);
        }
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_python_in_class(source, child, file_path, symbols);
    }
}
