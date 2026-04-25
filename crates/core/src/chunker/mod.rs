//! Chunker Module
//!
//! Creates code chunks from extracted symbols for embedding.

use crate::analyzer::symbol::Symbol;
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};

/// A code chunk for embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// Unique identifier (UUID)
    pub id: String,
    /// Content hash for deduplication
    pub content_hash: String,
    /// Repository name
    pub repo: String,
    /// Branch name
    pub branch: String,
    /// Commit OID
    pub commit: String,
    /// Programming language
    pub language: String,
    /// File path relative to repository root
    pub file: String,
    /// Module path (e.g., "crate::module::Class")
    pub module: String,
    /// Symbol name
    pub symbol: String,
    /// Symbol kind (function, class, method, etc.)
    pub kind: String,
    /// Function/class signature
    pub signature: String,
    /// Documentation comment
    pub doc: Option<String>,
    /// Full source code of the symbol
    pub code: String,
    /// Starting line number
    pub start_line: usize,
    /// Ending line number
    pub end_line: usize,
}

/// Configuration for chunking
#[derive(Debug, Clone)]
pub struct ChunkConfig {
    /// Maximum chunk size in characters
    pub max_chunk_size: usize,
    /// Include doc comments in chunks
    pub include_docs: bool,
    /// Include function signatures
    pub include_signatures: bool,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            max_chunk_size: 2000,
            include_docs: true,
            include_signatures: true,
        }
    }
}

/// Chunker for creating code chunks
pub struct Chunker {
    config: ChunkConfig,
}

impl Chunker {
    /// Create a new chunker with default config
    pub fn new() -> Self {
        Self {
            config: ChunkConfig::default(),
        }
    }

    /// Create a new chunker with custom config
    pub fn with_config(config: ChunkConfig) -> Self {
        Self { config }
    }

    /// Create chunks from a symbol
    pub fn chunk_symbol(&self, symbol: &Symbol, repo: &str, branch: &str, commit: &str) -> Vec<Chunk> {
        let mut chunks = Vec::new();

        // Create the main code chunk
        let code_chunk = self.create_code_chunk(symbol, repo, branch, commit);
        chunks.push(code_chunk);

        // Create a doc chunk if documentation exists
        if self.config.include_docs {
            if let Some(ref doc) = symbol.doc {
                if !doc.is_empty() {
                    let doc_chunk = self.create_doc_chunk(symbol, repo, branch, commit);
                    chunks.push(doc_chunk);
                }
            }
        }

        chunks
    }

    /// Create the main code chunk
    fn create_code_chunk(&self, symbol: &Symbol, repo: &str, branch: &str, commit: &str) -> Chunk {
        let id = uuid::Uuid::new_v4().to_string();

        // Build content for hashing
        let content = format!(
            "{}{}{}{}{}",
            symbol.code,
            symbol.signature,
            symbol.file_path,
            symbol.module_path,
            symbol.language
        );
        let content_hash = self.compute_hash(&content);

        Chunk {
            id,
            content_hash,
            repo: repo.to_string(),
            branch: branch.to_string(),
            commit: commit.to_string(),
            language: symbol.language.clone(),
            file: symbol.file_path.clone(),
            module: symbol.module_path.clone(),
            symbol: symbol.name.clone(),
            kind: symbol.kind.to_string(),
            signature: if self.config.include_signatures {
                symbol.signature.clone()
            } else {
                String::new()
            },
            doc: if self.config.include_docs {
                symbol.doc.clone()
            } else {
                None
            },
            code: symbol.code.clone(),
            start_line: symbol.line_start,
            end_line: symbol.line_end,
        }
    }

    /// Create a documentation chunk
    fn create_doc_chunk(&self, symbol: &Symbol, repo: &str, branch: &str, commit: &str) -> Chunk {
        let id = uuid::Uuid::new_v4().to_string();

        let doc_content = format!(
            "{}\n{}",
            symbol.signature,
            symbol.doc.as_ref().unwrap()
        );

        let content = format!(
            "{}{}{}{}{}",
            doc_content,
            symbol.signature,
            symbol.file_path,
            symbol.module_path,
            symbol.language
        );
        let content_hash = self.compute_hash(&content);

        Chunk {
            id,
            content_hash,
            repo: repo.to_string(),
            branch: branch.to_string(),
            commit: commit.to_string(),
            language: symbol.language.clone(),
            file: symbol.file_path.clone(),
            module: symbol.module_path.clone(),
            symbol: symbol.name.clone(),
            kind: format!("{}_doc", symbol.kind),
            signature: symbol.signature.clone(),
            doc: symbol.doc.clone(),
            code: doc_content,
            start_line: symbol.line_start,
            end_line: symbol.line_start, // Doc usually on one line
        }
    }

    /// Compute SHA256 hash of content
    fn compute_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Chunk multiple symbols
    pub fn chunk_symbols(
        &self,
        symbols: &[Symbol],
        repo: &str,
        branch: &str,
        commit: &str,
    ) -> Vec<Chunk> {
        symbols
            .iter()
            .flat_map(|s| self.chunk_symbol(s, repo, branch, commit))
            .collect()
    }
}

impl Default for Chunker {
    fn default() -> Self {
        Self::new()
    }
}

/// Embedding text representation of a chunk
pub fn chunk_to_embedding_text(chunk: &Chunk) -> String {
    let mut parts = Vec::new();

    // Language tag
    parts.push(format!("[{}] ", chunk.language));

    // Signature
    if !chunk.signature.is_empty() {
        parts.push(chunk.signature.clone());
    }

    // Documentation
    if let Some(ref doc) = chunk.doc {
        if !doc.is_empty() {
            parts.push(doc.clone());
        }
    }

    // Code
    parts.push(chunk.code.clone());

    parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::symbol::SymbolKind;

    #[test]
    fn test_chunk_creation() {
        let chunker = Chunker::new();

        let symbol = Symbol::new(
            "hello".to_string(),
            SymbolKind::Function,
            1,
            5,
            "rust".to_string(),
        )
        .with_file_path("src/lib.rs".to_string())
        .with_module_path("crate".to_string())
        .with_signature("fn hello()".to_string())
        .with_code("fn hello() {\n    println!(\"Hello!\");\n}".to_string())
        .with_doc(Some("Says hello".to_string()));

        let chunks = chunker.chunk_symbol(&symbol, "test-repo", "main", "abc123");

        assert_eq!(chunks.len(), 2); // Code chunk + doc chunk

        let code_chunk = &chunks[0];
        assert_eq!(code_chunk.symbol, "hello");
        assert_eq!(code_chunk.kind, "function");
        assert_eq!(code_chunk.language, "rust");
        assert!(!code_chunk.content_hash.is_empty());
    }

    #[test]
    fn test_content_hash() {
        let chunker = Chunker::new();

        let symbol = Symbol::new(
            "test".to_string(),
            SymbolKind::Function,
            1,
            10,
            "rust".to_string(),
        )
        .with_code("fn test() {}".to_string())
        .with_signature("fn test()".to_string())
        .with_file_path("test.rs".to_string())
        .with_module_path("".to_string());

        let chunks = chunker.chunk_symbol(&symbol, "repo", "main", "commit");

        // Same content should produce same hash
        let chunks2 = chunker.chunk_symbol(&symbol, "repo", "main", "commit");

        assert_eq!(chunks[0].content_hash, chunks2[0].content_hash);

        // Different content should produce different hash
        let mut symbol2 = symbol.clone();
        symbol2.code = "fn test2() {}".to_string();
        let chunks3 = chunker.chunk_symbol(&symbol2, "repo", "main", "commit");

        assert_ne!(chunks[0].content_hash, chunks3[0].content_hash);
    }

    #[test]
    fn test_embedding_text() {
        let chunk = Chunk {
            id: "test".to_string(),
            content_hash: "abc".to_string(),
            repo: "test-repo".to_string(),
            branch: "main".to_string(),
            commit: "123".to_string(),
            language: "rust".to_string(),
            file: "lib.rs".to_string(),
            module: "crate".to_string(),
            symbol: "hello".to_string(),
            kind: "function".to_string(),
            signature: "fn hello() -> String".to_string(),
            doc: Some("Returns greeting".to_string()),
            code: "fn hello() -> String { \"Hello\".to_string() }".to_string(),
            start_line: 1,
            end_line: 2,
        };

        let text = chunk_to_embedding_text(&chunk);
        assert!(text.starts_with("[rust]"));
        assert!(text.contains("fn hello()"));
        assert!(text.contains("Returns greeting"));
    }
}
