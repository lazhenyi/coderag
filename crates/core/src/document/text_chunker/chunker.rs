//! Text chunking logic

use crate::chunker::Chunk;
use crate::document::text_chunker::config::TextChunkConfig;
use sha2::{Digest, Sha256};

/// Split text into chunks with overlap
pub fn chunk_text(
    text: &str,
    config: &TextChunkConfig,
    language: &str,
    file_path: &str,
    repo: &str,
    branch: &str,
    commit: &str,
) -> Vec<Chunk> {
    let text = text.trim();
    if text.is_empty() {
        return vec![];
    }

    let mut chunks = Vec::new();

    if text.len() <= config.max_chunk_size {
        chunks.push(make_chunk(text, 1, 1, language, file_path, repo, branch, commit));
        return chunks;
    }

    let paragraphs: Vec<&str> = text
        .split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .collect();

    let mut current_chunk = String::new();
    let mut chunk_start_line = 1;
    let mut current_line = 1;

    for para in &paragraphs {
        let para_lines = para.lines().count();

        if para.len() > config.max_chunk_size {
            if !current_chunk.is_empty() {
                let chunk_lines = current_chunk.lines().count();
                chunks.push(make_chunk(
                    &current_chunk, chunk_start_line, chunk_start_line + chunk_lines - 1,
                    language, file_path, repo, branch, commit,
                ));
                current_chunk.clear();
            }

            let mut remaining: &str = para;
            let mut line_offset = current_line;
            while remaining.len() > config.max_chunk_size {
                let mut split_at = config.max_chunk_size;
                if let Some(idx) = remaining[..split_at].rfind(char::is_whitespace) {
                    if idx > 10 {
                        split_at = idx;
                    }
                }
                let part = &remaining[..split_at];
                chunks.push(make_chunk(
                    part.trim(), line_offset, line_offset + part.lines().count(),
                    language, file_path, repo, branch, commit,
                ));
                remaining = &remaining[split_at..];
                line_offset += part.lines().count();
            }
            current_chunk = remaining.trim().to_string();
            chunk_start_line = line_offset;
        } else if current_chunk.len() + para.len() + 2 > config.max_chunk_size && !current_chunk.is_empty() {
            let chunk_lines = current_chunk.lines().count();
            chunks.push(make_chunk(
                &current_chunk, chunk_start_line, chunk_start_line + chunk_lines - 1,
                language, file_path, repo, branch, commit,
            ));

            let overlap_text = if current_chunk.len() > config.overlap {
                let overlap_start = current_chunk.len() - config.overlap;
                let idx = current_chunk[overlap_start..]
                    .find(char::is_whitespace)
                    .map(|i| overlap_start + i)
                    .unwrap_or(overlap_start);
                current_chunk[idx..].to_string()
            } else {
                current_chunk.clone()
            };

            current_chunk = overlap_text.trim().to_string();
            chunk_start_line = current_line - current_chunk.lines().count();
            if chunk_start_line < 1 {
                chunk_start_line = 1;
            }

            if !current_chunk.is_empty() {
                current_chunk.push_str("\n\n");
            }
            current_chunk.push_str(para);
        } else {
            if !current_chunk.is_empty() {
                current_chunk.push_str("\n\n");
            }
            current_chunk.push_str(para);
        }

        current_line += para_lines;
    }

    if !current_chunk.is_empty() {
        let chunk_lines = current_chunk.lines().count();
        chunks.push(make_chunk(
            &current_chunk, chunk_start_line, chunk_start_line + chunk_lines - 1,
            language, file_path, repo, branch, commit,
        ));
    }

    if chunks.is_empty() {
        let lines: Vec<&str> = text.lines().collect();
        let mut buf = String::new();
        let mut start = 1;
        let mut line_num = 1;

        for line in &lines {
            if buf.len() + line.len() + 1 > config.max_chunk_size && !buf.is_empty() {
                chunks.push(make_chunk(&buf, start, start + buf.lines().count() - 1, language, file_path, repo, branch, commit));
                buf = String::new();
                start = line_num;
            }
            if !buf.is_empty() {
                buf.push('\n');
            }
            buf.push_str(line);
            line_num += 1;
        }
        if !buf.is_empty() {
            chunks.push(make_chunk(&buf, start, start + buf.lines().count() - 1, language, file_path, repo, branch, commit));
        }
    }

    chunks
}

/// Create a Chunk from text content
fn make_chunk(
    text: &str,
    start_line: usize,
    end_line: usize,
    language: &str,
    file_path: &str,
    repo: &str,
    branch: &str,
    commit: &str,
) -> Chunk {
    let id = uuid::Uuid::new_v4().to_string();
    let content_hash = compute_hash(text);

    let signature = text
        .lines()
        .find(|l| !l.trim().is_empty())
        .map(|l| {
            let trimmed = l.trim();
            if trimmed.len() > 80 {
                format!("{}...", &trimmed[..80])
            } else {
                trimmed.to_string()
            }
        })
        .unwrap_or_else(|| "(empty)".to_string());

    Chunk {
        id,
        content_hash,
        repo: repo.to_string(),
        branch: branch.to_string(),
        commit: commit.to_string(),
        language: language.to_string(),
        file: file_path.to_string(),
        module: String::new(),
        symbol: String::new(),
        kind: "document".to_string(),
        signature,
        doc: None,
        code: text.to_string(),
        start_line,
        end_line,
    }
}

fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}
