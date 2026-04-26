//! Search command handler

use crate::config::Config;
use anyhow::{Context, Result as AnyResult};
use coderag_storage::{SearchResult, StorageBackend, StorageConfig, QdrantConfig, SearchFilter, SearchOptions};
use serde::Serialize;

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
    val: u32,
}

#[derive(Serialize)]
struct GraphLink {
    source: String,
    target: String,
    relation: String,
}

pub async fn search_code(
    repo_path: &str,
    query: &str,
    language: Option<&str>,
    limit: usize,
    use_local: bool,
    graph: bool,
    config: &Config,
) -> AnyResult<()> {
    if !graph {
        println!("Searching for: {}", query);
    }

    let storage = if use_local {
        let store = StorageBackend::from_config(&StorageConfig::Local {
            store_path: repo_path.to_string(),
            is_bare: false,
        })?;
        store
    } else {
        let qdrant_url = config.qdrant_url.clone()
            .unwrap_or_else(|| "http://localhost:6333".to_string());

        let storage_config = QdrantConfig {
            url: qdrant_url.clone(),
            api_key: config.qdrant_api_key.clone(),
            collection_name: config.collection_name.clone()
                .unwrap_or_else(|| "coderag".to_string()),
            ..Default::default()
        };

        let store = StorageBackend::from_config(&StorageConfig::Qdrant(storage_config))?;

        if !store.health_check().await {
            if !graph {
                println!("\nError: Cannot connect to Qdrant at {}", qdrant_url);
                println!("Make sure Qdrant is running (docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant)");
                println!("Or use --local to search with local storage: coderag search --local \"query\"");
            } else {
                eprintln!("Error: Cannot connect to Qdrant at {}", qdrant_url);
            }
            return Ok(());
        }

        store
    };

    let openai_api_key = config.openai_api_key.clone();

    let embedder_config = coderag_core::embedder::EmbedderConfig {
        api_url: config.embed_url.clone()
            .unwrap_or_else(|| "https://dashscope.aliyuncs.com/compatible-mode/v1/embeddings".to_string()),
        model: config.embed_model.clone()
            .unwrap_or_else(|| "text-embedding-v4".to_string()),
        dimension: config.embed_dimension.unwrap_or(1024),
        api_key: openai_api_key,
        ..Default::default()
    };

    let embedder = coderag_core::embedder::Embedder::new(embedder_config)
        .context("Failed to create embedder")?;

    let query_chunk = coderag_core::chunker::Chunk {
        id: "query".to_string(),
        content_hash: "".to_string(),
        repo: repo_path.to_string(),
        branch: "main".to_string(),
        commit: "".to_string(),
        language: language.unwrap_or("").to_string(),
        file: "".to_string(),
        module: "".to_string(),
        symbol: "".to_string(),
        kind: "query".to_string(),
        signature: query.to_string(),
        doc: None,
        code: query.to_string(),
        start_line: 0,
        end_line: 0,
    };

    let embedding = match embedder.embed_chunk(&query_chunk).await {
        Ok(e) => e,
        Err(e) => {
            if !graph {
                println!("\nError: Failed to generate embedding: {}", e);
                println!("Check your OPENAI_API_KEY environment variable.");
            } else {
                eprintln!("Error: Failed to generate embedding: {}", e);
            }
            return Ok(());
        }
    };

    let filter = SearchFilter {
        language: language.map(String::from),
        ..Default::default()
    };

    let options = SearchOptions {
        limit,
        score_threshold: Some(0.5),
        filter: if filter.language.is_some() || filter.file.is_some() || filter.kind.is_some() {
            Some(filter)
        } else {
            None
        },
        ..Default::default()
    };

    let results = storage.search(&embedding.vector, options).await?;

    if results.is_empty() {
        if !graph {
            if use_local {
                println!("\nNo results found in local storage. Try indexing the repository first:");
                println!("  coderag index --repo {} --local --full", repo_path);
            } else {
                println!("\nNo results found. Try indexing the repository first:");
                println!("  coderag index --repo {} --full", repo_path);
            }
        }
        return Ok(());
    }

    if graph {
        output_graph_json(&results);
    } else {
        output_text_results(&results);
    }

    Ok(())
}

fn output_text_results(results: &[SearchResult]) {
    println!("\nFound {} results:\n", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("{}. [{}] {:.2}", i + 1, result.payload.kind, result.score);
        println!("   Symbol: {}", result.payload.symbol);
        println!("   File: {}", result.payload.file);
        if let Some(ref doc) = result.payload.doc {
            if !doc.is_empty() {
                let doc_short = if doc.len() > 100 { &doc[..100] } else { doc };
                println!("   Doc: {}", doc_short);
            }
        }
        if !result.payload.signature.is_empty() && result.payload.signature != result.payload.symbol {
            println!("   Signature: {}", result.payload.signature);
        }
        println!("   Code: {}", result.payload.code.lines().next().unwrap_or(""));
        println!();
    }
}

fn output_graph_json(results: &[SearchResult]) {
    let nodes: Vec<GraphNode> = results.iter().map(|r| GraphNode {
        id: r.id.clone(),
        name: r.payload.symbol.clone(),
        kind: r.payload.kind.clone(),
        file: r.payload.file.clone(),
        module: r.payload.module.clone(),
        language: r.payload.language.clone(),
        score: r.score,
        code: r.payload.code.clone(),
        val: (r.score * 20.0).max(3.0).min(20.0) as u32,
    }).collect();

    let mut links: Vec<GraphLink> = Vec::new();
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            let a = &nodes[i];
            let b = &nodes[j];
            if a.file == b.file && !a.file.is_empty() {
                links.push(GraphLink { source: a.id.clone(), target: b.id.clone(), relation: "same_file".into() });
            }
            if a.module == b.module && !a.module.is_empty() {
                links.push(GraphLink { source: a.id.clone(), target: b.id.clone(), relation: "same_module".into() });
            }
        }
    }

    let output = serde_json::json!({
        "nodes": nodes,
        "links": links,
        "total": nodes.len(),
    });

    println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
}
