//! MCP server implementation with CodeRAG tools

use std::borrow::Cow;

use futures::future::BoxFuture;
use parking_lot::RwLock;
use rmcp::ErrorData as McpError;
use rmcp::ServerHandler;
use rmcp::handler::server::common::cached_schema_for_type;
use rmcp::handler::server::router::tool::ToolRoute;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::ToolCallContext;
use rmcp::model::{CallToolRequestParam, ListToolsResult, *};
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;

use crate::config::McpConfig;

#[derive(Clone)]
pub struct CodeRagServer {
    pub config: Arc<McpConfig>,
    pub indexing_status: Arc<RwLock<IndexingStatus>>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct IndexingStatus {
    is_indexing: bool,
    progress: Option<IndexProgress>,
    last_error: Option<String>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct IndexProgress {
    files_processed: usize,
    symbols_extracted: usize,
    chunks_created: usize,
}

// ─── Tool request types ─────────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
struct SearchRequest {
    /// Semantic search query
    query: String,
    /// Filter by language (e.g. "Rust", "Python", "JavaScript")
    #[serde(default)]
    language: Option<String>,
    /// Maximum number of results
    #[serde(default = "default_limit")]
    limit: usize,
    /// Use local file storage instead of Qdrant
    #[serde(default)]
    local: bool,
    /// Minimum relevance score (0.0–1.0)
    #[serde(default = "default_threshold")]
    score_threshold: f32,
}

fn default_limit() -> usize {
    10
}
fn default_threshold() -> f32 {
    0.5
}

#[derive(Debug, Deserialize, JsonSchema)]
struct IndexRequest {
    /// Branch to index
    #[serde(default = "default_branch")]
    branch: String,
    /// Use local file storage instead of Qdrant
    #[serde(default)]
    local: bool,
}

fn default_branch() -> String {
    "main".into()
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GraphSearchRequest {
    /// Semantic search query
    query: String,
    /// Filter by language
    #[serde(default)]
    language: Option<String>,
    /// Maximum number of results
    #[serde(default = "default_limit")]
    limit: usize,
    /// Use local file storage instead of Qdrant
    #[serde(default)]
    local: bool,
    /// Minimum relevance score (0.0–1.0)
    #[serde(default = "default_threshold")]
    score_threshold: f32,
}

// ─── Helpers ───────────────────────────────────────────────────────

fn parse_args<T: for<'de> Deserialize<'de>>(
    ctx: &mut ToolCallContext<'_, CodeRagServer>,
) -> Result<T, McpError> {
    let args = ctx.arguments.take().unwrap_or_default();
    serde_json::from_value(serde_json::Value::Object(args)).map_err(|e| {
        McpError::invalid_params(format!("failed to deserialize parameters: {}", e), None)
    })
}

fn err_internal(msg: String) -> McpError {
    McpError::internal_error(msg, None)
}

fn input_schema<T: JsonSchema + 'static>() -> Arc<serde_json::Map<String, serde_json::Value>> {
    cached_schema_for_type::<T>()
}

fn empty_schema() -> Arc<serde_json::Map<String, serde_json::Value>> {
    cached_schema_for_type::<()>()
}

async fn create_storage(
    config: &McpConfig,
    use_local: bool,
) -> anyhow::Result<coderag_storage::StorageBackend> {
    if use_local {
        coderag_storage::StorageBackend::from_config(&coderag_storage::StorageConfig::Local {
            store_path: config.repo_path.clone(),
            is_bare: false,
        })
    } else {
        let storage_config = coderag_storage::QdrantConfig {
            url: config.qdrant_url.clone(),
            api_key: config.qdrant_api_key.clone(),
            collection_name: config.collection_name.clone(),
            ..Default::default()
        };
        coderag_storage::StorageBackend::from_config(&coderag_storage::StorageConfig::Qdrant(
            storage_config,
        ))
    }
}

fn create_embedder(config: &McpConfig) -> anyhow::Result<coderag_core::embedder::Embedder> {
    let embedder_config = coderag_core::embedder::EmbedderConfig {
        api_url: config.embed_url.clone(),
        model: config.embed_model.clone(),
        dimension: config.embed_dimension,
        api_key: config.embed_api_key.clone(),
        ..Default::default()
    };
    coderag_core::embedder::Embedder::new(embedder_config)
}

fn make_query_chunk(
    config: &McpConfig,
    query: &str,
    language: &str,
) -> coderag_core::chunker::Chunk {
    coderag_core::chunker::Chunk {
        id: "query".into(),
        content_hash: "".into(),
        repo: config.repo_path.clone(),
        branch: "main".into(),
        commit: "".into(),
        language: language.to_string(),
        file: "".into(),
        module: "".into(),
        symbol: "".into(),
        kind: "query".into(),
        signature: query.to_string(),
        doc: None,
        code: query.to_string(),
        start_line: 0,
        end_line: 0,
    }
}

fn result_to_markdown(
    index: usize,
    symbol: &str,
    kind: &str,
    file: &str,
    language: &str,
    score: f32,
    code: &str,
    doc: Option<&str>,
    signature: &str,
    start_line: usize,
    end_line: usize,
) -> String {
    let mut out = String::new();
    out.push_str(&format!("### {}. `{}` [{}]\n\n", index, symbol, kind));
    out.push_str(&format!("**File:** `{}` ({})\n", file, language));
    out.push_str(&format!("**Score:** {:.3}\n", score));
    if start_line > 0 {
        out.push_str(&format!("**Lines:** {}–{}\n", start_line, end_line));
    }
    if let Some(d) = doc {
        if !d.is_empty() {
            out.push_str(&format!("\n**Doc:** {}\n", d));
        }
    }
    if !signature.is_empty() && signature != symbol {
        out.push_str(&format!("\n**Signature:** `{}`\n", signature));
    }
    out.push_str(&format!("\n```{}\n{}\n```\n", language, code));
    out
}

// ─── Tools ──────────────────────────────────────────────────────────

fn search_tool() -> ToolRoute<CodeRagServer> {
    ToolRoute::new_dyn(
        Tool {
            name: "search".into(),
            title: None,
            description: Some("Search for code symbols using semantic similarity. Returns matching symbols with code snippets, relevance scores, and documentation.".into()),
            input_schema: input_schema::<SearchRequest>(),
            output_schema: None,
            annotations: Some(ToolAnnotations {
                title: None,
                read_only_hint: Some(true),
                destructive_hint: Some(false),
                idempotent_hint: Some(true),
                open_world_hint: Some(false),
            }),
            icons: None,
        },
        |mut ctx: ToolCallContext<'_, CodeRagServer>| {
            let config = ctx.service.config.clone();
            let args = match parse_args::<SearchRequest>(&mut ctx) {
                Ok(a) => a,
                Err(e) => return Box::pin(std::future::ready(Err(e))),
            };
            Box::pin(async move {
                let storage = create_storage(&config, args.local)
                    .await
                    .map_err(|e| err_internal(format!("Storage init failed: {}", e)))?;

                let embedder = create_embedder(&config)
                    .map_err(|e| err_internal(format!("Embedder init failed: {}", e)))?;

                let lang = args.language.as_deref().unwrap_or("");
                let query_chunk = make_query_chunk(&config, &args.query, lang);

                let embedding = embedder.embed_chunk(&query_chunk).await
                    .map_err(|e| err_internal(format!("Embedding failed: {}", e)))?;

                let filter = coderag_storage::SearchFilter {
                    language: args.language.clone(),
                    file: None,
                    kind: None,
                    repo: None,
                };

                let options = coderag_storage::SearchOptions {
                    limit: args.limit,
                    score_threshold: Some(args.score_threshold),
                    filter: if filter.language.is_some() { Some(filter) } else { None },
                    ..Default::default()
                };

                let results = storage.search(&embedding.vector, options).await
                    .map_err(|e| err_internal(format!("Search failed: {}", e)))?;

                if results.is_empty() {
                    return Ok(CallToolResult::success(vec![
                        Content::text(format!("No results found for query: \"{}\"", args.query))
                    ]));
                }

                let mut text = String::new();
                text.push_str(&format!("## Search Results ({} found)\n\n", results.len()));
                for (i, r) in results.iter().enumerate() {
                    text.push_str(&result_to_markdown(
                        i + 1, &r.payload.symbol, &r.payload.kind, &r.payload.file,
                        &r.payload.language, r.score, &r.payload.code,
                        r.payload.doc.as_deref(), &r.payload.signature,
                        r.payload.start_line, r.payload.end_line,
                    ));
                }

                Ok(CallToolResult::success(vec![Content::text(text)]))
            })
        },
    )
}

fn search_graph_tool() -> ToolRoute<CodeRagServer> {
    ToolRoute::new_dyn(
        Tool {
            name: "search_graph".into(),
            title: None,
            description: Some("Search for code and return results as a force-graph with node/link relationships. Use for visualizing connections between symbols.".into()),
            input_schema: input_schema::<GraphSearchRequest>(),
            output_schema: None,
            annotations: Some(ToolAnnotations {
                title: None,
                read_only_hint: Some(true),
                destructive_hint: Some(false),
                idempotent_hint: Some(true),
                open_world_hint: Some(false),
            }),
            icons: None,
        },
        |mut ctx: ToolCallContext<'_, CodeRagServer>| {
            let config = ctx.service.config.clone();
            let args = match parse_args::<GraphSearchRequest>(&mut ctx) {
                Ok(a) => a,
                Err(e) => return Box::pin(std::future::ready(Err(e))),
            };
            Box::pin(async move {
                let storage = create_storage(&config, args.local)
                    .await
                    .map_err(|e| err_internal(format!("Storage init failed: {}", e)))?;

                let embedder = create_embedder(&config)
                    .map_err(|e| err_internal(format!("Embedder init failed: {}", e)))?;

                let lang = args.language.as_deref().unwrap_or("");
                let query_chunk = make_query_chunk(&config, &args.query, lang);

                let embedding = embedder.embed_chunk(&query_chunk).await
                    .map_err(|e| err_internal(format!("Embedding failed: {}", e)))?;

                let filter = coderag_storage::SearchFilter {
                    language: args.language.clone(),
                    file: None,
                    kind: None,
                    repo: None,
                };

                let options = coderag_storage::SearchOptions {
                    limit: args.limit,
                    score_threshold: Some(args.score_threshold),
                    filter: if filter.language.is_some() { Some(filter) } else { None },
                    ..Default::default()
                };

                let results = storage.search(&embedding.vector, options).await
                    .map_err(|e| err_internal(format!("Search failed: {}", e)))?;

                let mut nodes_json = Vec::new();
                for r in &results {
                    nodes_json.push(serde_json::json!({
                        "id": r.id,
                        "name": r.payload.symbol,
                        "kind": r.payload.kind,
                        "file": r.payload.file,
                        "language": r.payload.language,
                        "score": r.score,
                        "code": r.payload.code,
                    }));
                }

                let mut links = Vec::new();
                for i in 0..results.len() {
                    for j in (i + 1)..results.len() {
                        let a = &results[i];
                        let b = &results[j];
                        if a.payload.file == b.payload.file && !a.payload.file.is_empty() {
                            links.push(serde_json::json!({ "source": a.id, "target": b.id, "relation": "same_file" }));
                        }
                        if a.payload.module == b.payload.module && !a.payload.module.is_empty() {
                            links.push(serde_json::json!({ "source": a.id, "target": b.id, "relation": "same_module" }));
                        }
                    }
                }

                let graph_json = serde_json::json!({
                    "nodes": nodes_json,
                    "links": links,
                    "total": results.len(),
                });

                let mut text = String::new();
                text.push_str(&format!("## Graph Search Results ({} nodes, {} links)\n\n", results.len(), links.len()));
                for (i, r) in results.iter().enumerate() {
                    text.push_str(&format!(
                        "{}. `{}` [{}] in `{}` (score: {:.3})\n",
                        i + 1, r.payload.symbol, r.payload.kind, r.payload.file, r.score
                    ));
                }
                if !links.is_empty() {
                    text.push_str("\n### Relationships\n\n");
                    for link in &links {
                        text.push_str(&format!(
                            "- {} -> {} ({})\n",
                            link["source"], link["target"], link["relation"]
                        ));
                    }
                }
                text.push_str(&format!("\n### Full Graph JSON\n\n```json\n{}\n```\n", serde_json::to_string_pretty(&graph_json).unwrap_or_default()));

                Ok(CallToolResult::success(vec![Content::text(text)]))
            })
        },
    )
}

fn index_tool() -> ToolRoute<CodeRagServer> {
    ToolRoute::new_dyn(
        Tool {
            name: "index".into(),
            title: None,
            description: Some("Trigger repository indexing. Creates vector embeddings for all code symbols. Returns status.".into()),
            input_schema: input_schema::<IndexRequest>(),
            output_schema: None,
            annotations: Some(ToolAnnotations {
                title: None,
                read_only_hint: Some(false),
                destructive_hint: Some(false),
                idempotent_hint: Some(true),
                open_world_hint: Some(false),
            }),
            icons: None,
        },
        |mut ctx: ToolCallContext<'_, CodeRagServer>| {
            let status = ctx.service.indexing_status.read();
            if status.is_indexing {
                return Box::pin(std::future::ready(Ok(CallToolResult::success(vec![
                    Content::text("Indexing already in progress. Check status with the `index_status` tool.")
                ]))));
            }
            drop(status);

            let config = ctx.service.config.clone();
            let status_ref = ctx.service.indexing_status.clone();
            let args = match parse_args::<IndexRequest>(&mut ctx) {
                Ok(a) => a,
                Err(e) => return Box::pin(std::future::ready(Err(e))),
            };
            let branch = args.branch.clone();
            let repo_path = config.repo_path.clone();

            Box::pin(async move {
                // Mark indexing as started immediately
                {
                    let mut s = status_ref.write();
                    s.is_indexing = true;
                    s.last_error = None;
                    s.progress = None;
                }

                let config_clone = (*config).clone();
                let branch_clone = args.branch.clone();
                let local = args.local;

                // Run blocking indexing work in a spawned blocking task
                let result = tokio::task::spawn_blocking(move || {
                    let index_config = coderag_indexer::IndexConfig {
                        repo_path: config_clone.repo_path.clone(),
                        branch: branch_clone.clone(),
                        qdrant_url: config_clone.qdrant_url.clone(),
                        qdrant_api_key: config_clone.qdrant_api_key.clone(),
                        collection_name: config_clone.collection_name.clone(),
                        batch_size: config_clone.batch_size,
                        embed_api_url: Some(config_clone.embed_url.clone()),
                        embed_api_key: config_clone.embed_api_key.clone(),
                        embed_model: Some(config_clone.embed_model.clone()),
                        embed_dimension: Some(config_clone.embed_dimension),
                        use_local_storage: local,
                        ..Default::default()
                    };

                    let rt = tokio::runtime::Handle::current();
                    rt.block_on(async {
                        match coderag_indexer::FullIndexer::new(index_config) {
                            Ok(indexer) => indexer.run().await,
                            Err(e) => Err(anyhow::anyhow!("Failed to create indexer: {}", e)),
                        }
                    })
                }).await;

                {
                    let mut s = status_ref.write();
                    match result {
                        Ok(Ok(stats)) => {
                            s.progress = Some(IndexProgress {
                                files_processed: stats.files_processed,
                                symbols_extracted: stats.symbols_extracted,
                                chunks_created: stats.chunks_created,
                            });
                            s.is_indexing = false;
                        }
                        Ok(Err(e)) => {
                            s.last_error = Some(e.to_string());
                            s.is_indexing = false;
                        }
                        Err(join_err) => {
                            s.last_error = Some(format!("Task join error: {}", join_err));
                            s.is_indexing = false;
                        }
                    }
                }

                let text = format!(
                    "Indexing completed for branch `{}` on `{}`.\nUse `index_status` for details.",
                    branch, repo_path
                );
                Ok(CallToolResult::success(vec![Content::text(text)]))
            })
        },
    )
}

fn index_status_tool() -> ToolRoute<CodeRagServer> {
    ToolRoute::new_dyn(
        Tool {
            name: "index_status".into(),
            title: None,
            description: Some("Check the status of a running indexing operation. Returns progress or completion status.".into()),
            input_schema: empty_schema(),
            output_schema: None,
            annotations: None,
            icons: None,
        },
        |ctx: ToolCallContext<'_, CodeRagServer>| {
            let status = ctx.service.indexing_status.read();
            let text = if status.is_indexing {
                if let Some(ref p) = status.progress {
                    format!(
                        "Indexing in progress...\nFiles: {}\nSymbols: {}\nChunks: {}",
                        p.files_processed, p.symbols_extracted, p.chunks_created
                    )
                } else {
                    "Indexing in progress... (initializing)".to_string()
                }
            } else if let Some(ref err) = status.last_error {
                format!("Indexing failed: {}", err)
            } else if let Some(ref p) = status.progress {
                format!(
                    "Indexing completed.\nFiles: {}\nSymbols: {}\nChunks: {}",
                    p.files_processed, p.symbols_extracted, p.chunks_created
                )
            } else {
                "No indexing has been run yet.".to_string()
            };

            Box::pin(std::future::ready(Ok(CallToolResult::success(vec![Content::text(text)]))))
        },
    )
}

fn repo_info_tool() -> ToolRoute<CodeRagServer> {
    ToolRoute::new_dyn(
        Tool {
            name: "repo_info".into(),
            title: None,
            description: Some(
                "Get repository information including current branch, HEAD commit, and file count."
                    .into(),
            ),
            input_schema: empty_schema(),
            output_schema: None,
            annotations: None,
            icons: None,
        },
        |ctx: ToolCallContext<'_, CodeRagServer>| {
            let repo_path = ctx.service.config.repo_path.clone();
            Box::pin(async move {
                let result = tokio::task::spawn_blocking(move || {
                    let repo = coderag_core::repo::GitRepo::open(&repo_path)
                        .map_err(|e| err_internal(format!("Failed to open repo: {}", e)))?;

                    let head = repo.head_commit()
                        .map_err(|e| err_internal(format!("Failed to get HEAD: {}", e)))?;

                    let tree = repo.commit_to_tree(head.oid.inner())
                        .map_err(|e| err_internal(format!("Failed to read tree: {}", e)))?;

                    let files = repo.walk_tree(&tree)
                        .map_err(|e| err_internal(format!("Failed to walk tree: {}", e)))?;

                    let branch = repo.current_branch().unwrap_or_else(|_| "unknown".into());

                    Ok::<_, McpError>(format!(
                        "## Repository Info\n\n**Path:** {}\n**Branch:** {}\n**HEAD:** {}\n**Commit:** {}\n**Files:** {}\n",
                        repo_path, branch, head.oid, head.message, files.len()
                    ))
                }).await;

                match result {
                    Ok(Ok(text)) => Ok(CallToolResult::success(vec![Content::text(text)])),
                    Ok(Err(e)) => Err(e),
                    Err(join_err) => Err(err_internal(format!("Task join error: {}", join_err))),
                }
            })
        },
    )
}

fn repo_tree_tool() -> ToolRoute<CodeRagServer> {
    ToolRoute::new_dyn(
        Tool {
            name: "repo_tree".into(),
            title: None,
            description: Some(
                "List the file tree of the repository. Returns all tracked file paths.".into(),
            ),
            input_schema: empty_schema(),
            output_schema: None,
            annotations: None,
            icons: None,
        },
        |ctx: ToolCallContext<'_, CodeRagServer>| {
            let repo_path = ctx.service.config.repo_path.clone();
            Box::pin(async move {
                let result = tokio::task::spawn_blocking(move || {
                    let repo = coderag_core::repo::GitRepo::open(&repo_path)
                        .map_err(|e| err_internal(format!("Failed to open repo: {}", e)))?;

                    let head = repo
                        .head_commit()
                        .map_err(|e| err_internal(format!("Failed to get HEAD: {}", e)))?;

                    let tree = repo
                        .commit_to_tree(head.oid.inner())
                        .map_err(|e| err_internal(format!("Failed to read tree: {}", e)))?;

                    let files = repo
                        .walk_tree(&tree)
                        .map_err(|e| err_internal(format!("Failed to walk tree: {}", e)))?;

                    let mut text = format!("## File Tree ({} files)\n\n", files.len());
                    for file in files.iter().take(100) {
                        text.push_str(&format!("- `{}`\n", file.path));
                    }
                    if files.len() > 100 {
                        text.push_str(&format!("\n... and {} more files\n", files.len() - 100));
                    }

                    Ok::<_, McpError>(text)
                })
                .await;

                match result {
                    Ok(Ok(text)) => Ok(CallToolResult::success(vec![Content::text(text)])),
                    Ok(Err(e)) => Err(e),
                    Err(join_err) => Err(err_internal(format!("Task join error: {}", join_err))),
                }
            })
        },
    )
}

fn list_languages_tool() -> ToolRoute<CodeRagServer> {
    ToolRoute::new_dyn(
        Tool {
            name: "list_languages".into(),
            title: None,
            description: Some(
                "List all supported programming languages for AST-based symbol extraction.".into(),
            ),
            input_schema: empty_schema(),
            output_schema: None,
            annotations: None,
            icons: None,
        },
        |_| {
            let languages: [(&str, [&str; 3]); 20] = [
                ("Rust", [".rs", "", ""]),
                ("Python", [".py", ".pyw", ""]),
                ("JavaScript", [".js", ".jsx", ".mjs"]),
                ("TypeScript", [".ts", ".tsx", ".mts"]),
                ("Java", [".java", "", ""]),
                ("Go", [".go", "", ""]),
                ("C", [".c", ".h", ""]),
                ("C++", [".cpp", ".cc", ".hpp"]),
                ("C#", [".cs", "", ""]),
                ("Swift", [".swift", "", ""]),
                ("Kotlin", [".kt", ".kts", ""]),
                ("PHP", [".php", "", ""]),
                ("Ruby", [".rb", "", ""]),
                ("Shell", [".sh", ".bash", ""]),
                ("Scala", [".scala", "", ""]),
                ("Dart", [".dart", "", ""]),
                ("Lua", [".lua", "", ""]),
                ("R", [".r", ".R", ""]),
                ("Perl", [".pl", ".pm", ""]),
                ("SQL", [".sql", "", ""]),
            ];

            let mut text = String::from("## Supported Languages\n\n");
            for (name, exts) in &languages {
                let exts: Vec<&str> = exts.iter().filter(|e| !e.is_empty()).map(|&e| e).collect();
                text.push_str(&format!("- **{}**: {}\n", name, exts.join(", ")));
            }

            Box::pin(std::future::ready(Ok(CallToolResult::success(vec![
                Content::text(text),
            ]))))
        },
    )
}

// ─── Server implementation ──────────────────────────────────────────

impl ServerHandler for CodeRagServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_03_26,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "coderag-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                title: Some("CodeRAG MCP Server".to_string()),
                website_url: None,
                icons: None,
            },
            instructions: Some(
                "CodeRAG MCP server provides semantic code search using AST-based symbol extraction. \
                Use `search` for semantic queries, `search_graph` for relationship visualization, \
                `index` to build the vector index, and `repo_info`/`repo_tree` for repository metadata."
                    .to_string(),
            ),
        }
    }

    fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = std::result::Result<ListToolsResult, McpError>> + Send + '_
    {
        let router = self.tool_router();
        let tools = router.list_all();
        std::future::ready(Ok(ListToolsResult::with_all_items(tools)))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> std::result::Result<CallToolResult, McpError> {
        let ctx = ToolCallContext::new(self, request, context);
        let router = self.tool_router();
        router.call(ctx).await
    }
}

impl CodeRagServer {
    pub fn new(config: McpConfig) -> Self {
        Self {
            config: Arc::new(config),
            indexing_status: Arc::new(RwLock::new(IndexingStatus {
                is_indexing: false,
                progress: None,
                last_error: None,
            })),
        }
    }

    fn tool_router(&self) -> ToolRouter<Self> {
        let mut router = ToolRouter::new();
        router.add_route(search_tool());
        router.add_route(search_graph_tool());
        router.add_route(index_tool());
        router.add_route(index_status_tool());
        router.add_route(repo_info_tool());
        router.add_route(repo_tree_tool());
        router.add_route(list_languages_tool());
        router
    }
}
