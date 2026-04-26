//! Language Definitions
//!
//! Provides tree-sitter language configurations for supported languages.

mod lang_enum;
mod configs_tier1;
mod configs_tier2;

pub use lang_enum::Language;
pub use configs_tier1::*;
pub use configs_tier2::*;
