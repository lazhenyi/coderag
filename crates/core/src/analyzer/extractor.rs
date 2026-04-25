//! Symbol Extractor
//!
//! Extracts symbols from parsed AST using node traversal.

use crate::analyzer::symbol::{Symbol, SymbolKind, SymbolScope};

/// Extracts symbols from parsed source code
pub struct SymbolExtractor {
    /// Current scope stack
    scope_stack: Vec<SymbolScope>,
}

impl SymbolExtractor {
    /// Create a new symbol extractor
    pub fn new() -> Self {
        Self {
            scope_stack: Vec::new(),
        }
    }

    /// Extract all symbols from parsed tree
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
            crate::parser::Language::Rust => self.extract_rust(source, root, file_path, &mut symbols),
            crate::parser::Language::Python => self.extract_python(source, root, file_path, &mut symbols),
            crate::parser::Language::JavaScript | crate::parser::Language::TypeScript => {
                self.extract_javascript(source, root, file_path, &mut symbols)
            }
            crate::parser::Language::Java => self.extract_java(source, root, file_path, &mut symbols),
            crate::parser::Language::Go => self.extract_go(source, root, file_path, &mut symbols),
            _ => {}
        }

        symbols
    }

    /// Extract Rust symbols
    fn extract_rust(&self, source: &[u8], node: tree_sitter::Node, file_path: &str, symbols: &mut Vec<Symbol>) {
        let symbol_kinds = [
            ("function_item", SymbolKind::Function),
            ("struct_item", SymbolKind::Struct),
            ("enum_item", SymbolKind::Enum),
            ("trait_item", SymbolKind::Trait),
            ("impl_item", SymbolKind::Trait),
            ("mod_item", SymbolKind::Module),
            ("type_alias_item", SymbolKind::TypeAlias),
        ];

        self.extract_symbols_by_kind(source, node, &symbol_kinds, file_path, symbols);
    }

    /// Extract Python symbols
    fn extract_python(&self, source: &[u8], node: tree_sitter::Node, file_path: &str, symbols: &mut Vec<Symbol>) {
        let symbol_kinds = [
            ("function_definition", SymbolKind::Function),
            ("class_definition", SymbolKind::Class),
            ("async_function_definition", SymbolKind::Function),
        ];

        self.extract_symbols_by_kind(source, node, &symbol_kinds, file_path, symbols);
    }

    /// Extract JavaScript/TypeScript symbols
    fn extract_javascript(&self, source: &[u8], node: tree_sitter::Node, file_path: &str, symbols: &mut Vec<Symbol>) {
        let symbol_kinds = [
            ("function_declaration", SymbolKind::Function),
            ("class_declaration", SymbolKind::Class),
            ("method_definition", SymbolKind::Method),
        ];

        self.extract_symbols_by_kind(source, node, &symbol_kinds, file_path, symbols);
    }

    /// Extract Java symbols
    fn extract_java(&self, source: &[u8], node: tree_sitter::Node, file_path: &str, symbols: &mut Vec<Symbol>) {
        let symbol_kinds = [
            ("class_declaration", SymbolKind::Class),
            ("interface_declaration", SymbolKind::Interface),
            ("method_declaration", SymbolKind::Method),
            ("constructor_declaration", SymbolKind::Method),
            ("enum_declaration", SymbolKind::Enum),
        ];

        self.extract_symbols_by_kind(source, node, &symbol_kinds, file_path, symbols);
    }

    /// Extract Go symbols
    fn extract_go(&self, source: &[u8], node: tree_sitter::Node, file_path: &str, symbols: &mut Vec<Symbol>) {
        let symbol_kinds = [
            ("function_declaration", SymbolKind::Function),
            ("method_declaration", SymbolKind::Method),
            ("type_declaration", SymbolKind::Struct),
            ("const_declaration", SymbolKind::Constant),
        ];

        self.extract_symbols_by_kind(source, node, &symbol_kinds, file_path, symbols);
    }

    /// Extract symbols by node kind
    fn extract_symbols_by_kind(
        &self,
        source: &[u8],
        node: tree_sitter::Node,
        kinds: &[(&str, SymbolKind)],
        file_path: &str,
        symbols: &mut Vec<Symbol>,
    ) {
        let node_kind = node.kind();

        for (kind_str, kind) in kinds {
            if node_kind == *kind_str {
                let symbol = self.node_to_symbol(source, node, kind.clone(), file_path);
                if !symbol.name.is_empty() {
                    symbols.push(symbol);
                }
                break;
            }
        }

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_symbols_by_kind(source, child, kinds, file_path, symbols);
        }
    }

    /// Convert a tree-sitter node to a symbol
    fn node_to_symbol(&self, source: &[u8], node: tree_sitter::Node, kind: SymbolKind, file_path: &str) -> Symbol {
        let name = self.extract_name(source, node);
        let (line_start, line_end) = self.extract_line_numbers(node);
        let code = self.extract_code(source, node);
        let signature = self.extract_signature(source, node, kind.clone());

        let mut symbol = Symbol::new(
            name,
            kind,
            line_start,
            line_end,
            "unknown".to_string(),
        );

        symbol.file_path = file_path.to_string();
        symbol.code = code;
        symbol.signature = signature;

        // Check for visibility
        symbol.is_public = self.is_public_node(node);

        // Check for async
        symbol.is_async = self.is_async_node(node);

        symbol
    }

    /// Extract the name from a node
    fn extract_name(&self, source: &[u8], node: tree_sitter::Node) -> String {
        let mut cursor = node.walk();

        // Try to find identifier nodes directly
        for child in node.children(&mut cursor) {
            let child_kind = child.kind();
            if child_kind == "identifier" || child_kind == "type_identifier" {
                if let Ok(text) = child.utf8_text(source) {
                    return text.to_string();
                }
            }
        }

        // Fallback: use the first word (identifier)
        if let Ok(text) = node.utf8_text(source) {
            return text.split_whitespace().next().unwrap_or("").to_string();
        }

        String::new()
    }

    /// Extract line numbers from a node
    fn extract_line_numbers(&self, node: tree_sitter::Node) -> (usize, usize) {
        let start = node.start_position();
        let end = node.end_position();
        (start.row + 1, end.row + 1) // 1-indexed lines
    }

    /// Extract source code for a node
    fn extract_code(&self, source: &[u8], node: tree_sitter::Node) -> String {
        node.utf8_text(source)
            .map(|s| s.to_string())
            .unwrap_or_default()
    }

    /// Extract signature from a node
    fn extract_signature(&self, source: &[u8], node: tree_sitter::Node, kind: SymbolKind) -> String {
        match kind {
            SymbolKind::Function | SymbolKind::Method => {
                self.extract_function_signature(source, node)
            }
            _ => {
                self.extract_name(source, node)
            }
        }
    }

    /// Extract function signature
    fn extract_function_signature(&self, source: &[u8], node: tree_sitter::Node) -> String {
        let mut result = String::new();
        let mut cursor = node.walk();

        // Get the function name
        result.push_str(&self.extract_name(source, node));

        // Find and extract parameters
        for child in node.children(&mut cursor) {
            if child.kind() == "parameters" || child.kind() == "formal_parameters" || child.kind() == "parameter_list" {
                if let Ok(params) = child.utf8_text(source) {
                    result.push('(');
                    let clean_params = params
                        .trim_start_matches("(")
                        .trim_end_matches(")")
                        .trim();
                    result.push_str(clean_params);
                    result.push(')');
                }
                break;
            }
        }

        result
    }

    /// Check if a node represents a public declaration
    fn is_public_node(&self, node: tree_sitter::Node) -> bool {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "visibility_modifier" {
                if let Ok(text) = child.utf8_text(&[]) {
                    return text.contains("pub");
                }
            }
            if child.kind() == "export" {
                return true;
            }
        }

        false
    }

    /// Check if a node represents an async function
    fn is_async_node(&self, node: tree_sitter::Node) -> bool {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "async" {
                return true;
            }
        }

        false
    }
}
