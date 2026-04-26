//! JavaScript, Rust, Python analyzer tests

use crate::analyzer::Analyzer;
use crate::analyzer::symbol::SymbolKind;
use crate::parser::{Language, Parser};

#[test]
fn test_extract_javascript_symbols() {
    let parser = Parser::new().unwrap();
    let analyzer = Analyzer::new();

    let js_code = r#"
function standaloneFunction() { return 42; }
class MyClass {
    constructor() { this.value = 42; }
    method() { return this.value; }
    static staticMethod() { return 100; }
}
async function asyncFunction() { return await Promise.resolve(42); }
"#;

    let result = parser
        .parse_with_language(js_code.as_bytes(), Language::JavaScript)
        .unwrap();
    let symbols = analyzer.extract_symbols(
        js_code.as_bytes(),
        &result.tree,
        &Language::JavaScript,
        "test.js",
    );
    let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
    assert!(kinds.contains(&SymbolKind::Function));
    assert!(kinds.contains(&SymbolKind::Class));
    assert!(kinds.contains(&SymbolKind::Method));
}

#[test]
fn test_extract_rust_symbols() {
    let parser = Parser::new().unwrap();
    let analyzer = Analyzer::new();

    let rust_code = r#"
mod foo {
    pub fn public_function(x: i32) -> String { "hello" }
    fn private_function() {}
    pub struct MyStruct { pub value: i32 }
    impl MyStruct { pub fn new() -> Self { Self { value: 42 } } fn private_method(&self) {} }
    pub trait MyTrait { fn trait_method(&self); }
    pub enum MyEnum { VariantA, VariantB(i32) }
}
fn standalone_function() {}
const MY_CONST: i32 = 42;
static MY_STATIC: &str = "hello";
"#;

    let result = parser
        .parse_with_language(rust_code.as_bytes(), Language::Rust)
        .unwrap();
    let symbols = analyzer.extract_symbols(
        rust_code.as_bytes(),
        &result.tree,
        &Language::Rust,
        "src/lib.rs",
    );
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
    def __init__(self): self.value = 42
    def method(self): return self.value
    @staticmethod
    def static_method(): return 100
def standalone_function(x, y): return x + y
async def async_function(): pass
class AnotherClass(BaseClass): pass
"#;

    let result = parser
        .parse_with_language(python_code.as_bytes(), Language::Python)
        .unwrap();
    let symbols = analyzer.extract_symbols(
        python_code.as_bytes(),
        &result.tree,
        &Language::Python,
        "test.py",
    );
    let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
    assert!(kinds.contains(&SymbolKind::Class));
    assert!(kinds.contains(&SymbolKind::Method));
    assert!(kinds.contains(&SymbolKind::Function));
}

#[test]
fn test_extract_doc_comments() {
    let parser = Parser::new().unwrap();
    let analyzer = Analyzer::new();

    let rust_code = r#"
/// Adds two numbers together.
fn add(a: i32, b: i32) -> i32 { a + b }

/// Represents a point in 2D space.
struct Point { x: f64, y: f64 }
"#;

    let result = parser
        .parse_with_language(rust_code.as_bytes(), Language::Rust)
        .unwrap();
    let symbols = analyzer.extract_symbols(
        rust_code.as_bytes(),
        &result.tree,
        &Language::Rust,
        "test.rs",
    );

    let add_fn = symbols.iter().find(|s| s.name == "add");
    let point_struct = symbols.iter().find(|s| s.name == "Point");

    assert!(add_fn.is_some());
    assert!(point_struct.is_some());

    let add_doc = &add_fn.unwrap().doc;
    assert!(add_doc.is_some());
    assert!(add_doc.as_ref().unwrap().contains("Adds two numbers"));

    let point_doc = &point_struct.unwrap().doc;
    assert!(point_doc.is_some());
    assert!(point_doc.as_ref().unwrap().contains("point in 2D space"));
}
