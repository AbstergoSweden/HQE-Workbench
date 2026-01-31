//! HQE Core - Scan pipeline, redaction, and data models
//!
//! This crate provides the foundational types and logic for the HQE Workbench.
//!
//! # Modules
//!
//! - [`models`] - Core data models for scans, findings, and reports
//! - [`redaction`] - PII and secret redaction utilities
//! - [`repo`] - Repository scanning and analysis
//! - [`scan`] - The main scan pipeline

#![warn(missing_docs)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

pub mod models;
pub mod persistence;
pub mod redaction;
pub mod repo;
pub mod scan;

pub use models::*;
pub use persistence::*;
pub use redaction::*;
pub use repo::*;
pub use scan::*;

use thiserror::Error;

/// Core error type for the HQE system
///
/// This enum represents all possible errors that can occur during
/// HQE operations, from IO errors to LLM provider failures.
#[derive(Error, Debug)]
pub enum HqeError {
    /// IO operation failed
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON or other serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Error during repository scanning
    #[error("Scan error: {0}")]
    Scan(String),

    /// Error during content redaction
    #[error("Redaction error: {0}")]
    Redaction(String),

    /// Invalid configuration provided
    #[error("Invalid configuration: {0}")]
    Config(String),

    /// Git operation failed
    #[error("Git operation failed: {0}")]
    Git(String),

    /// LLM provider API error
    #[error("LLM provider error: {0}")]
    Provider(String),

    /// Report or manifest generation failed
    #[error("Artifact generation failed: {0}")]
    Artifacts(String),
}

/// Result type alias using [`HqeError`]
pub type Result<T> = std::result::Result<T, HqeError>;
