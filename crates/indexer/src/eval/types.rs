//! Evaluation types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single evaluation query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalQuery {
    pub language: String,
    pub query_text: String,
    pub expected_symbols: Vec<String>,
    pub expected_files: Vec<String>,
}

/// Evaluation result for a single query
#[derive(Debug, Clone)]
pub struct EvalResult {
    pub language: String,
    #[allow(dead_code)]
    pub query_text: String,
    #[allow(dead_code)]
    pub expected_symbols: Vec<String>,
    pub top1_hit: bool,
    pub top5_hit: bool,
    #[allow(dead_code)]
    pub top1_symbol: Option<String>,
    #[allow(dead_code)]
    pub top5_symbols: Vec<String>,
    pub reciprocal_rank: f64,
}

/// Aggregate evaluation metrics
#[derive(Debug, Clone, Default)]
pub struct EvalMetrics {
    pub top1_accuracy: f64,
    pub top5_accuracy: f64,
    pub mrr: f64,
    pub total_queries: usize,
    pub per_language: HashMap<String, LanguageMetrics>,
}

/// Per-language evaluation metrics
#[derive(Debug, Clone, Default)]
pub struct LanguageMetrics {
    pub top1_accuracy: f64,
    pub top5_accuracy: f64,
    pub mrr: f64,
    pub total_queries: usize,
}
