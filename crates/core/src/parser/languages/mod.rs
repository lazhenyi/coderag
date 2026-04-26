//! Language Definitions
//!
//! Provides tree-sitter language configurations for supported languages.

mod configs_tier1;
mod configs_tier2;
mod lang_enum;

pub use configs_tier1::*;
pub use configs_tier2::*;
pub use lang_enum::Language;
