//! XLSX extraction using calamine

/// Extract text from .xlsx
pub fn extract_xlsx(content: &[u8]) -> Option<String> {
    #[cfg(feature = "doc-p1")]
    {
        use calamine::{Data, Reader, Xlsx};
        use std::io::Cursor;

        let mut workbook: Xlsx<_> = match Xlsx::new(Cursor::new(content)) {
            Ok(w) => w,
            Err(e) => {
                tracing::warn!("Failed to open XLSX: {}", e);
                return None;
            }
        };

        let mut result = String::new();
        let sheet_names = workbook.sheet_names().to_vec();
        for sheet_name in &sheet_names {
            if let Ok(range) = workbook.worksheet_range(sheet_name) {
                if !result.is_empty() {
                    result.push_str("\n--- Sheet: ");
                    result.push_str(sheet_name);
                    result.push_str(" ---\n");
                } else if sheet_names.len() > 1 {
                    result.push_str("--- Sheet: ");
                    result.push_str(sheet_name);
                    result.push_str(" ---\n");
                }
                for row in range.rows() {
                    let cells: Vec<String> = row
                        .iter()
                        .map(|c| match c {
                            Data::Empty => String::new(),
                            _ => c.to_string(),
                        })
                        .collect();
                    if cells.iter().any(|c| !c.is_empty()) {
                        result.push_str(&cells.join("\t"));
                        result.push('\n');
                    }
                }
            }
        }

        let trimmed = result.trim().to_string();
        if trimmed.is_empty() { None } else { Some(trimmed) }
    }

    #[cfg(not(feature = "doc-p1"))]
    {
        let _ = content;
        tracing::warn!("P1 document features not enabled. Add 'doc-p1' feature to coderag-core.");
        None
    }
}
