//! Chunker implementation

use crate::analyzer::symbol::Symbol;
use crate::chunker::types::{Chunk, ChunkConfig};
use sha2::{Sha256, Digest};

/// Chunker for creating code chunks
pub struct Chunker { config: ChunkConfig }

impl Chunker {
    pub fn new() -> Self { Self { config: ChunkConfig::default() } }
    pub fn with_config(config: ChunkConfig) -> Self { Self { config } }

    pub fn chunk_symbol(&self, symbol: &Symbol, repo: &str, branch: &str, commit: &str) -> Vec<Chunk> {
        let mut chunks = vec![self.create_code_chunk(symbol, repo, branch, commit)];
        if self.config.include_docs {
            if let Some(ref doc) = symbol.doc {
                if !doc.is_empty() {
                    chunks.push(self.create_doc_chunk(symbol, repo, branch, commit));
                }
            }
        }
        chunks
    }

    fn create_code_chunk(&self, symbol: &Symbol, repo: &str, branch: &str, commit: &str) -> Chunk {
        let id = uuid::Uuid::new_v4().to_string();
        let content = format!("{}{}{}{}{}", symbol.code, symbol.signature, symbol.file_path, symbol.module_path, symbol.language);
        let content_hash = compute_hash(&content);
        Chunk {
            id, content_hash, repo: repo.to_string(), branch: branch.to_string(), commit: commit.to_string(),
            language: symbol.language.clone(), file: symbol.file_path.clone(), module: symbol.module_path.clone(),
            symbol: symbol.name.clone(), kind: symbol.kind.to_string(),
            signature: if self.config.include_signatures { symbol.signature.clone() } else { String::new() },
            doc: if self.config.include_docs { symbol.doc.clone() } else { None },
            code: symbol.code.clone(), start_line: symbol.line_start, end_line: symbol.line_end,
        }
    }

    fn create_doc_chunk(&self, symbol: &Symbol, repo: &str, branch: &str, commit: &str) -> Chunk {
        let id = uuid::Uuid::new_v4().to_string();
        let doc_content = format!("{}\n{}", symbol.signature, symbol.doc.as_ref().unwrap());
        let content = format!("{}{}{}{}{}", doc_content, symbol.signature, symbol.file_path, symbol.module_path, symbol.language);
        let content_hash = compute_hash(&content);
        Chunk {
            id, content_hash, repo: repo.to_string(), branch: branch.to_string(), commit: commit.to_string(),
            language: symbol.language.clone(), file: symbol.file_path.clone(), module: symbol.module_path.clone(),
            symbol: symbol.name.clone(), kind: format!("{}_doc", symbol.kind),
            signature: symbol.signature.clone(), doc: symbol.doc.clone(), code: doc_content,
            start_line: symbol.line_start, end_line: symbol.line_start,
        }
    }

    pub fn chunk_symbols(&self, symbols: &[Symbol], repo: &str, branch: &str, commit: &str) -> Vec<Chunk> {
        symbols.iter().flat_map(|s| self.chunk_symbol(s, repo, branch, commit)).collect()
    }
}

impl Default for Chunker {
    fn default() -> Self { Self::new() }
}

fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}
