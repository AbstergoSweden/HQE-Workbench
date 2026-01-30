//! Model Context Protocol (MCP) Integration
//!
//! Provides tools for loading and managing MCP servers and tools.

#![warn(missing_docs)]

/// Server registry and tool management
pub mod registry;
/// File-based prompt loader
pub mod loader;

pub use registry::*;
pub use loader::*;

/// Initialize the MCP subsystem
pub fn init() {
    println!("hqe-mcp initialized");
}