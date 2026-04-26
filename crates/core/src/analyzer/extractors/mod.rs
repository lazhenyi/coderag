//! Language-specific symbol extractors

pub mod doc_helpers;
pub mod node_helpers;
mod rust;
mod python;
mod go;
mod c_cpp;
mod csharp_swift;
mod java_js_php;

pub use rust::extract_rust;
pub use python::extract_python;
pub use go::extract_go;
pub use c_cpp::{extract_c, extract_cpp};
pub use csharp_swift::{extract_csharp, extract_swift};
pub use java_js_php::{extract_javascript, extract_java, extract_php, extract_ruby, extract_bash};
pub use node_helpers::*;
