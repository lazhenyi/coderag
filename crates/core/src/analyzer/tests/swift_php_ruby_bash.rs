//! Swift, PHP, Ruby, Bash analyzer tests

use crate::analyzer::Analyzer;
use crate::analyzer::symbol::SymbolKind;
use crate::parser::{Language, Parser};

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
class User { public function login() {} }
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
function greet() { echo "Hello" }
"#;
    let result = parser.parse_with_language(bash_code.as_bytes(), Language::Shell).unwrap();
    let symbols = analyzer.extract_symbols(bash_code.as_bytes(), &result.tree, &Language::Shell, "test.sh");
    let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
    assert!(kinds.contains(&SymbolKind::Function));
}
