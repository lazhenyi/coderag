//! Analyzer Module
//!
//! Extracts symbols (functions, classes, methods, etc.) from parsed AST.

pub mod symbol;
mod extractor;

pub use symbol::{Symbol, SymbolKind, SymbolScope};
pub use extractor::SymbolExtractor;

/// Analyzer for extracting symbols from source code
pub struct Analyzer {
    extractor: SymbolExtractor,
}

impl Analyzer {
    /// Create a new analyzer
    pub fn new() -> Self {
        Self {
            extractor: SymbolExtractor::new(),
        }
    }

    /// Extract all symbols from parsed tree
    pub fn extract_symbols(
        &self,
        source: &[u8],
        tree: &tree_sitter::Tree,
        language: &crate::parser::Language,
        file_path: &str,
    ) -> Vec<Symbol> {
        self.extractor.extract(source, tree, language, file_path)
    }

    /// Extract symbols with module path
    pub fn extract_symbols_with_module(
        &self,
        source: &[u8],
        tree: &tree_sitter::Tree,
        language: &crate::parser::Language,
        file_path: &str,
        module_path: &str,
    ) -> Vec<Symbol> {
        let mut symbols = self.extract_symbols(source, tree, language, file_path);
        for symbol in &mut symbols {
            symbol.module_path = module_path.to_string();
        }
        symbols
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Language, Parser};

    #[test]
    fn test_extract_rust_symbols() {
        let parser = Parser::new().unwrap();
        let analyzer = Analyzer::new();

        let rust_code = r#"
mod foo {
    pub fn public_function(x: i32) -> String {
        "hello"
    }

    fn private_function() {}

    pub struct MyStruct {
        pub value: i32,
    }

    impl MyStruct {
        pub fn new() -> Self {
            Self { value: 42 }
        }

        fn private_method(&self) {}
    }

    pub trait MyTrait {
        fn trait_method(&self);
    }

    pub enum MyEnum {
        VariantA,
        VariantB(i32),
    }
}

fn standalone_function() {}

const MY_CONST: i32 = 42;
static MY_STATIC: &str = "hello";
"#;

        let result = parser.parse_with_language(rust_code.as_bytes(), Language::Rust)
            .unwrap();
        let symbols = analyzer.extract_symbols(
            rust_code.as_bytes(),
            &result.tree,
            &Language::Rust,
            "src/lib.rs",
        );

        println!("Found {} symbols:", symbols.len());
        for symbol in &symbols {
            println!("  {:?}: {} (line {})", symbol.kind, symbol.name, symbol.line_start);
        }

        // Verify we found key symbols
        let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
        assert!(kinds.contains(&SymbolKind::Module));
        assert!(kinds.contains(&SymbolKind::Function));
        assert!(kinds.contains(&SymbolKind::Struct));
        assert!(kinds.contains(&SymbolKind::Method));
        assert!(kinds.contains(&SymbolKind::Trait));
        assert!(kinds.contains(&SymbolKind::Enum));
    }

    #[test]
    fn test_extract_python_symbols() {
        let parser = Parser::new().unwrap();
        let analyzer = Analyzer::new();

        let python_code = r#"
import os
from typing import List

class MyClass:
    def __init__(self):
        self.value = 42

    def method(self):
        return self.value

    @staticmethod
    def static_method():
        return 100

    @classmethod
    def class_method(cls):
        return cls

def standalone_function(x, y):
    return x + y

async def async_function():
    pass

class AnotherClass(BaseClass):
    pass
"#;

        let result = parser.parse_with_language(python_code.as_bytes(), Language::Python)
            .unwrap();
        let symbols = analyzer.extract_symbols(
            python_code.as_bytes(),
            &result.tree,
            &Language::Python,
            "test.py",
        );

        println!("Found {} Python symbols:", symbols.len());
        for symbol in &symbols {
            println!("  {:?}: {} (line {})", symbol.kind, symbol.name, symbol.line_start);
        }

        let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
        assert!(kinds.contains(&SymbolKind::Class));
        assert!(kinds.contains(&SymbolKind::Method));
        assert!(kinds.contains(&SymbolKind::Function));
    }

    #[test]
    fn test_extract_java_symbols() {
        let parser = Parser::new().unwrap();
        let analyzer = Analyzer::new();

        let java_code = r#"
package com.example;

public class MyClass extends BaseClass implements MyInterface {
    private int value;
    protected String name;

    public MyClass() {
        this.value = 42;
    }

    public void method() {
        System.out.println("Hello");
    }

    private void privateMethod() {}

    public static void staticMethod() {}

    interface InnerInterface {
        void innerMethod();
    }

    static class StaticInnerClass {}
}

interface MyInterface {
    void interfaceMethod();
}

enum MyEnum {
    A, B, C;

    MyEnum() {}

    void enumMethod() {}
}
"#;

        let result = parser.parse_with_language(java_code.as_bytes(), Language::Java)
            .unwrap();
        let symbols = analyzer.extract_symbols(
            java_code.as_bytes(),
            &result.tree,
            &Language::Java,
            "MyClass.java",
        );

        println!("Found {} Java symbols:", symbols.len());
        for symbol in &symbols {
            println!("  {:?}: {} (line {})", symbol.kind, symbol.name, symbol.line_start);
        }

        let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
        assert!(kinds.contains(&SymbolKind::Class));
        assert!(kinds.contains(&SymbolKind::Interface));
        assert!(kinds.contains(&SymbolKind::Method));
        assert!(kinds.contains(&SymbolKind::Enum));
    }

    #[test]
    fn test_extract_go_symbols() {
        let parser = Parser::new().unwrap();
        let analyzer = Analyzer::new();

        let go_code = r#"
package main

import "fmt"

type MyStruct struct {
    Value int
    Name  string
}

type MyInterface interface {
    Method() string
}

func NewMyStruct() *MyStruct {
    return &MyStruct{Value: 42}
}

func (m *MyStruct) Method() string {
    return m.Name
}

func StandaloneFunction() {
    fmt.Println("Hello")
}

const MyConst = 42

var MyVar = "hello"
"#;

        let result = parser.parse_with_language(go_code.as_bytes(), Language::Go)
            .unwrap();
        let symbols = analyzer.extract_symbols(
            go_code.as_bytes(),
            &result.tree,
            &Language::Go,
            "main.go",
        );

        println!("Found {} Go symbols:", symbols.len());
        for symbol in &symbols {
            println!("  {:?}: {} (line {})", symbol.kind, symbol.name, symbol.line_start);
        }

        let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
        assert!(kinds.contains(&SymbolKind::Struct));
        assert!(kinds.contains(&SymbolKind::Interface));
        assert!(kinds.contains(&SymbolKind::Function));
    }
}
