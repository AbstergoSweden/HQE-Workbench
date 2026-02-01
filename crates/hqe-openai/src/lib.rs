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

use reqwest::header::{self, HeaderMap, HeaderName, HeaderValue};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info, instrument};
use url::Url;

/// Analysis module for processing content with LLMs.
pub mod analysis;
/// Provider profile loading, saving, and keychain integration.
pub mod profile;
/// Prompt templates and helpers for LLM requests.
pub mod prompts;
/// Provider and model discovery utilities for OpenAI-compatible APIs.
pub mod provider_discovery;
/// Rate limiting utilities for outbound provider requests.
pub mod rate_limiter;

pub use analysis::*;
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
    additional_headers: HashMap<String, String>,
    organization: Option<String>,
    project: Option<String>,
    max_retries: u32,
    local_db: Option<hqe_core::persistence::LocalDb>,
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
    /// Additional HTTP headers (excluding Authorization)
    pub headers: Option<HashMap<String, String>>,
    /// Organization identifier (OpenAI-compatible header)
    pub organization: Option<String>,
    /// Project identifier (OpenAI-compatible header)
    pub project: Option<String>,
    /// Disable use of system proxy configuration (macOS-safe)
    pub disable_system_proxy: bool,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum number of retries for failed requests
    pub max_retries: u32,
    /// Optional rate limiter configuration
    pub rate_limit_config: Option<rate_limiter::RateLimitConfig>,
    /// Enable local decision cache and logging (Privacy-First)
    pub cache_enabled: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: SecretString::new("".into()),
            default_model: "gpt-4o-mini".to_string(),
            headers: None,
            organization: None,
            project: None,
            disable_system_proxy: false,
            timeout_seconds: get_default_timeout(),
            max_retries: 3,
            rate_limit_config: None,
            cache_enabled: true,
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
    /// Number between -2.0 and 2.0 to penalize frequent tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    /// Number between -2.0 and 2.0 to penalize repeated topics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    /// Repetition penalty; 1.0 means no penalty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repetition_penalty: Option<f32>,
    /// Whether to include log probabilities in the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,
    /// Number of top logprobs to return (if enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u32>,
    /// Sampling temperature (0.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Minimum temperature for dynamic temperature scaling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_temp: Option<f32>,
    /// Maximum temperature for dynamic temperature scaling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_temp: Option<f32>,
    /// Nucleus sampling parameter (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// Top-k sampling parameter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    /// Maximum number of tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Preferred max tokens (OpenAI/Venice); supersedes max_tokens when supported
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,
    /// How many choices to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    /// Stop sequence(s)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Stop>,
    /// Stop token IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_token_ids: Option<Vec<u32>>,
    /// Deterministic seed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    /// Optional end-user identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Prompt cache routing key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
    /// Prompt cache retention policy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: Option<String>,
    /// Reasoning effort (OpenAI/Venice compatible)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    /// Reasoning configuration object
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningConfig>,
    /// Whether to stream responses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// Stream options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,
    /// Tool choice configuration (OpenAI-compatible)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<serde_json::Value>,
    /// Tools available to the model (OpenAI-compatible)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>,
    /// Venice-specific parameters (optional, forwarded as-is)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venice_parameters: Option<serde_json::Value>,
    /// Whether to enable parallel tool calls
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    /// Desired format for the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
}

/// Stop sequences for chat completion
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Stop {
    /// Single stop string
    String(String),
    /// Multiple stop strings
    Array(Vec<String>),
}

/// Streaming options
#[derive(Debug, Clone, Serialize)]
pub struct StreamOptions {
    /// Include usage info in the stream
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_usage: Option<bool>,
}

/// Reasoning configuration
#[derive(Debug, Clone, Serialize)]
pub struct ReasoningConfig {
    /// Effort level for reasoning models
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
}

/// Format of the response
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ResponseFormat {
    /// JSON object response
    #[serde(rename = "json_object")]
    JsonObject,
    /// JSON schema response
    #[serde(rename = "json_schema")]
    JsonSchema {
        /// JSON schema to enforce in the response
        json_schema: serde_json::Value,
    },
    /// Plain text response
    #[serde(rename = "text")]
    Text,
}

/// Message content can be either a plain string or an array of content parts (OpenAI/Venice schema).
///
/// We keep parts as raw JSON to tolerate provider extensions while still being able to extract the
/// text-only payload when needed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Plain string message content.
    Text(String),
    /// Array of structured content parts (text/image/audio/etc).
    Parts(Vec<serde_json::Value>),
}

impl MessageContent {
    /// Best-effort text extraction for downstream parsing (e.g. JSON-mode responses).
    pub fn to_text_lossy(&self) -> Option<String> {
        match self {
            MessageContent::Text(s) => Some(s.clone()),
            MessageContent::Parts(parts) => {
                let mut out = String::new();
                for part in parts {
                    // Venice spec uses: { type: "text", text: "..." }
                    if let Some(text) = part
                        .get("type")
                        .and_then(|t| t.as_str())
                        .filter(|t| *t == "text")
                        .and_then(|_| part.get("text"))
                        .and_then(|t| t.as_str())
                    {
                        if !out.is_empty() {
                            out.push('\n');
                        }
                        out.push_str(text);
                        continue;
                    }

                    // Some providers may omit the "type" field for simple parts.
                    if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                        if !out.is_empty() {
                            out.push('\n');
                        }
                        out.push_str(text);
                    }
                }

                if out.is_empty() {
                    None
                } else {
                    Some(out)
                }
            }
        }
    }
}

impl From<String> for MessageContent {
    fn from(value: String) -> Self {
        MessageContent::Text(value)
    }
}

impl From<&str> for MessageContent {
    fn from(value: &str) -> Self {
        MessageContent::Text(value.to_string())
    }
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message author
    pub role: Role,
    /// Content of the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<MessageContent>,
    /// Tool call details (OpenAI-compatible responses may omit content)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<serde_json::Value>>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Detailed error information
    pub error: ErrorDetail,
}

/// Detailed error information
#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub use hqe_protocol::models::{ProviderKind, ProviderProfile};

impl OpenAIClient {
    /// Create a new client
    pub fn new(config: ClientConfig) -> anyhow::Result<Self> {
        let base_url = provider_discovery::sanitize_base_url(&config.base_url)
            .map_err(|e| anyhow::anyhow!("Invalid base URL: {e}"))?;

        // Log security-relevant information (without exposing the API key)
        info!(
            "Creating OpenAI client for URL: {}",
            base_url.domain().unwrap_or("unknown")
        );

        let mut builder =
            reqwest::Client::builder().timeout(Duration::from_secs(config.timeout_seconds));
        if config.disable_system_proxy {
            builder = builder.no_proxy();
        }

        let http = builder.build()?;

        let rate_limiter = config.rate_limit_config.map(rate_limiter::RateLimiter::new);

        Ok(Self {
            base_url,
            api_key: config.api_key,
            http,
            default_model: config.default_model,
            rate_limiter,
            additional_headers: config.headers.unwrap_or_default(),
            organization: config.organization,
            project: config.project,
            max_retries: config.max_retries,
            local_db: if config.cache_enabled {
                match hqe_core::persistence::LocalDb::init() {
                    Ok(db) => Some(db),
                    Err(e) => {
                        error!("Failed to initialize local DB for caching: {}", e);
                        None
                    }
                }
            } else {
                None
            },
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
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        let api_key = self.api_key.expose_secret();
        if !api_key.is_empty() {
            let api_key_val = HeaderValue::from_str(&format!("Bearer {}", api_key))
                .map_err(|e| anyhow::anyhow!("Invalid API key characters: {}", e))?;
            headers.insert(header::AUTHORIZATION, api_key_val);
        }

        if let Some(org) = &self.organization {
            headers.insert(
                HeaderName::from_static("openai-organization"),
                HeaderValue::from_str(org)
                    .map_err(|e| anyhow::anyhow!("Invalid organization header: {}", e))?,
            );
        }

        if let Some(project) = &self.project {
            headers.insert(
                HeaderName::from_static("openai-project"),
                HeaderValue::from_str(project)
                    .map_err(|e| anyhow::anyhow!("Invalid project header: {}", e))?,
            );
        }

        for (key, value) in &self.additional_headers {
            let header_name = HeaderName::from_bytes(key.as_bytes())
                .map_err(|e| anyhow::anyhow!("Invalid header name '{}': {}", key, e))?;
            let header_value = HeaderValue::from_str(value)
                .map_err(|e| anyhow::anyhow!("Invalid header value for '{}': {}", key, e))?;
            headers.insert(header_name, header_value);
        }

        Ok(headers)
    }

    /// Send a chat completion request
    #[instrument(skip(self, request))]
    pub async fn chat(&self, request: ChatRequest) -> anyhow::Result<ChatResponse> {
        // Apply rate limiting before making the request
        if let Some(limiter) = &self.rate_limiter {
            // Estimate tokens: max_tokens + rough estimate of input size
            let estimated_tokens = request.max_completion_tokens.or(request.max_tokens);
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
        let mut last_error: Option<anyhow::Error> = None;
        let max_attempts = self.max_retries.saturating_add(1).max(1);

        // Calculate hash for caching
        let request_hash = if self.local_db.is_some() {
            match serde_json::to_string(&request) {
                Ok(prompt_json) => {
                    // Create a deterministic hash from the request
                    // We use prompt_json as both hash input + raw input
                    let hash = hqe_core::persistence::LocalDb::calculate_hash(
                        &request.model,
                        &prompt_json, // simplifying: using full json as messages input for now
                        "",
                    );

                    // Check cache
                    if let Some(db) = &self.local_db {
                        if let Ok(Some(cached_resp)) = db.get_cached_response(&hash) {
                            if let Ok(response) = serde_json::from_str::<ChatResponse>(&cached_resp)
                            {
                                info!("Cache HIT for model {}", request.model);
                                return Ok(response);
                            }
                        }
                    }
                    Some((hash, prompt_json))
                }
                Err(_) => None,
            }
        } else {
            None
        };

        for attempt in 0..max_attempts {
            let headers = self.build_headers()?;

            debug!(
                attempt = attempt + 1,
                max_attempts, "Sending chat request to {}", url
            );

            let response = self
                .http
                .post(url.clone())
                .headers(headers)
                .json(&request)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        let chat_response: ChatResponse = resp.json().await?;
                        info!(
                            "Chat completion successful: {} tokens used",
                            chat_response
                                .usage
                                .as_ref()
                                .map(|u| u.total_tokens)
                                .unwrap_or(0)
                        );

                        // Cache the response and log interaction
                        if let Some((hash, prompt_json)) = &request_hash {
                            if let Some(db) = &self.local_db {
                                if let Ok(resp_json) = serde_json::to_string(&chat_response) {
                                    // Store in cache
                                    let _ = db.cache_response(
                                        hash,
                                        &request.model,
                                        prompt_json,
                                        &resp_json,
                                    );

                                    // Log session interaction (audit)
                                    // Extract last user message content for preview
                                    let user_content = request
                                        .messages
                                        .last()
                                        .and_then(|m| m.content.as_ref())
                                        .and_then(|c| c.to_text_lossy())
                                        .unwrap_or_default();

                                    let id = uuid::Uuid::new_v4().to_string();
                                    let _ = db.log_interaction(
                                        &id,
                                        "user",
                                        &user_content,
                                        Some(prompt_json),
                                    );
                                    let _ = db.log_interaction(
                                        &id,
                                        "assistant",
                                        "Response received",
                                        Some(&resp_json),
                                    );
                                }
                            }
                        }

                        return Ok(chat_response);
                    }

                    let error_text = resp.text().await.unwrap_or_default();
                    error!("API error ({}): {}", status, error_text);

                    if attempt + 1 < max_attempts && is_retryable_status(status) {
                        let backoff = retry_backoff(attempt);
                        debug!(
                            status = %status,
                            backoff_ms = backoff.as_millis(),
                            "Retrying chat request"
                        );
                        tokio::time::sleep(backoff).await;
                        continue;
                    }

                    last_error = Some(match serde_json::from_str::<ApiError>(&error_text) {
                        Ok(api_error) => anyhow::anyhow!(
                            "API error: {} ({})",
                            sanitize_error_message(&api_error.error.message),
                            api_error.error.error_type
                        ),
                        Err(_) => anyhow::anyhow!(
                            "HTTP error {}: {}",
                            status,
                            status.canonical_reason().unwrap_or("Unknown error")
                        ),
                    });
                }
                Err(err) => {
                    if attempt + 1 < max_attempts && is_retryable_error(&err) {
                        let backoff = retry_backoff(attempt);
                        debug!(
                            backoff_ms = backoff.as_millis(),
                            "Retrying chat request after transport error: {}", err
                        );
                        tokio::time::sleep(backoff).await;
                        continue;
                    }

                    return Err(err.into());
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Request failed")))
    }

    /// Simple chat with default model
    pub async fn simple_chat(&self, system: &str, user: &str) -> anyhow::Result<String> {
        let request = ChatRequest {
            model: self.default_model.clone(),
            messages: vec![
                Message {
                    role: Role::System,
                    content: Some(system.to_string().into()),
                    tool_calls: None,
                },
                Message {
                    role: Role::User,
                    content: Some(user.to_string().into()),
                    tool_calls: None,
                },
            ],
            frequency_penalty: None,
            presence_penalty: None,
            repetition_penalty: None,
            logprobs: None,
            top_logprobs: None,
            temperature: Some(0.1),
            min_temp: None,
            max_temp: None,
            top_p: None,
            top_k: None,
            max_tokens: Some(4000),
            max_completion_tokens: None,
            n: None,
            stop: None,
            stop_token_ids: None,
            seed: None,
            user: None,
            prompt_cache_key: None,
            prompt_cache_retention: None,
            reasoning_effort: None,
            reasoning: None,
            stream: None,
            stream_options: None,
            tool_choice: None,
            tools: None,
            venice_parameters: None,
            parallel_tool_calls: None,
            response_format: None,
        };

        let response = self.chat(request).await?;

        response
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content.and_then(|c| c.to_text_lossy()))
            .ok_or_else(|| anyhow::anyhow!("No response content"))
    }

    /// Test connection to provider
    pub async fn test_connection(&self) -> anyhow::Result<bool> {
        // Try to list models or make a minimal request
        let test_request = ChatRequest {
            model: self.default_model.clone(),
            messages: vec![Message {
                role: Role::User,
                content: Some("Hi".into()),
                tool_calls: None,
            }],
            frequency_penalty: None,
            presence_penalty: None,
            repetition_penalty: None,
            logprobs: None,
            top_logprobs: None,
            temperature: Some(0.0),
            min_temp: None,
            max_temp: None,
            top_p: None,
            top_k: None,
            max_tokens: Some(5),
            max_completion_tokens: None,
            n: None,
            stop: None,
            stop_token_ids: None,
            seed: None,
            user: None,
            prompt_cache_key: None,
            prompt_cache_retention: None,
            reasoning_effort: None,
            reasoning: None,
            stream: None,
            stream_options: None,
            tool_choice: None,
            tools: None,
            venice_parameters: None,
            parallel_tool_calls: None,
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

fn is_retryable_status(status: reqwest::StatusCode) -> bool {
    matches!(status.as_u16(), 408 | 429 | 500 | 502 | 503 | 504)
}

fn is_retryable_error(err: &reqwest::Error) -> bool {
    err.is_timeout() || err.is_connect()
}

fn retry_backoff(attempt: u32) -> Duration {
    let exp = 2u64.saturating_pow(attempt.min(6));
    let ms = 200u64.saturating_mul(exp).min(2_000);
    Duration::from_millis(ms)
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
            headers: None,
            organization: None,
            project: None,
            disable_system_proxy: true,
            timeout_seconds: 5,
            max_retries: 0,
            rate_limit_config: None,
            cache_enabled: false,
        };

        // Would need mockito or similar to test properly
        // For now just verify it builds
        let client = OpenAIClient::new(config);
        assert!(client.is_ok());
    }
}
