//! Model Context Protocol (MCP) Integration
//!
//! Provides tools for loading and managing MCP servers and tools.

#![warn(missing_docs)]

/// File-based prompt loader
pub mod loader;
/// Server registry and tool management
pub mod registry;
/// Enhanced prompt registry (v2)
pub mod registry_v2;

pub use loader::*;
pub use registry::*;
pub use registry_v2::*;

/// Initialize the MCP subsystem
pub fn init() {
    println!("hqe-mcp initialized");
}
