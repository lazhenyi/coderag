//! Symbol Extractor
//!
//! Extracts symbols from parsed AST using node traversal.

use crate::analyzer::symbol::{Symbol, SymbolKind, SymbolScope};

/// Extracts symbols from parsed source code
pub struct SymbolExtractor {
    /// Current scope stack (for future module path support)
    _scope_stack: Vec<SymbolScope>,
}

impl SymbolExtractor {
    /// Create a new symbol extractor
    pub fn new() -> Self {
        Self {
            _scope_stack: Vec::new(),
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
            crate::parser::Language::C => self.extract_c(source, root, file_path, &mut symbols),
            crate::parser::Language::Cpp => self.extract_cpp(source, root, file_path, &mut symbols),
            crate::parser::Language::CSharp => self.extract_csharp(source, root, file_path, &mut symbols),
            crate::parser::Language::Swift => self.extract_swift(source, root, file_path, &mut symbols),
            crate::parser::Language::Php => self.extract_php(source, root, file_path, &mut symbols),
            crate::parser::Language::Ruby => self.extract_ruby(source, root, file_path, &mut symbols),
            crate::parser::Language::Shell => self.extract_bash(source, root, file_path, &mut symbols),
            crate::parser::Language::Scala => self.extract_symbols_by_kind(source, root, &[
                ("function_definition", SymbolKind::Function),
                ("class_definition", SymbolKind::Class),
                ("trait_definition", SymbolKind::Trait),
                ("object_definition", SymbolKind::Class),
            ], file_path, &mut symbols),
            crate::parser::Language::Dart => self.extract_symbols_by_kind(source, root, &[
                ("function_declaration", SymbolKind::Function),
                ("class_declaration", SymbolKind::Class),
                ("method_declaration", SymbolKind::Method),
                ("enum_declaration", SymbolKind::Enum),
                ("mixin_declaration", SymbolKind::Trait),
            ], file_path, &mut symbols),
            crate::parser::Language::Lua => self.extract_symbols_by_kind(source, root, &[
                ("function_declaration", SymbolKind::Function),
            ], file_path, &mut symbols),
            crate::parser::Language::R => self.extract_symbols_by_kind(source, root, &[
                ("function_definition", SymbolKind::Function),
                ("class_definition", SymbolKind::Class),
            ], file_path, &mut symbols),
            crate::parser::Language::Perl => self.extract_symbols_by_kind(source, root, &[
                ("function", SymbolKind::Function),
                ("package", SymbolKind::Module),
                ("class", SymbolKind::Class),
            ], file_path, &mut symbols),
            crate::parser::Language::Kotlin => self.extract_symbols_by_kind(source, root, &[
                ("function_declaration", SymbolKind::Function),
                ("class_declaration", SymbolKind::Class),
                ("object_declaration", SymbolKind::Class),
                ("type_alias", SymbolKind::TypeAlias),
            ], file_path, &mut symbols),
            crate::parser::Language::Sql => self.extract_symbols_by_kind(source, root, &[
                ("create_function", SymbolKind::Function),
                ("create_procedure", SymbolKind::Function),
                ("create_table", SymbolKind::Class),
                ("create_view", SymbolKind::Struct),
            ], file_path, &mut symbols),
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
            ("type_alias_item", SymbolKind::TypeAlias),
            ("mod_item", SymbolKind::Module),
            ("const_item", SymbolKind::Constant),
            ("static_item", SymbolKind::Variable),
        ];

        let node_kind = node.kind();

        // Handle impl_item: functions inside declaration_list are methods
        if node_kind == "impl_item" {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "declaration_list" {
                    let mut decl_cursor = child.walk();
                    for decl in child.children(&mut decl_cursor) {
                        if decl.kind() == "function_item" {
                            let symbol = self.node_to_symbol(source, decl, SymbolKind::Method, file_path);
                            if !symbol.name.is_empty() {
                                symbols.push(symbol);
                            }
                        }
                    }
                }
            }
            return;
        }

        // Recurse into children first
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_rust(source, child, file_path, symbols);
        }

        // Check if this node itself is a symbol (post-order: inner before outer)
        for (kind_str, kind) in &symbol_kinds {
            if node_kind == *kind_str {
                let symbol = self.node_to_symbol(source, node, kind.clone(), file_path);
                if !symbol.name.is_empty() {
                    symbols.push(symbol);
                }
                break;
            }
        }
    }

    /// Extract Python symbols
    fn extract_python(&self, source: &[u8], node: tree_sitter::Node, file_path: &str, symbols: &mut Vec<Symbol>) {
        let node_kind = node.kind();

        // Handle class_definition: add class, and its function children as methods
        if node_kind == "class_definition" {
            let symbol = self.node_to_symbol(source, node, SymbolKind::Class, file_path);
            if !symbol.name.is_empty() {
                symbols.push(symbol);
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                self.extract_python_in_class(source, child, file_path, symbols);
            }
            return;
        }

        // Top-level function_definition
        if node_kind == "function_definition" || node_kind == "async_function_definition" {
            let kind = if self.is_in_class(node) { SymbolKind::Method } else { SymbolKind::Function };
            let symbol = self.node_to_symbol(source, node, kind, file_path);
            if !symbol.name.is_empty() {
                symbols.push(symbol);
            }
            return;
        }

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_python(source, child, file_path, symbols);
        }
    }

    /// Extract children of a class_definition, marking functions as methods
    fn extract_python_in_class(&self, source: &[u8], node: tree_sitter::Node, file_path: &str, symbols: &mut Vec<Symbol>) {
        let node_kind = node.kind();

        if node_kind == "function_definition" || node_kind == "async_function_definition" {
            let symbol = self.node_to_symbol(source, node, SymbolKind::Method, file_path);
            if !symbol.name.is_empty() {
                symbols.push(symbol);
            }
            return;
        }

        // Recurse
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_python_in_class(source, child, file_path, symbols);
        }
    }

    /// Check if a node has a class_definition ancestor
    fn is_in_class(&self, node: tree_sitter::Node) -> bool {
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

    /// Extract JavaScript/TypeScript symbols
    fn extract_javascript(&self, source: &[u8], node: tree_sitter::Node, file_path: &str, symbols: &mut Vec<Symbol>) {
        let symbol_kinds = [
            ("function_declaration", SymbolKind::Function),
            ("class_declaration", SymbolKind::Class),
            ("method_definition", SymbolKind::Method),
            ("interface_declaration", SymbolKind::Interface),
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
        let node_kind = node.kind();

        match node_kind {
            "function_declaration" => {
                let symbol = self.node_to_symbol(source, node, SymbolKind::Function, file_path);
                if !symbol.name.is_empty() {
                    symbols.push(symbol);
                }
            }
            "method_declaration" => {
                // For methods, skip the receiver identifier
                let mut symbol = self.node_to_symbol(source, node, SymbolKind::Method, file_path);
                let method_name = self.extract_method_name(source, node);
                if !method_name.is_empty() {
                    symbol.name = method_name;
                    symbols.push(symbol);
                }
            }
            "type_declaration" => {
                // Check the type_spec child to determine struct vs interface
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "type_spec" {
                        let mut ts_cursor = child.walk();
                        for ts_child in child.children(&mut ts_cursor) {
                            let type_name = self.extract_type_name(source, node);
                            if ts_child.kind() == "struct_type" && !type_name.is_empty() {
                                let mut symbol = self.node_to_symbol(source, node, SymbolKind::Struct, file_path);
                                symbol.name = type_name;
                                symbols.push(symbol);
                            } else if ts_child.kind() == "interface_type" && !type_name.is_empty() {
                                let mut symbol = self.node_to_symbol(source, node, SymbolKind::Interface, file_path);
                                symbol.name = type_name;
                                symbols.push(symbol);
                            }
                        }
                    }
                }
            }
            "const_declaration" => {
                let symbol = self.node_to_symbol(source, node, SymbolKind::Constant, file_path);
                if !symbol.name.is_empty() {
                    symbols.push(symbol);
                }
            }
            _ => {}
        }

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_go(source, child, file_path, symbols);
        }
    }

    /// Extract C symbols
    fn extract_c(&self, source: &[u8], node: tree_sitter::Node, file_path: &str, symbols: &mut Vec<Symbol>) {
        let node_kind = node.kind();

        match node_kind {
            "function_definition" => {
                let symbol = self.node_to_symbol(source, node, SymbolKind::Function, file_path);
                if !symbol.name.is_empty() {
                    symbols.push(symbol);
                }
            }
            "struct_specifier" | "union_specifier" => {
                let symbol = self.node_to_symbol(source, node, SymbolKind::Struct, file_path);
                if !symbol.name.is_empty() {
                    symbols.push(symbol);
                }
            }
            "enum_specifier" => {
                let symbol = self.node_to_symbol(source, node, SymbolKind::Enum, file_path);
                if !symbol.name.is_empty() {
                    symbols.push(symbol);
                }
            }
            "type_definition" => {
                let symbol = self.node_to_symbol(source, node, SymbolKind::TypeAlias, file_path);
                if !symbol.name.is_empty() {
                    symbols.push(symbol);
                }
            }
            "preproc_def" => {
                let symbol = self.node_to_symbol(source, node, SymbolKind::Constant, file_path);
                if !symbol.name.is_empty() {
                    symbols.push(symbol);
                }
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_c(source, child, file_path, symbols);
        }
    }

    /// Extract C++ symbols
    fn extract_cpp(&self, source: &[u8], node: tree_sitter::Node, file_path: &str, symbols: &mut Vec<Symbol>) {
        let node_kind = node.kind();

        match node_kind {
            "function_definition" => {
                let symbol = self.node_to_symbol(source, node, SymbolKind::Function, file_path);
                if !symbol.name.is_empty() {
                    symbols.push(symbol);
                }
            }
            "class_specifier" => {
                let symbol = self.node_to_symbol(source, node, SymbolKind::Class, file_path);
                if !symbol.name.is_empty() {
                    symbols.push(symbol);
                }
            }
            "struct_specifier" => {
                let symbol = self.node_to_symbol(source, node, SymbolKind::Struct, file_path);
                if !symbol.name.is_empty() {
                    symbols.push(symbol);
                }
            }
            "enum_specifier" => {
                let symbol = self.node_to_symbol(source, node, SymbolKind::Enum, file_path);
                if !symbol.name.is_empty() {
                    symbols.push(symbol);
                }
            }
            "namespace_definition" => {
                let symbol = self.node_to_symbol(source, node, SymbolKind::Module, file_path);
                if !symbol.name.is_empty() {
                    symbols.push(symbol);
                }
            }
            "template_declaration" => {
                let symbol = self.node_to_symbol(source, node, SymbolKind::Function, file_path);
                if !symbol.name.is_empty() {
                    symbols.push(symbol);
                }
            }
            "type_definition" => {
                let symbol = self.node_to_symbol(source, node, SymbolKind::TypeAlias, file_path);
                if !symbol.name.is_empty() {
                    symbols.push(symbol);
                }
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_cpp(source, child, file_path, symbols);
        }
    }

    /// Extract C# symbols
    fn extract_csharp(&self, source: &[u8], node: tree_sitter::Node, file_path: &str, symbols: &mut Vec<Symbol>) {
        let node_kind = node.kind();

        match node_kind {
            "class_declaration" => {
                let symbol = self.node_to_symbol(source, node, SymbolKind::Class, file_path);
                if !symbol.name.is_empty() {
                    symbols.push(symbol);
                }
            }
            "interface_declaration" => {
                let symbol = self.node_to_symbol(source, node, SymbolKind::Interface, file_path);
                if !symbol.name.is_empty() {
                    symbols.push(symbol);
                }
            }
            "method_declaration" => {
                let symbol = self.node_to_symbol(source, node, SymbolKind::Method, file_path);
                if !symbol.name.is_empty() {
                    symbols.push(symbol);
                }
            }
            "struct_declaration" => {
                let symbol = self.node_to_symbol(source, node, SymbolKind::Struct, file_path);
                if !symbol.name.is_empty() {
                    symbols.push(symbol);
                }
            }
            "enum_declaration" => {
                let symbol = self.node_to_symbol(source, node, SymbolKind::Enum, file_path);
                if !symbol.name.is_empty() {
                    symbols.push(symbol);
                }
            }
            "namespace_declaration" => {
                let symbol = self.node_to_symbol(source, node, SymbolKind::Module, file_path);
                if !symbol.name.is_empty() {
                    symbols.push(symbol);
                }
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_csharp(source, child, file_path, symbols);
        }
    }

    /// Extract Swift symbols
    fn extract_swift(&self, source: &[u8], node: tree_sitter::Node, file_path: &str, symbols: &mut Vec<Symbol>) {
        // Swift grammar (0.7.x): class/struct use class_declaration with declaration_kind field
        // but the node kind check still applies via generic extraction.
        // class/struct/actor all share class_declaration; we handle via generic extraction
        let symbol_kinds = [
            ("function_declaration", SymbolKind::Function),
            ("enum_declaration", SymbolKind::Enum),
            ("protocol_declaration", SymbolKind::Interface),
        ];
        self.extract_symbols_by_kind(source, node, &symbol_kinds, file_path, symbols);
    }

    /// Extract PHP symbols
    fn extract_php(&self, source: &[u8], node: tree_sitter::Node, file_path: &str, symbols: &mut Vec<Symbol>) {
        let symbol_kinds = [
            ("function_definition", SymbolKind::Function),
            ("class_declaration", SymbolKind::Class),
            ("method_declaration", SymbolKind::Method),
            ("interface_declaration", SymbolKind::Interface),
            ("trait_declaration", SymbolKind::Trait),
            ("enum_declaration", SymbolKind::Enum),
        ];

        self.extract_symbols_by_kind(source, node, &symbol_kinds, file_path, symbols);
    }

    /// Extract Ruby symbols
    fn extract_ruby(&self, source: &[u8], node: tree_sitter::Node, file_path: &str, symbols: &mut Vec<Symbol>) {
        let symbol_kinds = [
            ("method", SymbolKind::Method),
            ("class", SymbolKind::Class),
            ("module", SymbolKind::Module),
        ];

        self.extract_symbols_by_kind(source, node, &symbol_kinds, file_path, symbols);
    }

    /// Extract Bash symbols
    fn extract_bash(&self, source: &[u8], node: tree_sitter::Node, file_path: &str, symbols: &mut Vec<Symbol>) {
        let symbol_kinds = [
            ("function_definition", SymbolKind::Function),
        ];

        self.extract_symbols_by_kind(source, node, &symbol_kinds, file_path, symbols);
    }
    fn extract_method_name(&self, source: &[u8], node: tree_sitter::Node) -> String {
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

    /// Extract the type name from a type_declaration
    fn extract_type_name(&self, source: &[u8], node: tree_sitter::Node) -> String {
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
        self.extract_name(source, node)
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
        let doc = self.extract_preceding_doc(source, node);

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
        symbol.doc = doc;

        // Check for visibility
        symbol.is_public = self.is_public_node(source, node);

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
            if child_kind == "identifier"
                || child_kind == "simple_identifier"
                || child_kind == "type_identifier"
            {
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
        // Validate byte range before accessing
        let start = node.start_byte();
        let end = node.end_byte();

        if start < source.len() && end <= source.len() && start <= end {
            if let Ok(text) = node.utf8_text(source) {
                return text.to_string();
            }
        }
        String::new()
    }

    /// Extract documentation comment preceding a node
    fn extract_preceding_doc(&self, source: &[u8], node: tree_sitter::Node) -> Option<String> {
        let node_start = node.start_byte();

        // Try AST sibling first (works for grammars that include comments in tree)
        if let Some(doc) = self.extract_doc_from_siblings(source, node) {
            return Some(doc);
        }

        // Fallback: scan source backwards from node start to find preceding comments
        self.extract_doc_from_source(source, node_start)
    }

    /// Try to extract doc from AST sibling comment nodes
    fn extract_doc_from_siblings(&self, source: &[u8], node: tree_sitter::Node) -> Option<String> {
        let mut prev = node.prev_sibling();
        let mut doc_lines = Vec::new();

        while let Some(sibling) = prev {
            if sibling.kind() == "comment" {
                if let Ok(text) = sibling.utf8_text(source) {
                    doc_lines.push(text.to_string());
                }
                prev = sibling.prev_sibling();
            } else {
                break;
            }
        }

        if doc_lines.is_empty() {
            return None;
        }

        doc_lines.reverse();
        Some(self.clean_doc_text(&doc_lines))
    }

    /// Scan source backwards from node start to find preceding comment blocks
    fn extract_doc_from_source(&self, source: &[u8], node_start: usize) -> Option<String> {
        let source_text = match std::str::from_utf8(source) {
            Ok(s) => s,
            Err(_) => return None,
        };

        let before_node = &source_text[..node_start.min(source_text.len())];
        let mut lines: std::iter::Peekable<std::iter::Rev<std::str::Lines<'_>>> =
            before_node.lines().rev().peekable();
        let mut doc_lines = Vec::new();

        for line in lines.by_ref() {
            let trimmed = line.trim();
            // C-style block comment on one line
            if trimmed.starts_with("/*") && trimmed.contains("*/") {
                let clean = trimmed
                    .trim_start_matches("/*")
                    .trim_end_matches("*/")
                    .trim();
                if !clean.is_empty() {
                    doc_lines.push(clean.to_string());
                }
                break;
            }
            // Block comment continuation or single line
            if (trimmed.starts_with("/*") && !trimmed.ends_with("*/"))
                || trimmed.starts_with("*/")
            {
                // Multi-line block comment started above
                if trimmed.starts_with("*/") {
                    // We're in the middle of a block comment
                    let clean = trimmed.trim_start_matches("*/").trim().to_string();
                    if !clean.is_empty() {
                        doc_lines.push(clean);
                    }
                    continue;
                }
                if trimmed.starts_with("/*") {
                    let clean = trimmed.trim_start_matches("/*").trim().to_string();
                    if !clean.is_empty() {
                        doc_lines.push(clean);
                    }
                    break;
                }
            }
            // Line comments (//, #, --, ...)
            else if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("--") {
                let clean = if trimmed.starts_with("//") {
                    trimmed.trim_start_matches("//").trim()
                } else if trimmed.starts_with('#') {
                    trimmed.trim_start_matches('#').trim()
                } else {
                    trimmed.trim_start_matches("--").trim()
                };
                // Stop at inner doc (//! or #!) or non-doc comment
                if clean.is_empty() {
                    break;
                }
                // Inner doc comments are module-level, not for this item
                if clean.starts_with('!') || clean.starts_with("TODO") || clean.starts_with("FIXME") {
                    break;
                }
                doc_lines.push(clean.to_string());
            } else if trimmed.is_empty() {
                // Empty line — keep going
                continue;
            } else {
                // Non-comment line found — stop
                break;
            }
        }

        if doc_lines.is_empty() {
            return None;
        }

        doc_lines.reverse();
        Some(self.clean_doc_text(&doc_lines))
    }

    /// Clean up doc text by normalizing comment markers
    fn clean_doc_text(&self, lines: &[String]) -> String {
        let cleaned: Vec<String> = lines.iter().map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("//!") {
                trimmed.trim_start_matches("//!").trim().to_string()
            } else if trimmed.starts_with("/*!") {
                trimmed.trim_start_matches("/*!").trim_end_matches("*/").trim().to_string()
            } else {
                trimmed.to_string()
            }
        }).collect();

        let doc = cleaned.join(" ");
        if doc.is_empty() { String::new() } else { doc }
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
    fn is_public_node(&self, source: &[u8], node: tree_sitter::Node) -> bool {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "visibility_modifier" {
                // Validate byte range
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
