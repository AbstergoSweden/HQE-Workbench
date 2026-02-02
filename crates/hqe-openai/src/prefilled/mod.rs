//! Prefilled Provider API Specifications
//!
//! This module provides complete provider profiles for popular LLM services.
//! Each spec includes base URL, auth scheme, default headers, discovery behavior,
//! and documented quirks.

use hqe_protocol::models::ProviderKind;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Authentication scheme for a provider
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthScheme {
    /// Bearer token in Authorization header (standard OpenAI)
    Bearer,
    /// API key in x-api-key header
    ApiKeyHeader,
    /// API key in query parameter
    ApiKeyQuery,
    /// Custom header
    CustomHeader,
}

impl std::fmt::Display for AuthScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthScheme::Bearer => write!(f, "Bearer"),
            AuthScheme::ApiKeyHeader => write!(f, "X-API-Key"),
            AuthScheme::ApiKeyQuery => write!(f, "Query Param"),
            AuthScheme::CustomHeader => write!(f, "Custom"),
        }
    }
}

/// A prefilled provider specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSpec {
    /// Unique identifier (e.g., "openai", "anthropic")
    pub id: String,
    /// Human-readable display name
    pub display_name: String,
    /// Provider kind
    pub kind: ProviderKind,
    /// Base URL (e.g., "https://api.openai.com/v1")
    pub base_url: String,
    /// Authentication scheme
    pub auth_scheme: AuthScheme,
    /// Default headers to include with every request
    pub default_headers: HashMap<String, String>,
    /// Discovery endpoint path (if different from /models)
    pub discovery_endpoint: Option<String>,
    /// Default model to use
    pub default_model: String,
    /// Recommended timeout in seconds
    pub recommended_timeout_s: u64,
    /// Documented quirks and special behaviors
    pub quirks: Vec<String>,
    /// Provider website URL
    pub website_url: String,
    /// Documentation URL
    pub docs_url: String,
    /// Whether this provider supports streaming
    pub supports_streaming: bool,
    /// Whether this provider supports function calling
    pub supports_tools: bool,
    /// Rate limit notes
    pub rate_limit_notes: Option<String>,
}

impl ProviderSpec {
    /// Create a builder for the spec
    pub fn builder(id: impl Into<String>) -> ProviderSpecBuilder {
        ProviderSpecBuilder::new(id)
    }

    /// Get the auth header name based on scheme
    pub fn auth_header_name(&self) -> &str {
        match self.auth_scheme {
            AuthScheme::Bearer => "Authorization",
            AuthScheme::ApiKeyHeader => "x-api-key",
            AuthScheme::ApiKeyQuery => "api_key",
            AuthScheme::CustomHeader => "Authorization",
        }
    }

    /// Format an API key according to the auth scheme
    pub fn format_api_key(&self, key: &str) -> String {
        match self.auth_scheme {
            AuthScheme::Bearer => format!("Bearer {}", key),
            AuthScheme::ApiKeyHeader => key.to_string(),
            AuthScheme::ApiKeyQuery => key.to_string(),
            AuthScheme::CustomHeader => key.to_string(),
        }
    }
}

/// Builder for provider specs
pub struct ProviderSpecBuilder {
    id: String,
    display_name: Option<String>,
    kind: Option<ProviderKind>,
    base_url: Option<String>,
    auth_scheme: AuthScheme,
    default_headers: HashMap<String, String>,
    discovery_endpoint: Option<String>,
    default_model: Option<String>,
    recommended_timeout_s: u64,
    quirks: Vec<String>,
    website_url: Option<String>,
    docs_url: Option<String>,
    supports_streaming: bool,
    supports_tools: bool,
    rate_limit_notes: Option<String>,
}

impl ProviderSpecBuilder {
    fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            display_name: None,
            kind: None,
            base_url: None,
            auth_scheme: AuthScheme::Bearer,
            default_headers: HashMap::new(),
            discovery_endpoint: None,
            default_model: None,
            recommended_timeout_s: 60,
            quirks: Vec::new(),
            website_url: None,
            docs_url: None,
            supports_streaming: true,
            supports_tools: true,
            rate_limit_notes: None,
        }
    }

    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    pub fn kind(mut self, kind: ProviderKind) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    pub fn auth_scheme(mut self, scheme: AuthScheme) -> Self {
        self.auth_scheme = scheme;
        self
    }

    pub fn default_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(key.into(), value.into());
        self
    }

    pub fn discovery_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.discovery_endpoint = Some(endpoint.into());
        self
    }

    pub fn default_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = Some(model.into());
        self
    }

    pub fn timeout(mut self, seconds: u64) -> Self {
        self.recommended_timeout_s = seconds;
        self
    }

    pub fn quirk(mut self, quirk: impl Into<String>) -> Self {
        self.quirks.push(quirk.into());
        self
    }

    pub fn website(mut self, url: impl Into<String>) -> Self {
        self.website_url = Some(url.into());
        self
    }

    pub fn docs(mut self, url: impl Into<String>) -> Self {
        self.docs_url = Some(url.into());
        self
    }

    pub fn streaming(mut self, supported: bool) -> Self {
        self.supports_streaming = supported;
        self
    }

    pub fn tools(mut self, supported: bool) -> Self {
        self.supports_tools = supported;
        self
    }

    pub fn rate_limit_notes(mut self, notes: impl Into<String>) -> Self {
        self.rate_limit_notes = Some(notes.into());
        self
    }

    pub fn build(self) -> ProviderSpec {
        ProviderSpec {
            id: self.id.clone(),
            display_name: self.display_name.unwrap_or_else(|| self.id.clone()),
            kind: self.kind.unwrap_or(ProviderKind::Generic),
            base_url: self.base_url.expect("base_url is required"),
            auth_scheme: self.auth_scheme,
            default_headers: self.default_headers,
            discovery_endpoint: self.discovery_endpoint,
            default_model: self.default_model.unwrap_or_else(|| "gpt-4o-mini".to_string()),
            recommended_timeout_s: self.recommended_timeout_s,
            quirks: self.quirks,
            website_url: self.website_url.unwrap_or_default(),
            docs_url: self.docs_url.unwrap_or_default(),
            supports_streaming: self.supports_streaming,
            supports_tools: self.supports_tools,
            rate_limit_notes: self.rate_limit_notes,
        }
    }
}

/// Get all prefilled provider specs
pub fn all_specs() -> Vec<ProviderSpec> {
    vec![
        openai(),
        anthropic(),
        venice(),
        openrouter(),
        xai_grok(),
        kimi(),
    ]
}

/// OpenAI spec
pub fn openai() -> ProviderSpec {
    ProviderSpec::builder("openai")
        .display_name("OpenAI")
        .kind(ProviderKind::OpenAI)
        .base_url("https://api.openai.com/v1")
        .auth_scheme(AuthScheme::Bearer)
        .default_header("Content-Type", "application/json")
        .default_model("gpt-4o-mini")
        .timeout(60)
        .website("https://openai.com")
        .docs("https://platform.openai.com/docs")
        .streaming(true)
        .tools(true)
        .rate_limit_notes("Rate limits vary by tier. Check your dashboard.")
        .build()
}

/// Anthropic spec
pub fn anthropic() -> ProviderSpec {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());

    ProviderSpec::builder("anthropic")
        .display_name("Anthropic")
        .kind(ProviderKind::Generic) // Using Generic until we add Anthropic variant
        .base_url("https://api.anthropic.com/v1")
        .auth_scheme(AuthScheme::Bearer)
        .default_header("Content-Type", "application/json")
        .default_header("anthropic-version", "2023-06-01")
        .default_model("claude-3-5-sonnet-latest")
        .timeout(60)
        .website("https://anthropic.com")
        .docs("https://docs.anthropic.com")
        .streaming(true)
        .tools(true)
        .quirk("Requires anthropic-version header")
        .quirk("Uses 'max_tokens' not 'max_completion_tokens'")
        .quirk("Messages API slightly different from OpenAI format")
        .rate_limit_notes("Rate limits: https://docs.anthropic.com/en/api/rate-limits")
        .build()
}

/// Venice.ai spec
pub fn venice() -> ProviderSpec {
    ProviderSpec::builder("venice")
        .display_name("Venice.ai")
        .kind(ProviderKind::Venice)
        .base_url("https://api.venice.ai/api/v1")
        .auth_scheme(AuthScheme::Bearer)
        .default_header("Content-Type", "application/json")
        .default_model("deepseek-r1-671b")
        .timeout(120)
        .website("https://venice.ai")
        .docs("https://docs.venice.ai")
        .streaming(true)
        .tools(false) // Verify current capabilities
        .quirk("Discovery endpoint uses '?type=all' parameter")
        .quirk("Model spec includes rich capability metadata")
        .quirk("Pricing returned in USD per million tokens")
        .rate_limit_notes("Varies by model. Check Venice dashboard.")
        .build()
}

/// OpenRouter spec
pub fn openrouter() -> ProviderSpec {
    ProviderSpec::builder("openrouter")
        .display_name("OpenRouter")
        .kind(ProviderKind::OpenRouter)
        .base_url("https://openrouter.ai/api/v1")
        .auth_scheme(AuthScheme::Bearer)
        .default_header("Content-Type", "application/json")
        .default_header("HTTP-Referer", "https://hqe-workbench.local")
        .default_header("X-Title", "HQE Workbench")
        .default_model("openai/gpt-4o-mini")
        .timeout(120)
        .website("https://openrouter.ai")
        .docs("https://openrouter.ai/docs")
        .streaming(true)
        .tools(true)
        .quirk("Requires HTTP-Referer and X-Title headers")
        .quirk("Model IDs include provider prefix (e.g., 'openai/gpt-4o')")
        .quirk("Provides context_length and pricing in /models")
        .quirk("Some models may have different capabilities")
        .rate_limit_notes("Rate limits: https://openrouter.ai/docs#limits")
        .build()
}

/// xAI / Grok spec
pub fn xai_grok() -> ProviderSpec {
    ProviderSpec::builder("xai")
        .display_name("xAI (Grok)")
        .kind(ProviderKind::XAI)
        .base_url("https://api.x.ai/v1")
        .auth_scheme(AuthScheme::Bearer)
        .default_header("Content-Type", "application/json")
        .default_model("grok-2-latest")
        .timeout(60)
        .website("https://x.ai")
        .docs("https://docs.x.ai")
        .streaming(true)
        .tools(true)
        .quirk("Model list is basic (id only)")
        .rate_limit_notes("Check xAI console for current limits")
        .build()
}

/// Kimi (Moonshot) spec
pub fn kimi() -> ProviderSpec {
    ProviderSpec::builder("kimi")
        .display_name("Kimi (Moonshot)")
        .kind(ProviderKind::Generic)
        .base_url("https://api.moonshot.cn/v1")
        .auth_scheme(AuthScheme::Bearer)
        .default_header("Content-Type", "application/json")
        .default_model("moonshot-v1-8k")
        .timeout(60)
        .website("https://moonshot.cn")
        .docs("https://platform.moonshot.cn/docs")
        .streaming(true)
        .tools(true)
        .quirk("Chinese provider - may have regional latency")
        .quirk("Supports long context models (up to 1M tokens)")
        .rate_limit_notes("TPM limits vary by model tier")
        .build()
}

/// Get a spec by ID
pub fn get_spec(id: &str) -> Option<ProviderSpec> {
    all_specs().into_iter().find(|s| s.id == id)
}

/// Get spec IDs and display names for UI
pub fn spec_list() -> Vec<(String, String)> {
    all_specs()
        .into_iter()
        .map(|s| (s.id, s.display_name))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_spec() {
        let spec = openai();
        assert_eq!(spec.id, "openai");
        assert_eq!(spec.base_url, "https://api.openai.com/v1");
        assert_eq!(spec.auth_scheme, AuthScheme::Bearer);
        assert!(spec.supports_streaming);
        assert!(spec.supports_tools);
    }

    #[test]
    fn test_anthropic_spec() {
        let spec = anthropic();
        assert_eq!(spec.id, "anthropic");
        assert!(spec.default_headers.contains_key("anthropic-version"));
        assert!(!spec.quirks.is_empty());
    }

    #[test]
    fn test_venice_spec() {
        let spec = venice();
        assert_eq!(spec.kind, ProviderKind::Venice);
        assert_eq!(spec.base_url, "https://api.venice.ai/api/v1");
        assert_eq!(spec.recommended_timeout_s, 120);
    }

    #[test]
    fn test_openrouter_spec() {
        let spec = openrouter();
        assert!(spec.default_headers.contains_key("HTTP-Referer"));
        assert!(spec.default_headers.contains_key("X-Title"));
    }

    #[test]
    fn test_xai_spec() {
        let spec = xai_grok();
        assert_eq!(spec.id, "xai");
        assert_eq!(spec.kind, ProviderKind::XAI);
    }

    #[test]
    fn test_kimi_spec() {
        let spec = kimi();
        assert_eq!(spec.id, "kimi");
        assert_eq!(spec.base_url, "https://api.moonshot.cn/v1");
    }

    #[test]
    fn test_all_specs_count() {
        let specs = all_specs();
        assert_eq!(specs.len(), 6);
    }

    #[test]
    fn test_get_spec_found() {
        let spec = get_spec("openai");
        assert!(spec.is_some());
        assert_eq!(spec.unwrap().id, "openai");
    }

    #[test]
    fn test_get_spec_not_found() {
        let spec = get_spec("nonexistent");
        assert!(spec.is_none());
    }

    #[test]
    fn test_spec_list() {
        let list = spec_list();
        assert_eq!(list.len(), 6);
        assert!(list.iter().any(|(id, _)| id == "openai"));
    }

    #[test]
    fn test_format_api_key_bearer() {
        let spec = openai();
        assert_eq!(spec.format_api_key("test123"), "Bearer test123");
    }

    #[test]
    fn test_format_api_key_plain() {
        let mut spec = openai();
        spec.auth_scheme = AuthScheme::ApiKeyHeader;
        assert_eq!(spec.format_api_key("test123"), "test123");
    }
}
