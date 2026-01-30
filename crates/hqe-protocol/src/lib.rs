//! HQE Protocol - Core types and definitions for the HQE Workbench
//!
//! This crate provides the shared protocol definitions used across all HQE crates.
//! It includes models for entities, topics, provider profiles, and workflows.
//!
//! # Core Types
//!
//! - [`Entity`] - The polymorphic data storage unit for any topic
//! - [`ProviderProfile`] - Configuration for LLM providers
//! - [`TopicManifest`] - Definition of topic capabilities and schemas
//! - [`TopicCapabilities`] - Tools, prompts, and workflows provided by a topic
//!
//! # Provider Support
//!
//! The protocol supports multiple LLM providers through the [`ProviderKind`] enum:
//! - OpenAI
//! - Venice
//! - OpenRouter
//! - XAI
//! - Generic (OpenAI-compatible)
//!
//! # Example
//!
//! ```rust
//! use hqe_protocol::models::{ProviderProfile, ProviderKind, Entity};
//! use serde_json::json;
//!
//! // Create a provider profile
//! let profile = ProviderProfile::new("my-provider", "https://api.example.com/v1")
//!     .with_model("gpt-4")
//!     .with_provider_kind(ProviderKind::Generic);
//!
//! // Create an entity
//! let entity = Entity {
//!     id: "uuid-123".to_string(),
//!     topic_id: "my-topic".to_string(),
//!     kind: "MyData".to_string(),
//!     data: json!({"key": "value"}),
//!     vector_embedding: None,
//!     created_at: chrono::Utc::now(),
//!     updated_at: chrono::Utc::now(),
//! };
//! ```

#![warn(missing_docs)]

/// Protocol models module
pub mod models;

pub use models::*;

/// Initialize the protocol crate
///
/// This function is primarily used for testing and debugging purposes.
/// It prints a message indicating the protocol crate has been initialized.
pub fn init() {
    println!("hqe-protocol initialized");
}
