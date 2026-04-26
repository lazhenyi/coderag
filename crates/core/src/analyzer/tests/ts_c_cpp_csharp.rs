//! TypeScript, C, C++, C# analyzer tests

use crate::analyzer::Analyzer;
use crate::analyzer::symbol::SymbolKind;
use crate::parser::{Language, Parser};

#[test]
fn test_extract_typescript_symbols() {
    let parser = Parser::new().unwrap();
    let analyzer = Analyzer::new();
    let ts_code = r#"
interface Config { name: string; }
class Service { method(): void {} }
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
class MyClass { public: void method(); };
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
class UserService : IRepository<User> { public void Save(User u) {} }
"#;
    let result = parser.parse_with_language(cs_code.as_bytes(), Language::CSharp).unwrap();
    let symbols = analyzer.extract_symbols(cs_code.as_bytes(), &result.tree, &Language::CSharp, "test.cs");
    let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
    assert!(kinds.contains(&SymbolKind::Interface));
    assert!(kinds.contains(&SymbolKind::Class));
    assert!(kinds.contains(&SymbolKind::Method));
}
