# CodeRAG

**AST-based semantic code indexing and retrieval — 20 languages, pure Rust, zero shell dependency.**

```
cargo add coderag
```

```
coderag index --repo . --full          # Index a repository
coderag search "parse JSON" --lang rust # Search code
coderag eval                            # Quality evaluation
```

## Features

- **AST-level chunking** — Extract functions, classes, methods from source code at the syntax level, not blind text splitting
- **20 languages** — Rust, Python, JavaScript, TypeScript, Java, Go, C, C++, C#, Swift, Kotlin, PHP, Ruby, Scala, Dart, Lua, R, Perl, Bash, SQL
- **Pure Rust** — Powered by `git2` and `tree-sitter`. No shell, no external git CLI
- **Incremental indexing** — Commit-level delta updates, no full rebuilds
- **Qdrant storage** — Vector similarity search with language filtering
- **Graceful degradation** — Deterministic hash fallback if embedding API is unreachable

## Quick start

```bash
# Install
cargo build --release -p coderag-cli

# Index a repo
export QDRANT_URL=http://localhost:6333
export CODERAG_EMBED_URL=https://dashscope.aliyuncs.com/compatible-mode/v1/embeddings
export OPENAI_API_KEY=your_key

./target/release/coderag index --repo /path/to/repo --full

# Search
./target/release/coderag search "parse JSON" --lang rust

# Or add to your project
[dependencies]
coderag = "0.1"
```

## Architecture

```
Git Repo ──► Parser ──► Analyzer ──► Chunker ──► Embedder ──► Qdrant
              │                        │           │
         tree-sitter              symbol AST    semantic
           (20 lang)              extraction   chunks
```

## Crates

| Crate | Description |
|-------|-------------|
| `coderag-core` | Git operations, parsers, analyzers, chunker, embedder |
| `coderag-storage` | Qdrant client and repository layer |
| `coderag-indexer` | Full/incremental indexing orchestration |
| `coderag` | Meta-crate re-exporting all sub-crates |

## Configuration

Three-layer priority: **env vars > .env > ~/.config/coderag/settings.json**

```bash
# Environment variables
QDRANT_URL=http://localhost:6333
QDRANT_API_KEY=
OPENAI_API_KEY=sk-...
CODERAG_EMBED_URL=https://dashscope.aliyuncs.com/compatible-mode/v1/embeddings
CODERAG_EMBED_MODEL=text-embedding-v4
CODERAG_EMBED_DIMENSION=1024
CODERAG_REPO=.
CODERAG_BRANCH=main
CODERAG_COLLECTION=coderag
CODERAG_STATE_FILE=.coderag/state.json
CODERAG_BATCH_SIZE=100
```

Or in `~/.config/coderag/settings.json`:

```json
{
  "qdrantUrl": "http://localhost:6333",
  "openaiApiKey": "sk-...",
  "embedUrl": "https://dashscope.aliyuncs.com/compatible-mode/v1/embeddings",
  "embedModel": "text-embedding-v4",
  "embedDimension": 1024,
  "collectionName": "coderag"
}
```

## Requirements

- Rust 1.93+ (edition 2024)
- Qdrant (for vector storage)

## License

MIT
