//! OpenAI-compatible provider client
//!
//! Supports any provider with an OpenAI-compatible API (OpenAI, Azure, LocalAI, etc.)
//!
//! # Features
//! - Chat completions with any OpenAI-compatible endpoint
//! - Provider auto-discovery via `/models` endpoint
//! - Secure API key storage via macOS Keychain
//! - Profile management with disk persistence
//! - Model caching to avoid repeated API calls

#![warn(missing_docs)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

use reqwest::header::{self, HeaderMap, HeaderValue};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info, instrument};
use url::Url;



pub mod profile;
pub mod prompts;
pub mod provider_discovery;
pub mod rate_limiter;

pub use profile::*;
pub use prompts::*;
pub use provider_discovery::*;

/// OpenAI-compatible client with rate limiting support
#[derive(Debug, Clone)]
pub struct OpenAIClient {
    base_url: Url,
    api_key: SecretString,
    http: reqwest::Client,
    default_model: String,
    rate_limiter: Option<rate_limiter::RateLimiter>,
}

/// Configuration for the client
/// Configuration for the client
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Base URL for the API (e.g. "https://api.openai.com/v1")
    pub base_url: String,
    /// API key for authentication
    pub api_key: SecretString,
    /// Default model to use for requests
    pub default_model: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum number of retries for failed requests
    pub max_retries: u32,
    /// Optional rate limiter configuration
    pub rate_limit_config: Option<rate_limiter::RateLimitConfig>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: SecretString::new("".into()),
            default_model: "gpt-4o-mini".to_string(),
            timeout_seconds: get_default_timeout(),
            max_retries: 3,
            rate_limit_config: None,
        }
    }
}

/// Get the default timeout from environment variable or use the default value
fn get_default_timeout() -> u64 {
    std::env::var("HQE_OPENAI_TIMEOUT_SECONDS")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(60) // Default to 60 seconds if not set or invalid
}

/// Chat completion request
/// Chat completion request
#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    /// ID of the model to use
    pub model: String,
    /// List of messages in the conversation
    pub messages: Vec<Message>,
    /// Sampling temperature (0.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Maximum number of tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Desired format for the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
}

/// Format of the response
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ResponseFormat {
    /// JSON object response
    #[serde(rename = "json_object")]
    JsonObject,
    /// Plain text response
    #[serde(rename = "text")]
    Text,
}

/// Chat message
/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message author
    pub role: Role,
    /// Content of the message
    pub content: String,
}

/// Role of the message author
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System prompt
    System,
    /// User input
    User,
    /// Assistant response
    Assistant,
}

/// Chat completion response
/// Chat completion response
#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    /// Unique ID of the response
    pub id: String,
    /// Object type (e.g. "chat.completion")
    pub object: String,
    /// Unix timestamp of creation
    pub created: i64,
    /// Model used for generation
    pub model: String,
    /// List of generated choices
    pub choices: Vec<Choice>,
    /// Token usage statistics
    pub usage: Option<Usage>,
}

/// Generated choice
#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    /// Index of the choice
    pub index: i32,
    /// Generated message
    pub message: Message,
    /// Reason for finishing (e.g. "stop", "length")
    #[serde(rename = "finish_reason")]
    pub finish_reason: Option<String>,
}

/// Token usage statistics
#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    /// Tokens in the prompt
    #[serde(rename = "prompt_tokens")]
    pub prompt_tokens: i32,
    /// Tokens in the completion
    #[serde(rename = "completion_tokens")]
    pub completion_tokens: i32,
    /// Total tokens used
    #[serde(rename = "total_tokens")]
    pub total_tokens: i32,
}

/// API error response
/// API error response
#[derive(Debug, Clone, Deserialize)]
pub struct ApiError {
    /// Detailed error information
    pub error: ErrorDetail,
}

/// Detailed error information
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorDetail {
    /// Error message
    pub message: String,
    /// Error type
    #[serde(rename = "type")]
    pub error_type: String,
    /// Error code
    pub code: Option<String>,
}

// Re-export ProviderProfile from hqe-protocol for backward compatibility
pub use hqe_protocol::models::{ProviderProfile, ProviderKind};

impl OpenAIClient {
    /// Create a new client
    pub fn new(config: ClientConfig) -> anyhow::Result<Self> {
        let base_url = Url::parse(&config.base_url)?;

        // Log security-relevant information (without exposing the API key)
        info!(
            "Creating OpenAI client for URL: {}",
            base_url.domain().unwrap_or("unknown")
        );

        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;

        let rate_limiter = config.rate_limit_config.map(rate_limiter::RateLimiter::new);

        Ok(Self {
            base_url,
            api_key: config.api_key,
            http,
            default_model: config.default_model,
            rate_limiter,
        })
    }

    /// Set rate limiting configuration
    pub fn with_rate_limiting(mut self, config: rate_limiter::RateLimitConfig) -> Self {
        self.rate_limiter = Some(rate_limiter::RateLimiter::new(config));
        self
    }

    /// Get the default model configured for this client
    pub fn default_model(&self) -> &str {
        &self.default_model
    }

    /// Build request headers
    fn build_headers(&self) -> anyhow::Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        let api_key_val = HeaderValue::from_str(&format!("Bearer {}", self.api_key.expose_secret()))
            .map_err(|e| anyhow::anyhow!("Invalid API key characters: {}", e))?;

        headers.insert(header::AUTHORIZATION, api_key_val);

        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        Ok(headers)
    }

    /// Send a chat completion request
    #[instrument(skip(self, request))]
    pub async fn chat(&self, request: ChatRequest) -> anyhow::Result<ChatResponse> {
        // Apply rate limiting before making the request
        if let Some(limiter) = &self.rate_limiter {
            // Estimate tokens: max_tokens + rough estimate of input size
            let estimated_tokens = request.max_tokens.map(|t| t as u32);
            limiter.acquire(estimated_tokens).await;
        }

        // Ensure trailing slash to prevent Url::join from stripping the last path segment
        // Url::join behavior: "v1".join("chat") = "chat" (replaces last segment)
        //                      "v1/".join("chat") = "v1/chat" (appends)
        let url = if self.base_url.path().ends_with('/') {
            self.base_url.join("chat/completions")?
        } else {
            // Manually construct to avoid segment replacement
            let mut url_str = self.base_url.to_string();
            if !url_str.ends_with('/') {
                url_str.push('/');
            }
            url_str.push_str("chat/completions");
            Url::parse(&url_str)?
        };
        let headers = self.build_headers()?;

        debug!("Sending chat request to {}", url);

        let response = self
            .http
            .post(url)
            .headers(headers)
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if status.is_success() {
            let chat_response: ChatResponse = response.json().await?;
            info!(
                "Chat completion successful: {} tokens used",
                chat_response
                    .usage
                    .as_ref()
                    .map(|u| u.total_tokens)
                    .unwrap_or(0)
            );
            Ok(chat_response)
        } else {
            let error_text = response.text().await?;
            // Log the full error for debugging but don't expose it in the user-facing error
            error!("API error ({}): {}", status, error_text);

            match serde_json::from_str::<ApiError>(&error_text) {
                Ok(api_error) => {
                    // Only expose the error type, not the full message which might contain sensitive info
                    anyhow::bail!(
                        "API error: {} ({})",
                        // Sanitize the message to prevent information disclosure
                        sanitize_error_message(&api_error.error.message),
                        api_error.error.error_type
                    )
                }
                Err(_) => {
                    // Don't expose the raw error text which might contain sensitive information
                    anyhow::bail!(
                        "HTTP error {}: {}",
                        status,
                        status.canonical_reason().unwrap_or("Unknown error")
                    )
                }
            }
        }
    }

    /// Simple chat with default model
    pub async fn simple_chat(&self, system: &str, user: &str) -> anyhow::Result<String> {
        let request = ChatRequest {
            model: self.default_model.clone(),
            messages: vec![
                Message {
                    role: Role::System,
                    content: system.to_string(),
                },
                Message {
                    role: Role::User,
                    content: user.to_string(),
                },
            ],
            temperature: Some(0.1),
            max_tokens: Some(4000),
            response_format: None,
        };

        let response = self.chat(request).await?;

        response
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| anyhow::anyhow!("No response content"))
    }

    /// Test connection to provider
    pub async fn test_connection(&self) -> anyhow::Result<bool> {
        // Try to list models or make a minimal request
        let test_request = ChatRequest {
            model: self.default_model.clone(),
            messages: vec![Message {
                role: Role::User,
                content: "Hi".to_string(),
            }],
            temperature: Some(0.0),
            max_tokens: Some(5),
            response_format: None,
        };

        match self.chat(test_request).await {
            Ok(_) => Ok(true),
            Err(e) => {
                error!("Connection test failed: {}", e);
                Ok(false)
            }
        }
    }
}

/// Sanitize error messages to prevent information disclosure
fn sanitize_error_message(message: &str) -> String {
    // Define patterns for sensitive data (API keys, secrets, tokens)
    // - Matches standard Bearer tokens, hex strings, and common API key formats
    let patterns = [
        (r"(?i)api[_-]?key", "api_key"),
        (r"(?i)secret", "secret"),
        (r"(?i)token", "token"),
        (r"(?i)password", "password"),
        (r"(?i)credential", "credential"),
        (r"sk-[a-zA-Z0-9]{20,}", "sk-***"), // OpenAI style
        (r"gh[pousr]_[A-Za-z0-9_]{36,}", "ghp_***"), // GitHub style
        (r"[a-zA-Z0-9_-]{32,}", "***REDACTED***"), // Long alphanumeric strings
    ];

    let mut sanitized = message.to_string();
    
    for (pattern, replacement) in patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            sanitized = re.replace_all(&sanitized, replacement).to_string();
        }
    }

    // Truncate very long messages to prevent exposing too much detail
    if sanitized.len() > 256 {
        format!("{}... [truncated]", &sanitized[..256])
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.base_url, "https://api.openai.com/v1");
        assert_eq!(config.default_model, "gpt-4o-mini");
        assert_eq!(config.timeout_seconds, 60);
    }

    #[tokio::test]
    async fn test_simple_chat_request() {
        // This would normally use a mock server
        let config = ClientConfig {
            base_url: "http://localhost:1234".to_string(),
            api_key: SecretString::new("test".into()),
            default_model: "test-model".to_string(),
            timeout_seconds: 5,
            max_retries: 0,
            rate_limit_config: None,
        };

        // Would need mockito or similar to test properly
        // For now just verify it builds
        let client = OpenAIClient::new(config);
        assert!(client.is_ok());
    }
}
