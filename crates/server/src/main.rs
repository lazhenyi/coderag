use actix_cors::Cors;
use actix_web::{web, App, HttpServer, HttpResponse, middleware};
use coderag_core::embedder::{Embedder, EmbedderConfig};
use coderag_core::repo::GitRepo;
use coderag_indexer::{FullIndexer, IndexConfig};
use coderag_storage::{StorageBackend, StorageConfig, QdrantConfig, SearchFilter, SearchOptions};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

mod frontend;
mod frontend_serve;

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
struct AppState {
    config: Arc<AppConfig>,
    indexing_status: Arc<RwLock<IndexingStatus>>,
}

#[derive(Clone)]
struct AppConfig {
    repo_path: String,
    embed_url: String,
    embed_model: String,
    embed_dimension: usize,
    embed_api_key: Option<String>,
    qdrant_url: String,
    qdrant_api_key: Option<String>,
    collection_name: String,
    #[allow(dead_code)]
    state_file: String,
    batch_size: usize,
}

#[derive(Clone, Serialize, Deserialize)]
struct IndexingStatus {
    is_indexing: bool,
    progress: Option<IndexProgress>,
    last_error: Option<String>,
    started_at: Option<u64>,
}

#[derive(Clone, Serialize, Deserialize)]
struct IndexProgress {
    files_processed: usize,
    symbols_extracted: usize,
    chunks_created: usize,
}

#[derive(Deserialize)]
struct SearchRequest {
    query: String,
    #[serde(default)]
    language: Option<String>,
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default = "default_use_local")]
    local: bool,
    #[serde(default = "default_score_threshold")]
    score_threshold: f32,
}

fn default_limit() -> usize { 10 }
fn default_use_local() -> bool { true }
fn default_score_threshold() -> f32 { 0.5 }

#[derive(Deserialize)]
struct IndexRequest {
    #[serde(default)]
    #[allow(dead_code)]
    full: bool,
    #[serde(default = "default_branch")]
    branch: String,
    #[serde(default = "default_use_local")]
    local: bool,
}

fn default_branch() -> String { "main".into() }

#[derive(Serialize)]
struct SearchResult {
    id: String,
    score: f32,
    symbol: String,
    file: String,
    language: String,
    kind: String,
    signature: String,
    doc: Option<String>,
    code: String,
    start_line: usize,
    end_line: usize,
}

#[derive(Serialize)]
struct RepoInfo {
    path: String,
    branch: String,
    head_oid: String,
    head_message: String,
    file_count: usize,
}

#[derive(Serialize)]
struct FileTreeNode {
    path: String,
    is_dir: bool,
}

#[derive(Serialize)]
struct LanguageInfo {
    name: String,
    extensions: Vec<String>,
}

async fn search(
    req: web::Json<SearchRequest>,
    state: web::Data<AppState>,
) -> HttpResponse {
    let storage = match create_storage(&state.config, req.local).await {
        Ok(s) => s,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let embedder = match create_embedder(&state.config) {
        Ok(e) => e,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    // Create a dummy chunk for embedding the query
    let query_chunk = coderag_core::chunker::Chunk {
        id: "query".into(),
        content_hash: "".into(),
        repo: state.config.repo_path.clone(),
        branch: "main".into(),
        commit: "".into(),
        language: req.language.clone().unwrap_or_default(),
        file: "".into(),
        module: "".into(),
        symbol: "".into(),
        kind: "query".into(),
        signature: req.query.clone(),
        doc: None,
        code: req.query.clone(),
        start_line: 0,
        end_line: 0,
    };

    let embedding = match embedder.embed_chunk(&query_chunk).await {
        Ok(e) => e,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("Embedding failed: {}", e)})),
    };

    let filter = SearchFilter {
        language: req.language.clone(),
        ..Default::default()
    };

    let options = SearchOptions {
        limit: req.limit,
        score_threshold: Some(req.score_threshold),
        filter: if filter.language.is_some() || filter.file.is_some() || filter.kind.is_some() {
            Some(filter)
        } else {
            None
        },
        ..Default::default()
    };

    match storage.search(&embedding.vector, options).await {
        Ok(results) => {
            let items: Vec<SearchResult> = results.iter().map(|r| SearchResult {
                id: r.id.clone(),
                score: r.score,
                symbol: r.payload.symbol.clone(),
                file: r.payload.file.clone(),
                language: r.payload.language.clone(),
                kind: r.payload.kind.clone(),
                signature: r.payload.signature.clone(),
                doc: r.payload.doc.clone(),
                code: r.payload.code.clone(),
                start_line: r.payload.start_line,
                end_line: r.payload.end_line,
            }).collect();

            HttpResponse::Ok().json(serde_json::json!({
                "results": items,
                "total": items.len(),
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

async fn index(
    req: web::Json<IndexRequest>,
    state: web::Data<AppState>,
) -> HttpResponse {
    let status = state.indexing_status.read();
    if status.is_indexing {
        return HttpResponse::Conflict().json(serde_json::json!({"error": "Indexing already in progress"}));
    }
    drop(status);

    let config = state.config.clone();
    let status_ref = state.indexing_status.clone();
    let req = req.into_inner();

    tokio::task::spawn_blocking(move || {
        {
            let mut status = status_ref.write();
            status.is_indexing = true;
            status.last_error = None;
            status.started_at = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
            status.progress = None;
        }

        let index_config = IndexConfig {
            repo_path: config.repo_path.clone(),
            branch: req.branch.clone(),
            qdrant_url: config.qdrant_url.clone(),
            qdrant_api_key: config.qdrant_api_key.clone(),
            collection_name: config.collection_name.clone(),
            batch_size: config.batch_size,
            embed_api_url: Some(config.embed_url.clone()),
            embed_api_key: config.embed_api_key.clone(),
            embed_model: Some(config.embed_model.clone()),
            embed_dimension: Some(config.embed_dimension),
            use_local_storage: req.local,
            ..Default::default()
        };

        let rt = tokio::runtime::Handle::current();
        let result = match FullIndexer::new(index_config) {
            Ok(indexer) => rt.block_on(indexer.run()),
            Err(e) => Err(anyhow::anyhow!("Failed to create indexer: {}", e)),
        };

        let mut status = status_ref.write();
        match result {
            Ok(stats) => {
                status.progress = Some(IndexProgress {
                    files_processed: stats.files_processed,
                    symbols_extracted: stats.symbols_extracted,
                    chunks_created: stats.chunks_created,
                });
                status.is_indexing = false;
            }
            Err(e) => {
                status.last_error = Some(e.to_string());
                status.is_indexing = false;
            }
        }
    });

    HttpResponse::Accepted().json(serde_json::json!({"message": "Indexing started"}))
}

async fn index_status(state: web::Data<AppState>) -> HttpResponse {
    let status = state.indexing_status.read();
    HttpResponse::Ok().json(&*status)
}

async fn repo_info(state: web::Data<AppState>) -> HttpResponse {
    match GitRepo::open(&state.config.repo_path) {
        Ok(repo) => {
            match repo.head_commit() {
                Ok(head) => {
                    let tree = match repo.commit_to_tree(head.oid.inner()) {
                        Ok(t) => t,
                        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to read tree"})),
                    };
                    let files = match repo.walk_tree(&tree) {
                        Ok(f) => f,
                        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to walk tree"})),
                    };
                    let branch = match repo.current_branch() {
                        Ok(b) => b,
                        Err(_) => "unknown".into(),
                    };

                    HttpResponse::Ok().json(RepoInfo {
                        path: state.config.repo_path.clone(),
                        branch,
                        head_oid: head.oid.to_string(),
                        head_message: head.message,
                        file_count: files.len(),
                    })
                }
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("Failed to get HEAD: {}", e)})),
            }
        }
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": format!("Failed to open repo: {}", e)})),
    }
}

async fn repo_tree(state: web::Data<AppState>) -> HttpResponse {
    match GitRepo::open(&state.config.repo_path) {
        Ok(repo) => {
            match repo.head_commit() {
                Ok(head) => {
                    let tree = match repo.commit_to_tree(head.oid.inner()) {
                        Ok(t) => t,
                        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to read tree"})),
                    };
                    let files = match repo.walk_tree(&tree) {
                        Ok(f) => f,
                        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to walk tree"})),
                    };
                    let nodes: Vec<FileTreeNode> = files.iter()
                        .map(|f| FileTreeNode {
                            path: f.path.clone(),
                            is_dir: false,
                        })
                        .collect();
                    HttpResponse::Ok().json(serde_json::json!({"files": nodes}))
                }
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("Failed to get HEAD: {}", e)})),
            }
        }
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": format!("Failed to open repo: {}", e)})),
    }
}

#[derive(Serialize)]
struct GraphNode {
    id: String,
    name: String,
    kind: String,
    file: String,
    module: String,
    language: String,
    score: f32,
    code: String,
    doc: Option<String>,
    signature: String,
    start_line: usize,
    end_line: usize,
    val: u32,
}

#[derive(Serialize)]
struct GraphLink {
    source: String,
    target: String,
    relation: String,
}

#[derive(Deserialize)]
struct GraphSearchRequest {
    query: String,
    #[serde(default)]
    language: Option<String>,
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default = "default_use_local")]
    local: bool,
    #[serde(default = "default_score_threshold")]
    score_threshold: f32,
}

async fn search_graph(
    req: web::Json<GraphSearchRequest>,
    state: web::Data<AppState>,
) -> HttpResponse {
    let storage = match create_storage(&state.config, req.local).await {
        Ok(s) => s,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let embedder = match create_embedder(&state.config) {
        Ok(e) => e,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let query_chunk = coderag_core::chunker::Chunk {
        id: "query".into(),
        content_hash: "".into(),
        repo: state.config.repo_path.clone(),
        branch: "main".into(),
        commit: "".into(),
        language: req.language.clone().unwrap_or_default(),
        file: "".into(),
        module: "".into(),
        symbol: "".into(),
        kind: "query".into(),
        signature: req.query.clone(),
        doc: None,
        code: req.query.clone(),
        start_line: 0,
        end_line: 0,
    };

    let embedding = match embedder.embed_chunk(&query_chunk).await {
        Ok(e) => e,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("Embedding failed: {}", e)})),
    };

    let filter = SearchFilter {
        language: req.language.clone(),
        ..Default::default()
    };

    let options = SearchOptions {
        limit: req.limit,
        score_threshold: Some(req.score_threshold),
        filter: if filter.language.is_some() || filter.file.is_some() || filter.kind.is_some() {
            Some(filter)
        } else {
            None
        },
        ..Default::default()
    };

    let results = match storage.search(&embedding.vector, options).await {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let nodes: Vec<GraphNode> = results.iter().map(|r| GraphNode {
        id: r.id.clone(),
        name: r.payload.symbol.clone(),
        kind: r.payload.kind.clone(),
        file: r.payload.file.clone(),
        module: r.payload.module.clone(),
        language: r.payload.language.clone(),
        score: r.score,
        code: r.payload.code.clone(),
        doc: r.payload.doc.clone(),
        signature: r.payload.signature.clone(),
        start_line: r.payload.start_line,
        end_line: r.payload.end_line,
        val: (r.score * 20.0).max(3.0).min(20.0) as u32,
    }).collect();

    let mut links: Vec<GraphLink> = Vec::new();
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            let a = &nodes[i];
            let b = &nodes[j];
            // Same file relationship
            if a.file == b.file && !a.file.is_empty() {
                links.push(GraphLink { source: a.id.clone(), target: b.id.clone(), relation: "same_file".into() });
            }
            // Same module relationship
            if a.module == b.module && !a.module.is_empty() {
                links.push(GraphLink { source: a.id.clone(), target: b.id.clone(), relation: "same_module".into() });
            }
            // Same language
            if a.language == b.language && !a.language.is_empty() && a.language != b.language {
                // already same language, skip
            }
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "nodes": nodes,
        "links": links,
        "total": nodes.len(),
    }))
}

async fn languages() -> HttpResponse {
    let langs = vec![
        LanguageInfo { name: "Rust".into(), extensions: vec![".rs".into()] },
        LanguageInfo { name: "Python".into(), extensions: vec![".py".into(), ".pyw".into()] },
        LanguageInfo { name: "JavaScript".into(), extensions: vec![".js".into(), ".jsx".into(), ".mjs".into()] },
        LanguageInfo { name: "TypeScript".into(), extensions: vec![".ts".into(), ".tsx".into(), ".mts".into()] },
        LanguageInfo { name: "Java".into(), extensions: vec![".java".into()] },
        LanguageInfo { name: "Go".into(), extensions: vec![".go".into()] },
        LanguageInfo { name: "C".into(), extensions: vec![".c".into(), ".h".into()] },
        LanguageInfo { name: "C++".into(), extensions: vec![".cpp".into(), ".cc".into(), ".hpp".into()] },
        LanguageInfo { name: "C#".into(), extensions: vec![".cs".into()] },
        LanguageInfo { name: "Swift".into(), extensions: vec![".swift".into()] },
        LanguageInfo { name: "Kotlin".into(), extensions: vec![".kt".into(), ".kts".into()] },
        LanguageInfo { name: "PHP".into(), extensions: vec![".php".into()] },
        LanguageInfo { name: "Ruby".into(), extensions: vec![".rb".into()] },
        LanguageInfo { name: "Shell".into(), extensions: vec![".sh".into(), ".bash".into()] },
        LanguageInfo { name: "Scala".into(), extensions: vec![".scala".into()] },
        LanguageInfo { name: "Dart".into(), extensions: vec![".dart".into()] },
        LanguageInfo { name: "Lua".into(), extensions: vec![".lua".into()] },
        LanguageInfo { name: "R".into(), extensions: vec![".r".into(), ".R".into()] },
        LanguageInfo { name: "Perl".into(), extensions: vec![".pl".into(), ".pm".into()] },
        LanguageInfo { name: "SQL".into(), extensions: vec![".sql".into()] },
    ];
    HttpResponse::Ok().json(serde_json::json!({"languages": langs}))
}

async fn create_storage(config: &AppConfig, use_local: bool) -> anyhow::Result<StorageBackend> {
    if use_local {
        StorageBackend::from_config(&StorageConfig::Local {
            store_path: config.repo_path.clone(),
            is_bare: false,
        })
    } else {
        let storage_config = QdrantConfig {
            url: config.qdrant_url.clone(),
            api_key: config.qdrant_api_key.clone(),
            collection_name: config.collection_name.clone(),
            ..Default::default()
        };
        StorageBackend::from_config(&StorageConfig::Qdrant(storage_config))
    }
}

fn create_embedder(config: &AppConfig) -> anyhow::Result<Embedder> {
    let embedder_config = EmbedderConfig {
        api_url: config.embed_url.clone(),
        model: config.embed_model.clone(),
        dimension: config.embed_dimension,
        api_key: config.embed_api_key.clone(),
        ..Default::default()
    };
    Embedder::new(embedder_config)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    dotenvy::dotenv().ok();

    let repo_path = std::env::var("CODERAG_REPO").unwrap_or_else(|_| ".".into());
    let embed_url = std::env::var("CODERAG_EMBED_URL")
        .unwrap_or_else(|_| "https://dashscope.aliyuncs.com/compatible-mode/v1/embeddings".into());
    let embed_model = std::env::var("CODERAG_EMBED_MODEL")
        .unwrap_or_else(|_| "text-embedding-v4".into());
    let embed_dimension: usize = std::env::var("CODERAG_EMBED_DIMENSION")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1024);
    let embed_api_key = std::env::var("OPENAI_API_KEY").ok();
    let qdrant_url = std::env::var("QDRANT_URL")
        .unwrap_or_else(|_| "http://localhost:6333".into());
    let qdrant_api_key = std::env::var("QDRANT_API_KEY").ok();
    let collection_name = std::env::var("CODERAG_COLLECTION")
        .unwrap_or_else(|_| "coderag".into());
    let state_file = std::env::var("CODERAG_STATE_FILE")
        .unwrap_or_else(|_| ".coderag/state.json".into());
    let batch_size: usize = std::env::var("CODERAG_BATCH_SIZE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);

    let app_state = AppState {
        config: Arc::new(AppConfig {
            repo_path,
            embed_url,
            embed_model,
            embed_dimension,
            embed_api_key,
            qdrant_url,
            qdrant_api_key,
            collection_name,
            state_file,
            batch_size,
        }),
        indexing_status: Arc::new(RwLock::new(IndexingStatus {
            is_indexing: false,
            progress: None,
            last_error: None,
            started_at: None,
        })),
    };

    let bind_addr = std::env::var("CODERAG_BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());

    println!("CodeRAG Server starting on {}", bind_addr);
    println!("  Repository: {}", app_state.config.repo_path);
    println!("  Embed model: {}", app_state.config.embed_model);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(app_state.clone()))
            .route("/api/search", web::post().to(search))
            .route("/api/search/graph", web::post().to(search_graph))
            .route("/api/index", web::post().to(index))
            .route("/api/index/status", web::get().to(index_status))
            .route("/api/repo/info", web::get().to(repo_info))
            .route("/api/repo/tree", web::get().to(repo_tree))
            .route("/api/languages", web::get().to(languages))
            .route("/health", web::get().to(|| async {
                HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
            }))
            .route("/{path:.*}", web::get().to(frontend_serve::serve_frontend))
    })
    .bind(&bind_addr)?
    .run()
    .await
}
