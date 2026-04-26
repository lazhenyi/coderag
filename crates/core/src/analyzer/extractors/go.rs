//! Go symbol extraction

use crate::analyzer::symbol::SymbolKind;
use crate::analyzer::extractors::node_helpers::{extract_name, node_to_symbol};

/// Extract Go symbols
pub fn extract_go(source: &[u8], node: tree_sitter::Node, file_path: &str, symbols: &mut Vec<crate::analyzer::Symbol>) {
    let node_kind = node.kind();

    match node_kind {
        "function_declaration" => {
            let symbol = node_to_symbol(source, node, SymbolKind::Function, file_path);
            if !symbol.name.is_empty() {
                symbols.push(symbol);
            }
        }
        "method_declaration" => {
            let mut symbol = node_to_symbol(source, node, SymbolKind::Method, file_path);
            let method_name = extract_method_name(source, node);
            if !method_name.is_empty() {
                symbol.name = method_name;
                symbols.push(symbol);
            }
        }
        "type_declaration" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "type_spec" {
                    let mut ts_cursor = child.walk();
                    for ts_child in child.children(&mut ts_cursor) {
                        let type_name = extract_type_name(source, node);
                        if ts_child.kind() == "struct_type" && !type_name.is_empty() {
                            let mut symbol = node_to_symbol(source, node, SymbolKind::Struct, file_path);
                            symbol.name = type_name;
                            symbols.push(symbol);
                        } else if ts_child.kind() == "interface_type" && !type_name.is_empty() {
                            let mut symbol = node_to_symbol(source, node, SymbolKind::Interface, file_path);
                            symbol.name = type_name;
                            symbols.push(symbol);
                        }
                    }
                }
            }
        }
        "const_declaration" => {
            let symbol = node_to_symbol(source, node, SymbolKind::Constant, file_path);
            if !symbol.name.is_empty() {
                symbols.push(symbol);
            }
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_go(source, child, file_path, symbols);
    }
}

fn extract_method_name(source: &[u8], node: tree_sitter::Node) -> String {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let child_kind = child.kind();
        if child_kind == "identifier" || child_kind == "field_identifier" {
            if let Some(parent) = child.parent() {
                if parent.kind() == "receiver" {
                    continue;
                }
            }
            if let Ok(text) = child.utf8_text(source) {
                return text.to_string();
            }
        }
    }
    String::new()
}

fn extract_type_name(source: &[u8], node: tree_sitter::Node) -> String {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "type_spec" {
            let mut ts_cursor = child.walk();
            for ts_child in child.children(&mut ts_cursor) {
                if ts_child.kind() == "type_identifier" || ts_child.kind() == "identifier" {
                    if let Ok(text) = ts_child.utf8_text(source) {
                        return text.to_string();
                    }
                }
            }
        }
    }
    extract_name(source, node)
}
