//! Knowledge Ingestion Engine
//!
//! Handles loading of topic definitions and watching for file changes.

#![warn(missing_docs)]

/// Topic loader and parser
pub mod loader;
/// File watcher and event handling
pub mod watcher;

pub use loader::TopicLoader;
pub use watcher::{IngestEngine, IngestEvent};

/// Initialize the ingestion subsystem
pub fn init() {
    println!("hqe-ingest initialized");
}