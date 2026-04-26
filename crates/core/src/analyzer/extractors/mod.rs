//! Language-specific symbol extractors

mod c_cpp;
mod csharp_swift;
pub mod doc_helpers;
mod go;
mod java_js_php;
pub mod node_helpers;
mod python;
mod rust;

pub use c_cpp::{extract_c, extract_cpp};
pub use csharp_swift::{extract_csharp, extract_swift};
pub use go::extract_go;
pub use java_js_php::{extract_bash, extract_java, extract_javascript, extract_php, extract_ruby};
pub use node_helpers::*;
pub use python::extract_python;
pub use rust::extract_rust;
