//! Info, languages, and eval handlers

use crate::config::Config;
use anyhow::{Context, Result as AnyResult};
use coderag_core::repo::GitRepo;
use coderag_indexer::{default_queries, evaluate, print_report};
use coderag_storage::{QdrantClient, QdrantConfig};

pub fn show_info(repo_path: &str, show_tree: bool) -> AnyResult<()> {
    let repo = GitRepo::open(repo_path).context("Failed to open repository")?;
    let head = repo.head_commit()?;
    println!("Repository: {}", repo_path);
    println!("Current branch: {}", repo.current_branch()?);
    println!("HEAD: {} ({})", head.oid, head.message);

    if show_tree {
        let tree = repo.commit_to_tree(head.oid.inner())?;
        let files = repo.walk_tree(&tree)?;

        println!("\nFile tree ({} files):", files.len());
        for file in files.iter().take(50) {
            println!("  {}", file.path);
        }
        if files.len() > 50 {
            println!("  ... and {} more", files.len() - 50);
        }
    }

    Ok(())
}

pub fn show_languages() -> AnyResult<()> {
    println!("Supported languages (AST extraction):");
    println!();
    println!("  Rust           (.rs)              - function, struct, enum, trait, impl, module");
    println!("  Python         (.py, .pyw)        - function, class, method");
    println!("  JavaScript     (.js, .jsx, .mjs)  - function, class, method");
    println!("  TypeScript     (.ts, .tsx, .mts)  - function, class, interface, method");
    println!("  Java           (.java)            - class, interface, method, enum");
    println!("  Go             (.go)              - function, struct, interface, method");
    println!("  C              (.c, .h)           - function, struct, enum, union");
    println!("  C++            (.cpp, .cc, .hpp)  - function, class, struct, enum, namespace");
    println!("  C#             (.cs)              - class, interface, method, struct, enum");
    println!("  Swift          (.swift)           - function, class, struct, enum, protocol");
    println!("  Kotlin         (.kt, .kts)        - function, class, interface, enum, object");
    println!("  PHP            (.php)             - function, class, method, interface, trait");
    println!("  Ruby           (.rb)              - method, class, module");
    println!("  Shell/Bash     (.sh, .bash)       - function");
    println!("  Scala          (.scala)           - function, class, trait, object");
    println!("  Dart           (.dart)            - function, class, method, enum, mixin");
    println!("  Lua            (.lua)             - function");
    println!("  R              (.r, .R)           - function, class");
    println!("  Perl           (.pl, .pm)         - function, package, class");
    println!("  SQL            (.sql)             - table, function, procedure, view");

    Ok(())
}

pub async fn eval_quality(
    repo_path: &str,
    language: Option<&str>,
    qdrant_url: Option<&str>,
    config: &Config,
) -> AnyResult<()> {
    let qdrant_url = qdrant_url
        .map(String::from)
        .or_else(|| config.qdrant_url.clone())
        .unwrap_or_else(|| "http://localhost:6333".to_string());

    let openai_api_key = config.openai_api_key.clone();

    let storage_config = QdrantConfig {
        url: qdrant_url.clone(),
        api_key: config.qdrant_api_key.clone(),
        collection_name: config
            .collection_name
            .clone()
            .unwrap_or_else(|| "coderag".to_string()),
        ..Default::default()
    };

    let storage = QdrantClient::new(storage_config).context("Failed to create Qdrant client")?;

    if !storage.health_check().await {
        println!("Error: Cannot connect to Qdrant at {}", qdrant_url);
        println!("Make sure Qdrant is running:");
        println!("  docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant");
        return Ok(());
    }

    let embedder_config = coderag_core::embedder::EmbedderConfig {
        api_url: config.embed_url.clone().unwrap_or_else(|| {
            "https://dashscope.aliyuncs.com/compatible-mode/v1/embeddings".to_string()
        }),
        model: config
            .embed_model
            .clone()
            .unwrap_or_else(|| "text-embedding-v4".to_string()),
        dimension: config.embed_dimension.unwrap_or(1024),
        api_key: openai_api_key,
        ..Default::default()
    };

    let embedder = coderag_core::embedder::Embedder::new(embedder_config)
        .context("Failed to create embedder")?;

    let all_queries = default_queries();
    let queries: Vec<_> = if let Some(lang) = language {
        let lang_lower = lang.to_lowercase();
        all_queries
            .into_iter()
            .filter(|q| q.language.to_lowercase() == lang_lower)
            .collect()
    } else {
        all_queries
    };

    if queries.is_empty() {
        println!("No evaluation queries for language: {:?}", language);
        return Ok(());
    }

    println!("Running quality evaluation ({} queries)...", queries.len());

    let metrics = evaluate(&queries, &storage, &embedder).await?;
    print_report(&metrics);

    println!();
    println!("To improve results:");
    println!(
        "  1. Index your repository: coderag index --repo {} --full",
        repo_path
    );
    println!("  2. Use a real OpenAI API key (export OPENAI_API_KEY=...)");
    println!(
        "  3. Run: coderag eval --repo {} --language <lang>",
        repo_path
    );

    Ok(())
}
