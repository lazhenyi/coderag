//! C and C++ symbol extraction

use crate::analyzer::symbol::SymbolKind;
use crate::analyzer::extractors::node_helpers::node_to_symbol;

/// Extract C symbols
pub fn extract_c(source: &[u8], node: tree_sitter::Node, file_path: &str, symbols: &mut Vec<crate::analyzer::Symbol>) {
    match node.kind() {
        "function_definition" | "struct_specifier" | "union_specifier" | "enum_specifier"
        | "type_definition" | "preproc_def" => {
            let kind = match node.kind() {
                "function_definition" => SymbolKind::Function,
                "struct_specifier" | "union_specifier" => SymbolKind::Struct,
                "enum_specifier" => SymbolKind::Enum,
                "type_definition" => SymbolKind::TypeAlias,
                "preproc_def" => SymbolKind::Constant,
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
        extract_c(source, child, file_path, symbols);
    }
}

/// Extract C++ symbols
pub fn extract_cpp(source: &[u8], node: tree_sitter::Node, file_path: &str, symbols: &mut Vec<crate::analyzer::Symbol>) {
    match node.kind() {
        "function_definition" | "class_specifier" | "struct_specifier" | "enum_specifier"
        | "namespace_definition" | "template_declaration" | "type_definition" => {
            let kind = match node.kind() {
                "function_definition" | "template_declaration" => SymbolKind::Function,
                "class_specifier" => SymbolKind::Class,
                "struct_specifier" => SymbolKind::Struct,
                "enum_specifier" => SymbolKind::Enum,
                "namespace_definition" => SymbolKind::Module,
                "type_definition" => SymbolKind::TypeAlias,
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
        extract_cpp(source, child, file_path, symbols);
    }
}
