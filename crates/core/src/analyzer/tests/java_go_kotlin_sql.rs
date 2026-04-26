//! Java, Go, Kotlin, SQL analyzer tests

use crate::analyzer::Analyzer;
use crate::analyzer::symbol::SymbolKind;
use crate::parser::{Language, Parser};

#[test]
fn test_extract_java_symbols() {
    let parser = Parser::new().unwrap();
    let analyzer = Analyzer::new();

    let java_code = r#"
package com.example;
public class MyClass extends BaseClass implements MyInterface {
    private int value;
    public MyClass() { this.value = 42; }
    public void method() { System.out.println("Hello"); }
    private void privateMethod() {}
    public static void staticMethod() {}
    interface InnerInterface { void innerMethod(); }
    static class StaticInnerClass {}
}
interface MyInterface { void interfaceMethod(); }
enum MyEnum { A, B, C; MyEnum() {} void enumMethod() {} }
"#;

    let result = parser
        .parse_with_language(java_code.as_bytes(), Language::Java)
        .unwrap();
    let symbols = analyzer.extract_symbols(
        java_code.as_bytes(),
        &result.tree,
        &Language::Java,
        "MyClass.java",
    );
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
type MyStruct struct { Value int; Name string }
type MyInterface interface { Method() string }
func NewMyStruct() *MyStruct { return &MyStruct{Value: 42} }
func (m *MyStruct) Method() string { return m.Name }
func StandaloneFunction() { fmt.Println("Hello") }
const MyConst = 42
var MyVar = "hello"
"#;

    let result = parser
        .parse_with_language(go_code.as_bytes(), Language::Go)
        .unwrap();
    let symbols =
        analyzer.extract_symbols(go_code.as_bytes(), &result.tree, &Language::Go, "main.go");
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
class MyClass(val name: String) { fun method(): String { return name } }
interface MyInterface { fun interfaceMethod(): Int }
object MySingleton { val instance = this }
fun topLevelFunction(): Boolean { return true }
enum class Color { RED, GREEN, BLUE }
"#;

    let result = parser
        .parse_with_language(kotlin_code.as_bytes(), Language::Kotlin)
        .unwrap();
    let symbols = analyzer.extract_symbols(
        kotlin_code.as_bytes(),
        &result.tree,
        &Language::Kotlin,
        "test.kt",
    );
    let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
    assert!(kinds.contains(&SymbolKind::Class));
    assert!(kinds.contains(&SymbolKind::Function));
}

#[test]
fn test_extract_sql_symbols() {
    let parser = Parser::new().unwrap();
    let analyzer = Analyzer::new();

    let sql_code = r#"
CREATE TABLE users ( id INTEGER PRIMARY KEY, name TEXT NOT NULL, email TEXT UNIQUE );
CREATE FUNCTION get_user_name(user_id INTEGER) RETURNS TEXT LANGUAGE SQL BEGIN SELECT name FROM users WHERE id = user_id; RETURN name; END;
CREATE PROCEDURE update_user_email(user_id INTEGER, new_email TEXT) LANGUAGE SQL BEGIN UPDATE users SET email = new_email WHERE id = user_id; END;
CREATE VIEW active_users AS SELECT id, name FROM users WHERE active = 1;
"#;

    let result = parser
        .parse_with_language(sql_code.as_bytes(), Language::Sql)
        .unwrap();
    let symbols = analyzer.extract_symbols(
        sql_code.as_bytes(),
        &result.tree,
        &Language::Sql,
        "schema.sql",
    );
    let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
    assert!(kinds.contains(&SymbolKind::Class));
    assert!(kinds.contains(&SymbolKind::Function));
}
