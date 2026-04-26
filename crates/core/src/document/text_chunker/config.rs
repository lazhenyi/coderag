//! Text chunker configuration

/// Configuration for text chunking
#[derive(Debug, Clone)]
pub struct TextChunkConfig {
    /// Maximum chunk size in characters
    pub max_chunk_size: usize,
    /// Overlap between chunks for context continuity
    pub overlap: usize,
    /// Strip markdown formatting for cleaner text
    pub strip_markdown: bool,
}

impl Default for TextChunkConfig {
    fn default() -> Self {
        Self {
            max_chunk_size: 2000,
            overlap: 200,
            strip_markdown: true,
        }
    }
}
