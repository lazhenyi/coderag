# CodeRAG

**AST-based semantic code indexing and retrieval — 20 languages, pure Rust, zero shell dependency.**

```
coderag index --repo . --full           # Index a repository
coderag search "parse JSON" --lang rust # Search code
coderag web                            # Start API + MCP + web UI
```

## Features

- **AST-level chunking** — Extract functions, classes, methods from source code at the syntax level, not blind text splitting
- **20 languages** — Rust, Python, JavaScript, TypeScript, Java, Go, C, C++, C#, Swift, Kotlin, PHP, Ruby, Scala, Dart, Lua, R, Perl, Bash, SQL
- **Pure Rust** — Powered by `git2` and `tree-sitter`. No shell, no external git CLI
- **Incremental indexing** — Commit-level delta updates, no full rebuilds
- **Qdrant storage** — Vector similarity search with language filtering
- **Graceful degradation** — Deterministic hash fallback if embedding API is unreachable
- **MCP server** — Model Context Protocol server for AI tool integration
- **Web UI** — Interactive search with force-graph visualization

## Quick start

```bash
# Build
cargo build --release -p coderag-cli

# Configure (or use .env / ~/.config/coderag/settings.json)
export QDRANT_URL=http://localhost:6333
export CODERAG_EMBED_URL=https://dashscope.aliyuncs.com/compatible-mode/v1/embeddings
export OPENAI_API_KEY=your_key

# Index a repo
coderag index --repo /path/to/repo --full

# Search from CLI
coderag search "parse JSON" --lang rust

# Search with graph output
coderag search "HTTP server" --graph

# Start API server with embedded web UI
coderag serve

# Start MCP server for AI tool integration
coderag mcp

# Start both API and MCP servers together
coderag web
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
| `coderag-cli` | CLI with serve, mcp, search, index commands |
| `coderag-server` | Actix-web API server with embedded React UI |
| `coderag-mcp` | MCP server with Streamable HTTP transport |
| `coderag` | Meta-crate re-exporting all sub-crates |

## CLI Commands

### `coderag index`
Index a repository for semantic search.

```bash
coderag index                    # Incremental index
coderag index --full             # Full reindex
coderag index --branch dev      # Index specific branch
coderag index --local           # Use local file storage
```

### `coderag search`
Semantic code search.

```bash
coderag search "parse JSON" --lang rust    # Language filter
coderag search "HTTP handler" -n 20         # Limit results
coderag search "auth middleware" --graph     # Force-graph output
coderag search "database query" --local      # Local storage
```

### `coderag serve`
Start the API server with embedded React web UI.

```bash
coderag serve --bind 0.0.0.0:8080
# Web UI:  http://localhost:8080/
# API:     http://localhost:8080/api
# Health:  http://localhost:8080/health
```

### `coderag mcp`
Start the MCP server for AI tool integration (Claude, Cursor, etc.).

```bash
coderag mcp --bind 127.0.0.1:8081
# MCP: http://localhost:8081/mcp
```

**MCP Tools:**
- `search` — Semantic code search with query, language filter, limit, score threshold
- `search_graph` — Search with force-graph node/link relationships in JSON
- `index` — Trigger repository indexing
- `index_status` — Check indexing progress
- `repo_info` — Repository metadata (branch, HEAD, file count)
- `repo_tree` — File tree listing
- `list_languages` — All supported languages

### `coderag web`
Start both API server and MCP server together.

```bash
coderag web                          # Default ports (API: 8080, MCP: 8081)
coderag web --api-bind 9000 --mcp-bind 9001
```

### `coderag config`
Manage configuration.

```bash
coderag config show          # Show current config
coderag config init          # Create default ~/.config/coderag/settings.json
coderag config set qdrant_url http://localhost:6333
coderag config path          # Show config file locations
```

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
CODERAG_MCP_BIND=127.0.0.1:8081
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

## MCP Integration

Configure in your MCP client (Claude Desktop, Cursor, etc.):

**macOS:**
```json
{
  "mcpServers": {
    "coderag": {
      "command": "/path/to/coderag-mcp",
      "env": {
        "CODERAG_REPO": "/path/to/your/repo",
        "QDRANT_URL": "http://localhost:6333",
        "OPENAI_API_KEY": "sk-..."
      }
    }
  }
}
```

**Windows:**
```json
{
  "mcpServers": {
    "coderag": {
      "command": "C:\\path\\to\\coderag-mcp.exe",
      "env": {
        "CODERAG_REPO": "C:\\path\\to\\your\\repo",
        "QDRANT_URL": "http://localhost:6333",
        "OPENAI_API_KEY": "sk-..."
      }
    }
  }
}
```

Or use the CLI:
```bash
coderag mcp
```

## Requirements

- Rust 1.93+ (edition 2024)
- Qdrant (for vector storage; optional with `--local` flag)

## License

MIT
