//! Archive Handler for P2 Formats

mod extractors;
mod handler_impl;
mod tests;

pub use handler_impl::{ArchiveEntry, ArchiveHandler};
