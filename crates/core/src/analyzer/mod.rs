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
    fn test_extract_javascript_symbols() {
        let parser = Parser::new().unwrap();
        let analyzer = Analyzer::new();

        let js_code = r#"
function standaloneFunction() {
    return 42;
}

class MyClass {
    constructor() {
        this.value = 42;
    }

    method() {
        return this.value;
    }

    static staticMethod() {
        return 100;
    }
}

function* generatorFunction() {
    yield 1;
}

async function asyncFunction() {
    return await Promise.resolve(42);
}
"#;

        let result = parser.parse_with_language(js_code.as_bytes(), Language::JavaScript)
            .unwrap();
        let symbols = analyzer.extract_symbols(
            js_code.as_bytes(),
            &result.tree,
            &Language::JavaScript,
            "test.js",
        );

        println!("Found {} JavaScript symbols:", symbols.len());
        for symbol in &symbols {
            println!("  {:?}: {} (line {})", symbol.kind, symbol.name, symbol.line_start);
        }

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

    #[test]
    fn test_extract_kotlin_symbols() {
        let parser = Parser::new().unwrap();
        let analyzer = Analyzer::new();

        let kotlin_code = r#"
package com.example

class MyClass(val name: String) {
    fun method(): String {
        return name
    }
}

interface MyInterface {
    fun interfaceMethod(): Int
}

object MySingleton {
    val instance = this
}

fun topLevelFunction(): Boolean {
    return true
}

enum class Color {
    RED, GREEN, BLUE
}
"#;

        let result = parser.parse_with_language(kotlin_code.as_bytes(), Language::Kotlin)
            .unwrap();
        let symbols = analyzer.extract_symbols(
            kotlin_code.as_bytes(),
            &result.tree,
            &Language::Kotlin,
            "test.kt",
        );

        println!("Found {} Kotlin symbols:", symbols.len());
        for symbol in &symbols {
            println!("  {:?}: {} (line {})", symbol.kind, symbol.name, symbol.line_start);
        }

        let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
        assert!(kinds.contains(&SymbolKind::Class));
        assert!(kinds.contains(&SymbolKind::Function));
    }

    #[test]
    fn test_extract_sql_symbols() {
        let parser = Parser::new().unwrap();
        let analyzer = Analyzer::new();

        let sql_code = r#"
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT UNIQUE
);

CREATE FUNCTION get_user_name(user_id INTEGER)
RETURNS TEXT
LANGUAGE SQL
BEGIN
    SELECT name FROM users WHERE id = user_id;
    RETURN name;
END;

CREATE PROCEDURE update_user_email(user_id INTEGER, new_email TEXT)
LANGUAGE SQL
BEGIN
    UPDATE users SET email = new_email WHERE id = user_id;
END;

CREATE VIEW active_users AS
SELECT id, name FROM users WHERE active = 1;
"#;

        let result = parser.parse_with_language(sql_code.as_bytes(), Language::Sql)
            .unwrap();
        let symbols = analyzer.extract_symbols(
            sql_code.as_bytes(),
            &result.tree,
            &Language::Sql,
            "schema.sql",
        );

        println!("Found {} SQL symbols:", symbols.len());
        for symbol in &symbols {
            println!("  {:?}: {} (line {})", symbol.kind, symbol.name, symbol.line_start);
        }

        let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
        assert!(kinds.contains(&SymbolKind::Class));
        assert!(kinds.contains(&SymbolKind::Function));
    }

    #[test]
    fn test_extract_doc_comments() {
        let parser = Parser::new().unwrap();
        let analyzer = Analyzer::new();

        let rust_code = r#"
/// Adds two numbers together.
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Represents a point in 2D space.
struct Point {
    x: f64,
    y: f64,
}
"#;

        let result = parser.parse_with_language(rust_code.as_bytes(), Language::Rust)
            .unwrap();
        let symbols = analyzer.extract_symbols(
            rust_code.as_bytes(),
            &result.tree,
            &Language::Rust,
            "test.rs",
        );

        let add_fn = symbols.iter().find(|s| s.name == "add");
        let point_struct = symbols.iter().find(|s| s.name == "Point");

        assert!(add_fn.is_some(), "add function should be found");
        assert!(point_struct.is_some(), "Point struct should be found");

        let add_doc = &add_fn.unwrap().doc;
        assert!(add_doc.is_some(), "add function should have a doc comment");
        assert!(add_doc.as_ref().unwrap().contains("Adds two numbers"), "doc should contain the comment text");

        let point_doc = &point_struct.unwrap().doc;
        assert!(point_doc.is_some(), "Point struct should have a doc comment");
        assert!(point_doc.as_ref().unwrap().contains("point in 2D space"), "doc should contain the comment text");
    }

    #[test]
    fn test_extract_typescript_symbols() {
        let parser = Parser::new().unwrap();
        let analyzer = Analyzer::new();
        let ts_code = r#"
interface Config {
    name: string;
}
class Service {
    method(): void {}
}
function handler() {}
"#;
        let result = parser.parse_with_language(ts_code.as_bytes(), Language::TypeScript).unwrap();
        let symbols = analyzer.extract_symbols(ts_code.as_bytes(), &result.tree, &Language::TypeScript, "test.ts");
        let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
        assert!(kinds.contains(&SymbolKind::Interface));
        assert!(kinds.contains(&SymbolKind::Class));
        assert!(kinds.contains(&SymbolKind::Function));
    }

    #[test]
    fn test_extract_c_symbols() {
        let parser = Parser::new().unwrap();
        let analyzer = Analyzer::new();
        let c_code = r#"
struct Point { double x; double y; };
typedef struct { int id; } User;
void process() {}
"#;
        let result = parser.parse_with_language(c_code.as_bytes(), Language::C).unwrap();
        let symbols = analyzer.extract_symbols(c_code.as_bytes(), &result.tree, &Language::C, "test.c");
        let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
        assert!(kinds.contains(&SymbolKind::Struct));
        assert!(kinds.contains(&SymbolKind::Function));
    }

    #[test]
    fn test_extract_cpp_symbols() {
        let parser = Parser::new().unwrap();
        let analyzer = Analyzer::new();
        let cpp_code = r#"
class MyClass {
public:
    void method();
};
namespace util { void helper() {} }
"#;
        let result = parser.parse_with_language(cpp_code.as_bytes(), Language::Cpp).unwrap();
        let symbols = analyzer.extract_symbols(cpp_code.as_bytes(), &result.tree, &Language::Cpp, "test.cpp");
        let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
        assert!(kinds.contains(&SymbolKind::Class));
        assert!(kinds.contains(&SymbolKind::Function));
    }

    #[test]
    fn test_extract_csharp_symbols() {
        let parser = Parser::new().unwrap();
        let analyzer = Analyzer::new();
        let cs_code = r#"
interface IRepository<T> { void Save(T item); }
class UserService : IRepository<User> {
    public void Save(User u) {}
}
"#;
        let result = parser.parse_with_language(cs_code.as_bytes(), Language::CSharp).unwrap();
        let symbols = analyzer.extract_symbols(cs_code.as_bytes(), &result.tree, &Language::CSharp, "test.cs");
        let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
        assert!(kinds.contains(&SymbolKind::Interface));
        assert!(kinds.contains(&SymbolKind::Class));
        assert!(kinds.contains(&SymbolKind::Method));
    }

    #[test]
    fn test_extract_swift_symbols() {
        let parser = Parser::new().unwrap();
        let analyzer = Analyzer::new();

        let swift_code = r#"
protocol Drawable { func draw() }
func process() {}
"#;
        let result = parser.parse_with_language(swift_code.as_bytes(), Language::Swift).unwrap();
        let symbols = analyzer.extract_symbols(swift_code.as_bytes(), &result.tree, &Language::Swift, "test.swift");
        let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
        assert!(kinds.contains(&SymbolKind::Interface));
        assert!(kinds.contains(&SymbolKind::Function));
    }

    #[test]
    fn test_extract_php_symbols() {
        let parser = Parser::new().unwrap();
        let analyzer = Analyzer::new();
        let php_code = r#"
<?php
function greet($name) { return "Hello $name"; }
class User {
    public function login() {}
}
interface Authenticatable { public function authenticate(); }
"#;
        let result = parser.parse_with_language(php_code.as_bytes(), Language::Php).unwrap();
        let symbols = analyzer.extract_symbols(php_code.as_bytes(), &result.tree, &Language::Php, "test.php");
        let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
        assert!(kinds.contains(&SymbolKind::Function));
        assert!(kinds.contains(&SymbolKind::Class));
        assert!(kinds.contains(&SymbolKind::Interface));
    }

    #[test]
    fn test_extract_ruby_symbols() {
        let parser = Parser::new().unwrap();
        let analyzer = Analyzer::new();
        let ruby_code = r#"
module MyModule
    class MyClass
        def my_method
        end
    end
end
"#;
        let result = parser.parse_with_language(ruby_code.as_bytes(), Language::Ruby).unwrap();
        let symbols = analyzer.extract_symbols(ruby_code.as_bytes(), &result.tree, &Language::Ruby, "test.rb");
        let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
        assert!(kinds.contains(&SymbolKind::Class));
        assert!(kinds.contains(&SymbolKind::Method));
    }

    #[test]
    fn test_extract_bash_symbols() {
        let parser = Parser::new().unwrap();
        let analyzer = Analyzer::new();
        let bash_code = r#"
#!/bin/bash
function greet() {
    echo "Hello"
}
"#;
        let result = parser.parse_with_language(bash_code.as_bytes(), Language::Shell).unwrap();
        let symbols = analyzer.extract_symbols(bash_code.as_bytes(), &result.tree, &Language::Shell, "test.sh");
        let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
        assert!(kinds.contains(&SymbolKind::Function));
    }

    #[test]
    fn test_extract_scala_symbols() {
        let parser = Parser::new().unwrap();
        let analyzer = Analyzer::new();
        let scala_code = r#"
class MyClass {
    def method(): Int = 42
}
trait Greeter { def greet(): Unit }
object Singleton { val x = 1 }
"#;
        let result = parser.parse_with_language(scala_code.as_bytes(), Language::Scala).unwrap();
        let symbols = analyzer.extract_symbols(scala_code.as_bytes(), &result.tree, &Language::Scala, "test.scala");
        let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
        assert!(kinds.contains(&SymbolKind::Class));
        assert!(kinds.contains(&SymbolKind::Function));
    }

    #[test]
    fn test_extract_dart_symbols() {
        let parser = Parser::new().unwrap();
        let analyzer = Analyzer::new();
        let dart_code = r#"
class Counter { int _count = 0; }
enum Color { red, green, blue }
mixin Logging { void log() {} }
"#;
        let result = parser.parse_with_language(dart_code.as_bytes(), Language::Dart).unwrap();
        let symbols = analyzer.extract_symbols(dart_code.as_bytes(), &result.tree, &Language::Dart, "test.dart");
        let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
        assert!(kinds.contains(&SymbolKind::Class));
        assert!(kinds.contains(&SymbolKind::Enum));
    }

    #[test]
    fn test_extract_lua_symbols() {
        let parser = Parser::new().unwrap();
        let analyzer = Analyzer::new();
        let lua_code = r#"
function greet(name)
    print("Hello " .. name)
end
"#;
        let result = parser.parse_with_language(lua_code.as_bytes(), Language::Lua).unwrap();
        let symbols = analyzer.extract_symbols(lua_code.as_bytes(), &result.tree, &Language::Lua, "test.lua");
        let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
        assert!(kinds.contains(&SymbolKind::Function));
    }

    #[test]
    fn test_extract_r_symbols() {
        let parser = Parser::new().unwrap();
        let analyzer = Analyzer::new();
        let r_code = r#"
my_function <- function(x) { x + 1 }
my_class <- setRefClass("MyClass", fields = list(x = "numeric"))
"#;
        let result = parser.parse_with_language(r_code.as_bytes(), Language::R).unwrap();
        let symbols = analyzer.extract_symbols(r_code.as_bytes(), &result.tree, &Language::R, "test.r");
        let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
        assert!(kinds.contains(&SymbolKind::Function));
    }

    #[test]
    fn test_extract_perl_symbols() {
        let parser = Parser::new().unwrap();
        let analyzer = Analyzer::new();
        let perl_code = r#"
package MyModule;
sub new { }
sub process { }
"#;
        let result = parser.parse_with_language(perl_code.as_bytes(), Language::Perl).unwrap();
        let symbols = analyzer.extract_symbols(perl_code.as_bytes(), &result.tree, &Language::Perl, "test.pm");
        let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
        assert!(kinds.contains(&SymbolKind::Module) || kinds.contains(&SymbolKind::Function));
    }
}
