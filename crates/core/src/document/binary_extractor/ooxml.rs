//! Structured OOXML (DOCX, ODT, ODS) extraction
//!
//! Parses XML semantically to preserve heading/paragraph/table structure
//! instead of stripping all tags.

/// Extract text from .docx with structure awareness
pub fn extract_docx(content: &[u8]) -> Option<String> {
    #[cfg(feature = "doc-p1")]
    { extract_ooxml(content, "word/document.xml", OoxmlType::Docx) }
    #[cfg(not(feature = "doc-p1"))]
    { let _ = content; None }
}

/// Extract text from .odt with structure awareness
pub fn extract_odt(content: &[u8]) -> Option<String> {
    #[cfg(feature = "doc-p1")]
    { extract_ooxml(content, "content.xml", OoxmlType::Odt) }
    #[cfg(not(feature = "doc-p1"))]
    { let _ = content; None }
}

/// Extract text from .ods (spreadsheet — simpler table format)
pub fn extract_ods(content: &[u8]) -> Option<String> {
    #[cfg(feature = "doc-p1")]
    { extract_ooxml(content, "content.xml", OoxmlType::Ods) }
    #[cfg(not(feature = "doc-p1"))]
    { let _ = content; None }
}

#[cfg(feature = "doc-p1")]
enum OoxmlType { Docx, Odt, Ods }

#[cfg(feature = "doc-p1")]
fn extract_ooxml(zip_data: &[u8], xml_path: &str, kind: OoxmlType) -> Option<String> {
    use std::io::{Cursor, Read};
    let mut archive = match zip::ZipArchive::new(Cursor::new(zip_data)) {
        Ok(a) => a,
        Err(e) => { tracing::warn!("Failed to open OOXML ZIP: {}", e); return None; }
    };
    let mut xml_content = String::new();
    match archive.by_name(xml_path) {
        Ok(mut f) => {
            if f.read_to_string(&mut xml_content).is_err() {
                tracing::warn!("Failed to read {} as UTF-8", xml_path);
                return None;
            }
        }
        Err(e) => {
            tracing::warn!("File {} not found in archive: {}", xml_path, e);
            return None;
        }
    }
    match kind {
        OoxmlType::Docx => parse_docx_xml(&xml_content),
        OoxmlType::Odt => parse_odt_xml(&xml_content),
        OoxmlType::Ods => parse_ods_xml(&xml_content),
    }
}

/// Strip XML tags from content to get raw text.
#[cfg(feature = "doc-p1")]
fn xml_text_content(xml: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for ch in xml.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

#[cfg(feature = "doc-p1")]
fn parse_docx_xml(xml: &str) -> Option<String> {
    let mut result = String::new();
    let mut heading_level: Option<u8> = None;

    let mut pos = 0;
    while pos < xml.len() {
        let rest = &xml[pos..];

        // Check for w:pStyle heading before paragraph
        if let Some(style_start) = rest.find("<w:pStyle ") {
            if let Some(val_start) = rest[style_start..].find("w:val=\"") {
                let val_pos = style_start + val_start + 7;
                if let Some(val_end) = rest[val_pos..].find('"') {
                    let style_val = &rest[val_pos..val_pos + val_end];
                    heading_level = parse_docx_heading(style_val);
                }
            }
        }

        // Find next <w:p
        if let Some(p_start) = rest.find("<w:p") {
            let search_from = p_start + 4;
            if let Some(p_end_rel) = rest[search_from..].find("</w:p>") {
                let p_end = search_from + p_end_rel + 6;
                let p_content = &rest[p_start..p_end];

                if p_content.contains("<w:tbl") {
                    result.push_str(&extract_docx_table_text(p_content));
                    result.push_str("\n\n");
                } else {
                    let text = xml_text_content(p_content);
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        if let Some(level) = heading_level {
                            let prefix = "#".repeat(level as usize);
                            result.push_str(&prefix);
                            result.push(' ');
                            heading_level = None;
                        }
                        result.push_str(trimmed);
                        result.push('\n');
                    }
                }
                pos += p_end;
            } else { break; }
        } else { break; }
    }

    let trimmed = result.trim().to_string();
    if trimmed.is_empty() { None } else { Some(trimmed) }
}

#[cfg(feature = "doc-p1")]
fn extract_docx_table_text(p_content: &str) -> String {
    let mut result = String::new();
    let mut pos = 0;
    let mut in_row = false;
    let mut in_cell = false;
    let mut cell_text = String::new();

    while pos < p_content.len() {
        let rest = &p_content[pos..];
        if let Some(t_start) = rest.find("<w:t") {
            if let Some(t_close) = rest[t_start..].find('>') {
                let text_start = pos + t_start + t_close + 1;
                if let Some(t_end_rel) = p_content[text_start..].find("</w:t>") {
                    let text = &p_content[text_start..text_start + t_end_rel];
                    let decoded = text.replace("&amp;", "&").replace("&lt;", "<")
                        .replace("&gt;", ">").replace("&quot;", "\"");
                    if in_cell { cell_text.push_str(&decoded); }
                    else if in_row { result.push_str(&decoded); }
                    pos = text_start + t_end_rel + 6;
                    continue;
                }
            }
        }
        if rest.starts_with("<w:tr") { in_row = true; }
        if rest.starts_with("</w:tr>") { result.push('\n'); in_row = false; }
        if rest.starts_with("<w:tc") { in_cell = true; cell_text.clear(); }
        if rest.starts_with("</w:tc>") {
            result.push_str(&cell_text);
            result.push('\t');
            in_cell = false;
        }
        pos += 1;
    }
    result
}

#[cfg(feature = "doc-p1")]
fn parse_odt_xml(xml: &str) -> Option<String> {
    let mut result = String::new();
    let mut pos = 0;
    while pos < xml.len() {
        let rest = &xml[pos..];

        // Find <text:h ...> heading
        if let Some(h_start) = rest.find("<text:h") {
            let mut heading_level: u8 = 1;
            if let Some(level_start) = rest[h_start..].find("text:outline-level=\"") {
                let l_pos = h_start + level_start + 20;
                if let Some(level_end) = rest[l_pos..].find('"') {
                    if let Ok(level) = rest[l_pos..l_pos + level_end].parse::<u8>() {
                        heading_level = level;
                    }
                }
            }
            if let Some(h_end_rel) = rest[h_start + 7..].find("</text:h>") {
                let h_end = h_start + 7 + h_end_rel + 9;
                let h_content = &rest[h_start..h_end];
                let text = xml_text_content(h_content).trim().to_string();
                if !text.is_empty() {
                    let prefix = "#".repeat(heading_level as usize);
                    result.push_str(&prefix);
                    result.push(' ');
                    result.push_str(&text);
                    result.push('\n');
                }
                pos += h_end;
                continue;
            }
        }

        // Find <text:p ...> paragraph
        if let Some(p_start) = rest.find("<text:p") {
            if let Some(p_end_rel) = rest[p_start + 7..].find("</text:p>") {
                let p_end = p_start + 7 + p_end_rel + 9;
                let p_content = &rest[p_start..p_end];
                let text = xml_text_content(p_content).trim().to_string();
                if !text.is_empty() {
                    result.push_str(&text);
                    result.push('\n');
                }
                pos += p_end;
                continue;
            }
        }
        pos += 1;
    }

    let trimmed = result.trim().to_string();
    if trimmed.is_empty() { None } else { Some(trimmed) }
}

#[cfg(feature = "doc-p1")]
fn parse_ods_xml(xml: &str) -> Option<String> {
    parse_odt_xml(xml)
}

#[cfg(feature = "doc-p1")]
fn parse_docx_heading(style_name: &str) -> Option<u8> {
    if style_name.starts_with("Heading") {
        return style_name[7..].parse().ok();
    }
    if style_name.starts_with("Title") || style_name.starts_with("title") {
        return Some(1);
    }
    None
}
