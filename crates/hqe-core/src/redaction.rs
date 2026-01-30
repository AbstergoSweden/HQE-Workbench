//! Content redaction for secrets and sensitive data
//!
//! Implements pattern-based redaction before sending content to LLM providers.

use crate::models::RedactionSummary;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

/// Redaction patterns for common secrets
#[derive(Debug, Clone)]
pub struct RedactionEngine {
    patterns: Vec<(SecretType, Regex)>,
    counters: HashMap<SecretType, usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Enumeration of secret types that can be detected and redacted
pub enum SecretType {
    /// AWS Access Key
    AwsAccessKey,
    /// AWS Secret Key
    AwsSecretKey,
    /// Private key (RSA, ECDSA, etc.)
    PrivateKey,
    /// SSH key
    SshKey,
    /// Slack token
    SlackToken,
    /// GitHub token
    GitHubToken,
    /// GitHub Personal Access Token
    GitHubPat,
    /// Google API key
    GoogleApiKey,
    /// Generic secret pattern
    GenericSecret,
    /// Password
    Password,
    /// API key
    ApiKey,
    /// Bearer token
    BearerToken,
}

impl std::fmt::Display for SecretType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretType::AwsAccessKey => write!(f, "AWS_ACCESS_KEY"),
            SecretType::AwsSecretKey => write!(f, "AWS_SECRET_KEY"),
            SecretType::PrivateKey => write!(f, "PRIVATE_KEY"),
            SecretType::SshKey => write!(f, "SSH_KEY"),
            SecretType::SlackToken => write!(f, "SLACK_TOKEN"),
            SecretType::GitHubToken => write!(f, "GITHUB_TOKEN"),
            SecretType::GitHubPat => write!(f, "GITHUB_PAT"),
            SecretType::GoogleApiKey => write!(f, "GOOGLE_API_KEY"),
            SecretType::GenericSecret => write!(f, "SECRET"),
            SecretType::Password => write!(f, "PASSWORD"),
            SecretType::ApiKey => write!(f, "API_KEY"),
            SecretType::BearerToken => write!(f, "BEARER_TOKEN"),
        }
    }
}

impl Default for RedactionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RedactionEngine {
    /// Create a new redaction engine with default patterns
    pub fn new() -> Self {
        let mut patterns = Vec::new();

        // AWS Access Key ID (AKIA...)
        if let Ok(re) = Regex::new(r"AKIA[0-9A-Z]{16}") {
            patterns.push((SecretType::AwsAccessKey, re));
        }

        // AWS Secret Access Key - with word boundaries to prevent ReDoS on long lines
        if let Ok(re) = Regex::new(r"\b[0-9a-zA-Z/+]{40}\b") {
            patterns.push((SecretType::AwsSecretKey, re));
        }

        // Private keys (PEM format)
        if let Ok(re) = Regex::new(r"-----BEGIN (RSA |DSA |EC |OPENSSH )?PRIVATE KEY-----") {
            patterns.push((SecretType::PrivateKey, re));
        }

        // SSH private keys (full block)
        if let Ok(re) = Regex::new(
            r"-----BEGIN OPENSSH PRIVATE KEY-----[\s\S]*?-----END OPENSSH PRIVATE KEY-----",
        ) {
            patterns.push((SecretType::SshKey, re));
        }

        // Slack tokens
        if let Ok(re) = Regex::new(r"xox[baprs]-[0-9a-zA-Z-]+") {
            patterns.push((SecretType::SlackToken, re));
        }

        // GitHub tokens (ghp_...)
        if let Ok(re) = Regex::new(r"ghp_[0-9a-zA-Z]{36,}") {
            patterns.push((SecretType::GitHubToken, re));
        }

        // GitHub PAT (github_pat_...)
        if let Ok(re) = Regex::new(r"github_pat_[0-9a-zA-Z_]+") {
            patterns.push((SecretType::GitHubPat, re));
        }

        // Google API keys (AIza...)
        if let Ok(re) = Regex::new(r"AIza[0-9A-Za-z_-]{35}") {
            patterns.push((SecretType::GoogleApiKey, re));
        }

        // Generic secret= or SECRET= patterns
        if let Ok(re) =
            Regex::new("(?i)(secret|api[_-]?key|token)\\s*=\\s*[\"']?[a-zA-Z0-9_-]{16,}[\"']?")
        {
            patterns.push((SecretType::GenericSecret, re));
        }

        // Password patterns (be careful with false positives)
        if let Ok(re) = Regex::new("(?i)(password|passwd|pwd)\\s*=\\s*[\"'][^\"']{8,}[\"']") {
            patterns.push((SecretType::Password, re));
        }

        // API Key patterns
        if let Ok(re) = Regex::new("(?i)api[_-]?key[\"']?\\s*[:=]\\s*[\"'][a-zA-Z0-9_-]{16,}[\"']")
        {
            patterns.push((SecretType::ApiKey, re));
        }

        // Bearer tokens
        if let Ok(re) = Regex::new(r"(?i)bearer\s+[a-zA-Z0-9_\-\.=]{20,}") {
            patterns.push((SecretType::BearerToken, re));
        }

        Self {
            patterns,
            counters: HashMap::new(),
        }
    }

    /// Redact secrets from content
    pub fn redact(&mut self, content: &str) -> String {
        let mut result = content.to_string();

        for (secret_type, pattern) in &self.patterns {
            let captures: Vec<(String, usize)> = pattern
                .find_iter(&result)
                .map(|m| (m.as_str().to_string(), m.start()))
                .collect();

            for (matched, _) in captures {
                let counter = self.counters.entry(*secret_type).or_insert(0);
                *counter += 1;
                let redacted = format!("REDACTED_{}_{}", secret_type, counter);
                result = result.replace(&matched, &redacted);
                debug!(
                    "Redacted {}: {}",
                    secret_type,
                    matched.chars().take(10).collect::<String>() + "..."
                );
            }
        }

        result
    }

    /// Get summary of redactions
    pub fn summary(&self) -> RedactionSummary {
        RedactionSummary {
            total_redactions: self.counters.values().sum(),
            by_type: self
                .counters
                .iter()
                .map(|(k, v)| (k.to_string(), *v))
                .collect(),
        }
    }

    /// Reset counters
    pub fn reset(&mut self) {
        self.counters.clear();
    }
}

/// Check if a file path should be excluded from scanning
pub fn should_exclude_file(path: &str) -> bool {
    let excluded_extensions = [
        ".exe", ".dll", ".so", ".dylib", ".bin", ".jpg", ".jpeg", ".png", ".gif", ".svg", ".ico",
        ".mp3", ".mp4", ".avi", ".mov", ".wav", ".zip", ".tar", ".gz", ".bz2", ".7z", ".rar",
        ".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ttf", ".otf", ".woff", ".woff2", ".eot",
    ];

    let excluded_paths = [
        ".git/",
        ".svn/",
        ".hg/",
        "node_modules/",
        "target/",
        "dist/",
        "build/",
        ".next/",
        ".nuxt/",
        ".vuepress/dist/",
        "__pycache__/",
        ".pytest_cache/",
        ".idea/",
        ".vscode/",
    ];

    let lower_path = path.to_lowercase();

    // Check extensions
    for ext in &excluded_extensions {
        if lower_path.ends_with(ext) {
            return true;
        }
    }

    // Check path segments
    for excluded in &excluded_paths {
        if path.contains(excluded) {
            return true;
        }
    }

    false
}

/// Check if a file is likely to contain secrets (for prioritization)
pub fn is_secret_likely_file(path: &str) -> bool {
    let secret_files = [
        ".env",
        ".env.local",
        ".env.production",
        ".env.development",
        ".aws/credentials",
        ".ssh/id_rsa",
        ".ssh/id_dsa",
        ".ssh/id_ecdsa",
        "secrets.yml",
        "secrets.yaml",
        "secrets.json",
        "credentials.json",
        "service-account.json",
        "kubeconfig",
        ".dockercfg",
        ".npmrc",
        ".pypirc",
    ];

    let lower_path = path.to_lowercase();

    for secret_file in &secret_files {
        if lower_path.contains(secret_file) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aws_key_redaction() {
        let mut engine = RedactionEngine::new();
        let content = "AKIAIOSFODNN7EXAMPLE";
        let redacted = engine.redact(content);
        assert!(redacted.contains("REDACTED_AWS_ACCESS_KEY"));
        assert!(!redacted.contains("AKIA"));
    }

    #[test]
    fn test_slack_token_redaction() {
        let mut engine = RedactionEngine::new();
        let content = "xoxb-123456789012-abcdefghijklmnop";
        let redacted = engine.redact(content);
        assert!(redacted.contains("REDACTED_SLACK_TOKEN"));
        assert!(!redacted.contains("xoxb-"));
    }

    #[test]
    fn test_private_key_redaction() {
        let mut engine = RedactionEngine::new();
        let content = "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...";
        let redacted = engine.redact(content);
        assert!(redacted.contains("REDACTED_PRIVATE_KEY"));
    }

    #[test]
    fn test_github_token_redaction() {
        let mut engine = RedactionEngine::new();
        let content = "ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
        let redacted = engine.redact(content);
        assert!(redacted.contains("REDACTED_GITHUB_TOKEN"));
    }

    #[test]
    fn test_summary_counts() {
        let mut engine = RedactionEngine::new();

        engine.redact("AKIAIOSFODNN7EXAMPLE");
        engine.redact("AKIAANOTHEREXAMPLE12");
        engine.redact("xoxb-slack-token-here");

        let summary = engine.summary();
        assert_eq!(summary.total_redactions, 3);
        assert_eq!(summary.by_type.get("AWS_ACCESS_KEY"), Some(&2));
        assert_eq!(summary.by_type.get("SLACK_TOKEN"), Some(&1));
    }

    #[test]
    fn test_should_exclude_file() {
        assert!(should_exclude_file("/path/to/image.png"));
        assert!(should_exclude_file("/path/to/binary.exe"));
        assert!(should_exclude_file("node_modules/lodash/index.js"));
        assert!(!should_exclude_file("/path/to/src/main.rs"));
        assert!(!should_exclude_file("/path/to/README.md"));
    }

    #[test]
    fn test_is_secret_likely_file() {
        assert!(is_secret_likely_file(".env"));
        assert!(is_secret_likely_file("config/.env.production"));
        assert!(is_secret_likely_file("secrets.yml"));
        assert!(!is_secret_likely_file("README.md"));
        assert!(!is_secret_likely_file("src/main.rs"));
    }
}
