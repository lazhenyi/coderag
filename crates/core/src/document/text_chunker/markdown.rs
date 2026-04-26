//! Markdown stripping utilities

/// Strip markdown formatting, leaving plain text
pub fn strip_markdown(text: &str) -> String {
    let mut result = String::with_capacity(text.len());

    for line in text.lines() {
        let stripped = if line.starts_with('#') {
            line.trim_start_matches('#').trim_start().to_string()
        } else {
            line.to_string()
        };

        let stripped = stripped
            .replace("**", "")
            .replace("__", "")
            .replace('*', "")
            .replace('_', "");

        let stripped = strip_links(&stripped);
        let stripped = strip_images(&stripped);

        let stripped = if stripped.starts_with("```") || stripped.starts_with("~~~") {
            String::new()
        } else {
            stripped
        };

        let stripped = strip_list_marker(&stripped);

        if !stripped.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&stripped);
        }
    }

    result
}

fn strip_links(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '[' {
            let mut link_text = String::new();
            let mut found_close = false;
            while let Some(c) = chars.next() {
                if c == ']' {
                    found_close = true;
                    break;
                }
                link_text.push(c);
            }
            if found_close {
                if chars.peek() == Some(&'(') {
                    chars.next();
                    let mut depth = 1;
                    while let Some(c) = chars.next() {
                        if c == '(' {
                            depth += 1;
                        } else if c == ')' {
                            depth -= 1;
                        }
                        if depth == 0 {
                            break;
                        }
                    }
                }
            }
            result.push_str(&link_text);
        } else {
            result.push(ch);
        }
    }

    result
}

fn strip_images(text: &str) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '!' && chars.peek() == Some(&'[') {
            chars.next();
            let mut depth = 1;
            while let Some(c) = chars.next() {
                if c == '[' {
                    depth += 1;
                } else if c == ']' {
                    depth -= 1;
                }
                if depth == 0 {
                    break;
                }
            }
            if chars.peek() == Some(&'(') {
                chars.next();
                let mut depth = 1;
                while let Some(c) = chars.next() {
                    if c == '(' {
                        depth += 1;
                    } else if c == ')' {
                        depth -= 1;
                    }
                    if depth == 0 {
                        break;
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

fn strip_list_marker(line: &str) -> String {
    let trimmed = line.trim_start();
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        trimmed[2..].to_string()
    } else {
        line.to_string()
    }
}
