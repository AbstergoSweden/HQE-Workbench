//! Workflow Execution Engine
//!
//! Orchestrates the execution of multi-step workflows using MCP tools.

#![warn(missing_docs)]

/// The core execution engine
pub mod engine;

pub use engine::FlowEngine;

/// Initialize the flow subsystem
pub fn init() {
    println!("hqe-flow initialized");
}