//! Parser tests

#[cfg(test)]
mod tests {
    use crate::parser::{Language, Parser};
    use std::path::Path;

    #[test]
    fn test_rust_parsing() {
        let parser = Parser::new().unwrap();
        let rust_code = r#"
fn main() { println!("Hello, world!"); }
struct MyStruct { value: i32 }
impl MyStruct { fn new() -> Self { Self { value: 42 } } }
"#;
        let result = parser
            .parse_auto(rust_code.as_bytes(), Some(Path::new("test.rs")))
            .unwrap();
        assert!(matches!(result.language, Language::Rust));
    }

    #[test]
    fn test_python_parsing() {
        let parser = Parser::new().unwrap();
        let python_code = r#"
def hello(): print("Hello, world!")
class MyClass:
    def __init__(self): self.value = 42
    def method(self): return self.value
"#;
        let result = parser
            .parse_auto(python_code.as_bytes(), Some(Path::new("test.py")))
            .unwrap();
        assert!(matches!(result.language, Language::Python));
    }

    #[test]
    fn test_javascript_parsing() {
        let parser = Parser::new().unwrap();
        let js_code = r#"
function hello() { console.log("Hello!"); }
class MyClass { constructor() { this.value = 42; } method() { return this.value; } }
"#;
        let result = parser
            .parse_auto(js_code.as_bytes(), Some(Path::new("test.js")))
            .unwrap();
        assert!(matches!(result.language, Language::JavaScript));
    }

    #[test]
    fn test_language_detection() {
        let parser = Parser::new().unwrap();
        assert_eq!(
            parser.detect_language_by_extension("rs"),
            Some(Language::Rust)
        );
        assert_eq!(
            parser.detect_language_by_extension("py"),
            Some(Language::Python)
        );
        assert_eq!(
            parser.detect_language_by_extension("js"),
            Some(Language::JavaScript)
        );
        assert_eq!(
            parser.detect_language_by_extension("java"),
            Some(Language::Java)
        );
        assert_eq!(
            parser.detect_language_by_extension("go"),
            Some(Language::Go)
        );

        let python_shebang = b"#!/usr/bin/env python\nprint('hello')";
        assert_eq!(
            parser.detect_language_from_content(python_shebang),
            Some(Language::Python)
        );

        let rust_shebang = b"#!/usr/bin/rust-run\nfn main() {}";
        assert_eq!(
            parser.detect_language_from_content(rust_shebang),
            Some(Language::Rust)
        );
    }
}
