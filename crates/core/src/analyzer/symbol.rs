//! Symbol Definitions
//!
//! Defines the symbol types and structures used for code analysis.

use serde::{Deserialize, Serialize};

/// Kind of symbol (function, class, method, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Method,
    Class,
    Struct,
    Interface,
    Enum,
    Trait,
    Module,
    Namespace,
    TypeAlias,
    Variable,
    Constant,
    Macro,
    Property,
    Parameter,
    Other,
}

impl std::fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolKind::Function => write!(f, "function"),
            SymbolKind::Method => write!(f, "method"),
            SymbolKind::Class => write!(f, "class"),
            SymbolKind::Struct => write!(f, "struct"),
            SymbolKind::Interface => write!(f, "interface"),
            SymbolKind::Enum => write!(f, "enum"),
            SymbolKind::Trait => write!(f, "trait"),
            SymbolKind::Module => write!(f, "module"),
            SymbolKind::Namespace => write!(f, "namespace"),
            SymbolKind::TypeAlias => write!(f, "type_alias"),
            SymbolKind::Variable => write!(f, "variable"),
            SymbolKind::Constant => write!(f, "constant"),
            SymbolKind::Macro => write!(f, "macro"),
            SymbolKind::Property => write!(f, "property"),
            SymbolKind::Parameter => write!(f, "parameter"),
            SymbolKind::Other => write!(f, "other"),
        }
    }
}

/// Represents a code symbol (function, class, method, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    /// Unique identifier for this symbol
    pub id: String,
    /// Name of the symbol
    pub name: String,
    /// Kind of symbol
    pub kind: SymbolKind,
    /// Line where the symbol starts
    pub line_start: usize,
    /// Line where the symbol ends
    pub line_end: usize,
    /// Column where the symbol starts
    pub column_start: usize,
    /// Column where the symbol ends
    pub column_end: usize,
    /// Module path (e.g., "crate::module::submodule")
    pub module_path: String,
    /// File path relative to repository root
    pub file_path: String,
    /// Full signature (e.g., "fn foo(a: i32, b: &str) -> bool")
    pub signature: String,
    /// Documentation comment (if any)
    pub doc: Option<String>,
    /// Full source code of the symbol definition
    pub code: String,
    /// Whether the symbol is public
    pub is_public: bool,
    /// Whether the symbol is static
    pub is_static: bool,
    /// Whether the symbol is async
    pub is_async: bool,
    /// Parent symbol (e.g., class for a method)
    pub parent: Option<String>,
    /// Language this symbol belongs to
    pub language: String,
}

impl Symbol {
    /// Create a new symbol
    pub fn new(
        name: String,
        kind: SymbolKind,
        line_start: usize,
        line_end: usize,
        language: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            kind,
            line_start,
            line_end,
            column_start: 0,
            column_end: 0,
            module_path: String::new(),
            file_path: String::new(),
            signature: String::new(),
            doc: None,
            code: String::new(),
            is_public: false,
            is_static: false,
            is_async: false,
            parent: None,
            language,
        }
    }

    /// Set the source code for this symbol
    pub fn with_code(mut self, code: String) -> Self {
        self.code = code;
        self
    }

    /// Set the file path for this symbol
    pub fn with_file_path(mut self, path: String) -> Self {
        self.file_path = path;
        self
    }

    /// Set the module path for this symbol
    pub fn with_module_path(mut self, path: String) -> Self {
        self.module_path = path;
        self
    }

    /// Set the documentation for this symbol
    pub fn with_doc(mut self, doc: Option<String>) -> Self {
        self.doc = doc;
        self
    }

    /// Set the signature for this symbol
    pub fn with_signature(mut self, sig: String) -> Self {
        self.signature = sig;
        self
    }

    /// Set visibility
    pub fn with_visibility(mut self, is_public: bool) -> Self {
        self.is_public = is_public;
        self
    }
}

/// Scope of a symbol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolScope {
    /// Global scope (top-level)
    Global,
    /// Module scope
    Module(String),
    /// Class/Struct scope
    Type(String),
    /// Function scope
    Function(String),
    /// Block scope
    Block,
}

impl SymbolScope {
    /// Get the full qualified name within this scope
    pub fn qualified_name(&self, name: &str) -> String {
        match self {
            SymbolScope::Global => name.to_string(),
            SymbolScope::Module(m) => format!("{}::{}", m, name),
            SymbolScope::Type(t) => format!("{}::{}", t, name),
            SymbolScope::Function(f) => format!("{}::{}", f, name),
            SymbolScope::Block => name.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_creation() {
        let symbol = Symbol::new(
            "hello".to_string(),
            SymbolKind::Function,
            1,
            10,
            "rust".to_string(),
        );

        assert_eq!(symbol.name, "hello");
        assert_eq!(symbol.kind, SymbolKind::Function);
        assert!(!symbol.is_public);
    }

    #[test]
    fn test_symbol_builder() {
        let symbol = Symbol::new(
            "MyStruct".to_string(),
            SymbolKind::Struct,
            1,
            20,
            "rust".to_string(),
        )
        .with_file_path("src/lib.rs".to_string())
        .with_module_path("crate::module".to_string())
        .with_signature("struct MyStruct { value: i32 }".to_string())
        .with_doc(Some("My documentation".to_string()))
        .with_visibility(true);

        assert_eq!(symbol.file_path, "src/lib.rs");
        assert_eq!(symbol.module_path, "crate::module");
        assert!(symbol.is_public);
        assert!(symbol.doc.is_some());
    }

    #[test]
    fn test_symbol_kind_display() {
        assert_eq!(SymbolKind::Function.to_string(), "function");
        assert_eq!(SymbolKind::Class.to_string(), "class");
    }

    #[test]
    fn test_symbol_scope_qualified_name() {
        let scope = SymbolScope::Module("foo::bar".to_string());
        assert_eq!(scope.qualified_name("baz"), "foo::bar::baz");

        let scope = SymbolScope::Global;
        assert_eq!(scope.qualified_name("hello"), "hello");
    }
}
