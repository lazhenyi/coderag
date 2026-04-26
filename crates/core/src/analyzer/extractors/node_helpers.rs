//! Node extraction helpers

use crate::analyzer::symbol::{Symbol, SymbolKind};

pub fn extract_name(source: &[u8], node: tree_sitter::Node) -> String {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let child_kind = child.kind();
        if matches!(child_kind, "identifier" | "simple_identifier" | "type_identifier") {
            if let Ok(text) = child.utf8_text(source) {
                return text.to_string();
            }
        }
    }
    if let Ok(text) = node.utf8_text(source) {
        return text.split_whitespace().next().unwrap_or("").to_string();
    }
    String::new()
}

pub fn extract_line_numbers(node: tree_sitter::Node) -> (usize, usize) {
    let start = node.start_position();
    let end = node.end_position();
    (start.row + 1, end.row + 1)
}

pub fn extract_code(source: &[u8], node: tree_sitter::Node) -> String {
    let start = node.start_byte();
    let end = node.end_byte();
    if start < source.len() && end <= source.len() && start <= end {
        if let Ok(text) = node.utf8_text(source) {
            return text.to_string();
        }
    }
    String::new()
}

pub fn extract_function_signature(source: &[u8], node: tree_sitter::Node) -> String {
    let mut result = String::new();
    let mut cursor = node.walk();
    result.push_str(&extract_name(source, node));
    for child in node.children(&mut cursor) {
        if matches!(child.kind(), "parameters" | "formal_parameters" | "parameter_list") {
            if let Ok(params) = child.utf8_text(source) {
                result.push('(');
                let clean_params = params.trim_start_matches('(').trim_end_matches(')').trim();
                result.push_str(clean_params);
                result.push(')');
            }
            break;
        }
    }
    result
}

pub fn is_public_node(source: &[u8], node: tree_sitter::Node) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "visibility_modifier" {
            let start = child.start_byte();
            let end = child.end_byte();
            if start < source.len() && end <= source.len() && start <= end {
                if let Ok(text) = child.utf8_text(source) {
                    return text.contains("pub");
                }
            }
        }
        if child.kind() == "export" {
            return true;
        }
    }
    false
}

pub fn is_async_node(node: tree_sitter::Node) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "async" {
            return true;
        }
    }
    false
}

pub fn is_in_class(node: tree_sitter::Node) -> bool {
    let mut current = node;
    loop {
        match current.parent() {
            Some(parent) => {
                if parent.kind() == "class_definition" {
                    return true;
                }
                current = parent;
            }
            None => return false,
        }
    }
}

fn extract_signature(source: &[u8], node: tree_sitter::Node, kind: SymbolKind) -> String {
    match kind {
        SymbolKind::Function | SymbolKind::Method => extract_function_signature(source, node),
        _ => extract_name(source, node),
    }
}

pub fn node_to_symbol(source: &[u8], node: tree_sitter::Node, kind: SymbolKind, file_path: &str) -> Symbol {
    let name = extract_name(source, node);
    let (line_start, line_end) = extract_line_numbers(node);
    let code = extract_code(source, node);
    let signature = extract_signature(source, node, kind.clone());
    let doc = extract_preceding_doc(source, node);
    let mut symbol = Symbol::new(name, kind, line_start, line_end, "unknown".to_string());
    symbol.file_path = file_path.to_string();
    symbol.code = code;
    symbol.signature = signature;
    symbol.doc = doc;
    symbol.is_public = is_public_node(source, node);
    symbol.is_async = is_async_node(node);
    symbol
}

pub fn extract_symbols_by_kind(
    source: &[u8],
    node: tree_sitter::Node,
    kinds: &[(&str, SymbolKind)],
    file_path: &str,
    symbols: &mut Vec<Symbol>,
) {
    let node_kind = node.kind();
    for (kind_str, kind) in kinds {
        if node_kind == *kind_str {
            let symbol = node_to_symbol(source, node, kind.clone(), file_path);
            if !symbol.name.is_empty() {
                symbols.push(symbol);
            }
            break;
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_symbols_by_kind(source, child, kinds, file_path, symbols);
    }
}

// Re-export doc helpers
pub use super::doc_helpers::{extract_preceding_doc, clean_doc_text};
