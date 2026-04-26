//! Scala, Dart, Lua, R, Perl analyzer tests

use crate::analyzer::Analyzer;
use crate::analyzer::symbol::SymbolKind;
use crate::parser::{Language, Parser};

#[test]
fn test_extract_scala_symbols() {
    let parser = Parser::new().unwrap();
    let analyzer = Analyzer::new();
    let scala_code = r#"
class MyClass { def method(): Int = 42 }
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
