//! Quality Evaluation Module

mod eval_impl;
mod types;

pub use eval_impl::{default_queries, evaluate, print_report};
pub use types::{EvalMetrics, EvalQuery, LanguageMetrics};
