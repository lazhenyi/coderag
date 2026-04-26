//! Documentation extraction helpers

/// Extract documentation comment preceding a node
pub fn extract_preceding_doc(source: &[u8], node: tree_sitter::Node) -> Option<String> {
    if let Some(doc) = extract_doc_from_siblings(source, node) {
        return Some(doc);
    }
    extract_doc_from_source(source, node.start_byte())
}

fn extract_doc_from_siblings(source: &[u8], node: tree_sitter::Node) -> Option<String> {
    let mut prev = node.prev_sibling();
    let mut doc_lines = Vec::new();
    while let Some(sibling) = prev {
        if sibling.kind() == "comment" {
            if let Ok(text) = sibling.utf8_text(source) {
                doc_lines.push(text.to_string());
            }
            prev = sibling.prev_sibling();
        } else {
            break;
        }
    }
    if doc_lines.is_empty() {
        return None;
    }
    doc_lines.reverse();
    Some(clean_doc_text(&doc_lines))
}

fn extract_doc_from_source(source: &[u8], node_start: usize) -> Option<String> {
    let source_text = std::str::from_utf8(source).ok()?;
    let before_node = &source_text[..node_start.min(source_text.len())];
    let mut lines = before_node.lines().rev().peekable();
    let mut doc_lines = Vec::new();

    for line in lines.by_ref() {
        let trimmed = line.trim();
        if trimmed.starts_with("/*") && trimmed.contains("*/") {
            let clean = trimmed.trim_start_matches("/*").trim_end_matches("*/").trim();
            if !clean.is_empty() {
                doc_lines.push(clean.to_string());
            }
            break;
        }
        if (trimmed.starts_with("/*") && !trimmed.ends_with("*/")) || trimmed.starts_with("*/") {
            if trimmed.starts_with("*/") {
                let clean = trimmed.trim_start_matches("*/").trim().to_string();
                if !clean.is_empty() { doc_lines.push(clean); }
                continue;
            }
            if trimmed.starts_with("/*") {
                let clean = trimmed.trim_start_matches("/*").trim().to_string();
                if !clean.is_empty() { doc_lines.push(clean); }
                break;
            }
        } else if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("--") {
            let clean = if trimmed.starts_with("//") {
                trimmed.trim_start_matches("//").trim()
            } else if trimmed.starts_with('#') {
                trimmed.trim_start_matches('#').trim()
            } else {
                trimmed.trim_start_matches("--").trim()
            };
            if clean.is_empty() || clean.starts_with('!') || clean.starts_with("TODO") || clean.starts_with("FIXME") {
                break;
            }
            doc_lines.push(clean.to_string());
        } else if trimmed.is_empty() {
            continue;
        } else {
            break;
        }
    }
    if doc_lines.is_empty() {
        return None;
    }
    doc_lines.reverse();
    Some(clean_doc_text(&doc_lines))
}

/// Clean up doc text by normalizing comment markers
pub fn clean_doc_text(lines: &[String]) -> String {
    let cleaned: Vec<String> = lines.iter().map(|line| {
        let trimmed = line.trim();
        if trimmed.starts_with("//!") {
            trimmed.trim_start_matches("//!").trim().to_string()
        } else if trimmed.starts_with("/*!") {
            trimmed.trim_start_matches("/*!").trim_end_matches("*/").trim().to_string()
        } else {
            trimmed.to_string()
        }
    }).collect();
    let doc = cleaned.join(" ");
    if doc.is_empty() { String::new() } else { doc }
}
