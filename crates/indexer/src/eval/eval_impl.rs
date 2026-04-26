//! Evaluation implementation

use super::types::{EvalQuery, EvalResult, EvalMetrics, LanguageMetrics};
use anyhow::Result as AnyResult;
use coderag_core::chunker::Chunk;
use coderag_core::embedder::Embedder;
use coderag_storage::{QdrantClient, SearchOptions, SearchFilter};
use std::collections::HashMap;
use std::time::Instant;

/// List of evaluation queries for Top5 languages
pub fn default_queries() -> Vec<EvalQuery> {
    vec![
        EvalQuery {
            language: "rust".to_string(),
            query_text: "function that takes two integers and returns their sum".to_string(),
            expected_symbols: vec!["add".to_string()],
            expected_files: vec![],
        },
        EvalQuery {
            language: "rust".to_string(),
            query_text: "a struct representing a 2D point with coordinates".to_string(),
            expected_symbols: vec!["Point".to_string(), "Point2D".to_string()],
            expected_files: vec![],
        },
        EvalQuery {
            language: "rust".to_string(),
            query_text: "trait for sorting algorithm implementations".to_string(),
            expected_symbols: vec!["Sort".to_string(), "Sorter".to_string(), "Sortable".to_string()],
            expected_files: vec![],
        },
        EvalQuery {
            language: "python".to_string(),
            query_text: "class representing a HTTP request handler".to_string(),
            expected_symbols: vec!["RequestHandler".to_string(), "HttpHandler".to_string()],
            expected_files: vec![],
        },
        EvalQuery {
            language: "python".to_string(),
            query_text: "function that validates email addresses using regex".to_string(),
            expected_symbols: vec!["validate_email".to_string(), "is_valid_email".to_string()],
            expected_files: vec![],
        },
        EvalQuery {
            language: "python".to_string(),
            query_text: "decorator for logging function calls".to_string(),
            expected_symbols: vec!["log_calls".to_string(), "logged".to_string()],
            expected_files: vec![],
        },
        EvalQuery {
            language: "javascript".to_string(),
            query_text: "function that fetches data from an API endpoint".to_string(),
            expected_symbols: vec!["fetchData".to_string(), "getData".to_string(), "fetch".to_string()],
            expected_files: vec![],
        },
        EvalQuery {
            language: "javascript".to_string(),
            query_text: "React component for rendering a user profile card".to_string(),
            expected_symbols: vec!["UserProfile".to_string(), "ProfileCard".to_string()],
            expected_files: vec![],
        },
        EvalQuery {
            language: "java".to_string(),
            query_text: "interface for database CRUD operations".to_string(),
            expected_symbols: vec!["Repository".to_string(), "CrudRepository".to_string()],
            expected_files: vec![],
        },
        EvalQuery {
            language: "java".to_string(),
            query_text: "singleton configuration manager class".to_string(),
            expected_symbols: vec!["ConfigManager".to_string(), "Configuration".to_string()],
            expected_files: vec![],
        },
        EvalQuery {
            language: "go".to_string(),
            query_text: "struct representing an HTTP request with headers and body".to_string(),
            expected_symbols: vec!["Request".to_string(), "HttpRequest".to_string()],
            expected_files: vec![],
        },
        EvalQuery {
            language: "go".to_string(),
            query_text: "interface for data persistence layer".to_string(),
            expected_symbols: vec!["Store".to_string(), "Repository".to_string(), "Storage".to_string()],
            expected_files: vec![],
        },
    ]
}

/// Run quality evaluation against a Qdrant-backed index
pub async fn evaluate(
    queries: &[EvalQuery],
    storage: &QdrantClient,
    embedder: &Embedder,
) -> AnyResult<EvalMetrics> {
    let mut results = Vec::new();
    let start = Instant::now();

    for (i, query) in queries.iter().enumerate() {
        println!(
            "Evaluating [{}/{}] ({}) {}",
            i + 1,
            queries.len(),
            query.language,
            query.query_text
        );

        let query_chunk = Chunk {
            id: format!("eval-{}", i),
            content_hash: "".to_string(),
            repo: "eval".to_string(),
            branch: "main".to_string(),
            commit: "".to_string(),
            language: query.language.clone(),
            file: "".to_string(),
            module: "".to_string(),
            symbol: "".to_string(),
            kind: "query".to_string(),
            signature: query.query_text.clone(),
            doc: None,
            code: query.query_text.clone(),
            start_line: 0,
            end_line: 0,
        };

        let embedding = match embedder.embed_chunk(&query_chunk).await {
            Ok(e) => e,
            Err(e) => {
                eprintln!("  Embedding failed: {}", e);
                continue;
            }
        };

        let options = SearchOptions {
            limit: 5,
            filter: Some(SearchFilter {
                language: Some(query.language.clone()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let search_results = storage.search(&embedding.vector, options).await?;

        let top5_symbols: Vec<String> = search_results.iter().map(|r| r.payload.symbol.clone()).collect();
        let top1_hit = top5_symbols.first()
            .map(|s| query.expected_symbols.iter().any(|e| s.contains(e) || e.contains(s)))
            .unwrap_or(false);
        let top5_hit = top5_symbols.iter()
            .any(|s| query.expected_symbols.iter().any(|e| s.contains(e) || e.contains(s)));

        let reciprocal_rank = top5_symbols.iter()
            .position(|s| query.expected_symbols.iter().any(|e| s.contains(e) || e.contains(s)))
            .map(|pos| 1.0 / (pos as f64 + 1.0))
            .unwrap_or(0.0);

        results.push(EvalResult {
            language: query.language.clone(),
            query_text: query.query_text.clone(),
            expected_symbols: query.expected_symbols.clone(),
            top1_hit,
            top5_hit,
            top1_symbol: top5_symbols.first().cloned(),
            top5_symbols,
            reciprocal_rank,
        });
    }

    let duration = start.elapsed();
    println!("\nEvaluation completed in {:.2}s", duration.as_secs_f64());

    let total = results.len();
    let top1_count = results.iter().filter(|r| r.top1_hit).count();
    let top5_count = results.iter().filter(|r| r.top5_hit).count();
    let mrr_sum: f64 = results.iter().map(|r| r.reciprocal_rank).sum();

    let mut per_language: HashMap<String, Vec<&EvalResult>> = HashMap::new();
    for r in &results {
        per_language.entry(r.language.clone()).or_default().push(r);
    }

    let metrics = EvalMetrics {
        top1_accuracy: if total > 0 { top1_count as f64 / total as f64 } else { 0.0 },
        top5_accuracy: if total > 0 { top5_count as f64 / total as f64 } else { 0.0 },
        mrr: if total > 0 { mrr_sum / total as f64 } else { 0.0 },
        total_queries: total,
        per_language: per_language.iter().map(|(lang, r)| {
            let t = r.len();
            let t1 = r.iter().filter(|r| r.top1_hit).count();
            let t5 = r.iter().filter(|r| r.top5_hit).count();
            let mrr = r.iter().map(|r| r.reciprocal_rank).sum::<f64>() / t as f64;
            (lang.clone(), LanguageMetrics {
                top1_accuracy: t1 as f64 / t as f64,
                top5_accuracy: t5 as f64 / t as f64,
                mrr,
                total_queries: t,
            })
        }).collect(),
    };

    Ok(metrics)
}

/// Print evaluation results
pub fn print_report(metrics: &EvalMetrics) {
    println!("\n{}", "=".repeat(60));
    println!("  QUALITY EVALUATION REPORT");
    println!("{}", "=".repeat(60));
    println!();
    println!("  Overall Top1 Accuracy: {:.1}%", metrics.top1_accuracy * 100.0);
    println!("  Overall Top5 Accuracy: {:.1}%", metrics.top5_accuracy * 100.0);
    println!("  Mean Reciprocal Rank:  {:.3}", metrics.mrr);
    println!("  Total Queries:         {}", metrics.total_queries);
    println!();
    println!("{}", "-".repeat(60));
    println!("  Per-Language Results:");
    println!("{}", "-".repeat(60));
    for (lang, lm) in &metrics.per_language {
        println!(
            "  {:12} Top1: {:5.1}%  Top5: {:5.1}%  MRR: {:.3}  ({} queries)",
            lang,
            lm.top1_accuracy * 100.0,
            lm.top5_accuracy * 100.0,
            lm.mrr,
            lm.total_queries
        );
    }
    println!("{}", "=".repeat(60));
}
