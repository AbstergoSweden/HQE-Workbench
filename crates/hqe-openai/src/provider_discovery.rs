//! Provider Auto-Discovery for OpenAI-compatible APIs
//!
//! Supports chat-model-only discovery for:
//! - Venice.ai
//! - xAI / Grok
//! - OpenRouter
//! - Generic OpenAI-compatible endpoints

use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tracing::{debug, info, instrument, warn};
use url::Url;

// Re-export ProviderKind from hqe-protocol for consistency
pub use hqe_protocol::models::ProviderKind;

/// Extension trait for ProviderKind with detection logic
pub trait ProviderKindExt {
    /// Auto-detect provider kind from base URL hostname
    fn detect(base_url: &Url) -> ProviderKind;
}

impl ProviderKindExt for ProviderKind {
    fn detect(base_url: &Url) -> ProviderKind {
        let host = base_url.host_str().unwrap_or_default().to_lowercase();
        if host.ends_with("venice.ai") || host.contains("api.venice.ai") {
            return ProviderKind::Venice;
        }
        if host.ends_with("openrouter.ai") || host.contains("openrouter.ai") {
            return ProviderKind::OpenRouter;
        }
        if host == "api.x.ai" || host.ends_with(".x.ai") {
            return ProviderKind::XAI;
        }
        if host == "api.openai.com" {
            return ProviderKind::OpenAI;
        }
        ProviderKind::Generic
    }
}

/// Model pricing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderModelPricing {
    /// USD per 1M input tokens (if known)
    pub input_usd_per_million: Option<f64>,
    /// USD per 1M output tokens (if known)
    pub output_usd_per_million: Option<f64>,
}

/// Model capabilities and traits
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderModelTraits {
    /// Whether the model supports image/vision inputs
    pub supports_vision: bool,
    /// Whether the model supports function calling / tools
    pub supports_tools: bool,
    /// Whether the model supports chain-of-thought reasoning
    pub supports_reasoning: bool,
    /// Whether the model can browse the web
    pub supports_web_search: bool,
    /// Whether the model supports structured outputs (JSON schema)
    pub supports_response_schema: bool,
    /// Whether the model supports log probabilities
    pub supports_logprobs: bool,
    /// Whether the model is optimized for code generation
    pub code_optimized: bool,
}

/// A discovered model from the provider's /models endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredModel {
    /// The model ID used for API requests
    pub id: String,
    /// Human-readable display name
    pub name: String,
    /// The detected provider kind (e.g. OpenAI, OpenRouter)
    pub provider_kind: ProviderKind,
    /// Context window size in tokens, if known
    pub context_length: Option<u32>,
    /// Capabilities of the model
    pub traits: ProviderModelTraits,
    /// Pricing information
    pub pricing: ProviderModelPricing,
}

/// Full model list response from provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderModelList {
    /// The provider kind that served this list
    pub provider_kind: ProviderKind,
    /// The base URL used for discovery
    pub base_url: String,
    /// Unix timestamp when this list was fetched
    pub fetched_at_unix_s: u64,
    /// List of discovered models
    pub models: Vec<DiscoveredModel>,
}

/// Client for discovering models from OpenAI-compatible providers
#[derive(Debug)]
pub struct ProviderDiscoveryClient {
    http: reqwest::Client,
    base_url: Url,
    provider_kind: ProviderKind,
    headers: HeaderMap,
    api_key: Option<SecretString>,
    timeout: Duration,
    cache: Option<DiskCache>,
}

impl ProviderDiscoveryClient {
    /// Create a new discovery client
    ///
    /// # Arguments
    /// * `base_url_raw` - The provider's base URL
    /// * `headers_raw` - Additional headers (excluding Authorization)
    /// * `api_key` - Optional API key (will be wrapped in Bearer auth)
    /// * `timeout` - Request timeout
    /// * `cache` - Optional disk cache for model lists
    #[instrument(skip(api_key, headers_raw))]
    pub fn new(
        base_url_raw: &str,
        headers_raw: &BTreeMap<String, String>,
        api_key: Option<SecretString>,
        timeout: Duration,
        cache: Option<DiskCache>,
    ) -> Result<Self, DiscoveryError> {
        let base_url = sanitize_base_url(base_url_raw)?;
        let provider_kind = ProviderKind::detect(&base_url);
        info!(%provider_kind, %base_url, "Detected provider kind");

        let mut headers = HeaderMap::new();
        let headers_hash: HashMap<String, String> = headers_raw
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (k, v) in sanitize_headers(&headers_hash)?.into_iter() {
            let name = HeaderName::from_bytes(k.as_bytes())
                .map_err(|_| DiscoveryError::InvalidHeaders(format!("bad header name: {k}")))?;
            let value = HeaderValue::from_str(&v)
                .map_err(|_| DiscoveryError::InvalidHeaders(format!("bad header value for {k}")))?;
            headers.insert(name, value);
        }

        // Prefer bearer auth for OpenAI-compat providers
        if let Some(ref key) = api_key {
            let key_str = key.expose_secret();
            let hv = HeaderValue::from_str(&format!("Bearer {key_str}")).map_err(|_| {
                DiscoveryError::InvalidApiKey("api key contains illegal characters".to_string())
            })?;
            headers.insert(AUTHORIZATION, hv);
        }

        let http = reqwest::Client::builder()
            .timeout(timeout)
            .default_headers(headers.clone())
            .build()
            .map_err(|e| DiscoveryError::Http(e.to_string()))?;

        Ok(Self {
            http,
            base_url,
            provider_kind,
            headers,
            api_key,
            timeout,
            cache,
        })
    }

    /// Get the detected provider kind
    pub fn provider_kind(&self) -> ProviderKind {
        self.provider_kind
    }

    /// Get the normalized base URL
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Get the configured headers
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Check if an API key is configured
    pub fn has_api_key(&self) -> bool {
        self.api_key.is_some()
    }

    /// Get the configured timeout
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Discover chat models from the provider
    ///
    /// Returns only chat models (filters out embeddings, audio, etc.)
    #[instrument(skip(self))]
    pub async fn discover_chat_models(&self) -> Result<ProviderModelList, DiscoveryError> {
        // Check cache first
        if let Some(cache) = &self.cache {
            if let Some(cached) = cache.get_fresh(&self.cache_key())? {
                debug!("Returning cached model list");
                return Ok(cached);
            }
        }

        let mut url = join_path(&self.base_url, "models")
            .map_err(|e| DiscoveryError::InvalidBaseUrl(e.to_string()))?;
        if self.provider_kind == ProviderKind::Venice {
            // Fetch all model types, then filter to text/code locally.
            url.query_pairs_mut().append_pair("type", "all");
        }

        info!(%url, "Fetching models from provider");

        let resp = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| DiscoveryError::Http(e.to_string()))?;
        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| DiscoveryError::Http(e.to_string()))?;

        if !status.is_success() {
            return Err(DiscoveryError::Provider(
                status.as_u16(),
                truncate(&body, 400),
            ));
        }

        let json: Value =
            serde_json::from_str(&body).map_err(|e| DiscoveryError::Json(e.to_string()))?;

        let mut models = parse_models_response(self.provider_kind, &json)?;

        // Text-model-only filter: use explicit model type when present, otherwise fallback to id heuristic.
        let original_count = models.len();
        if self.provider_kind == ProviderKind::Venice {
            models.retain(|m| m.provider_kind == ProviderKind::Venice);
        } else {
            models.retain(|m| is_chat_model_id(&m.id));
        }
        let filtered_count = original_count.saturating_sub(models.len());
        if filtered_count > 0 {
            debug!(filtered_count, "Filtered non-text models");
        }

        models.sort_by(|a, b| a.id.cmp(&b.id));

        let out = ProviderModelList {
            provider_kind: self.provider_kind,
            base_url: self.base_url.to_string(),
            fetched_at_unix_s: unix_now(),
            models,
        };

        // Save to cache
        if let Some(cache) = &self.cache {
            cache.set(&self.cache_key(), &out)?;
        }

        info!(
            model_count = out.models.len(),
            "Successfully discovered models"
        );
        Ok(out)
    }

    fn cache_key(&self) -> String {
        // URL-safe slug, no secrets
        let mut s = format!("{:?}_{}", self.provider_kind, self.base_url);
        s.make_ascii_lowercase();
        let slug: String = s
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        slug.trim_matches('_').to_string()
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    format!("{}…", &s[..max])
}

/// Sanitizes and normalizes an OpenAI-style base URL
///
/// Rules:
/// - Trims whitespace
/// - Rejects control chars / newlines
/// - Parses as URL
/// - Allows http for localhost only; otherwise requires https
/// - Normalizes path to include `/v1` if missing (unless already ends with `/v1`)
pub fn sanitize_base_url(input: &str) -> Result<Url, DiscoveryError> {
    let raw = input.trim();
    if raw.is_empty() {
        return Err(DiscoveryError::InvalidBaseUrl("empty".to_string()));
    }
    if raw.chars().any(|c| c.is_control()) {
        return Err(DiscoveryError::InvalidBaseUrl(
            "contains control characters".to_string(),
        ));
    }
    let mut url = Url::parse(raw).map_err(|e| DiscoveryError::InvalidBaseUrl(e.to_string()))?;

    if url.username() != "" || url.password().is_some() {
        return Err(DiscoveryError::InvalidBaseUrl(
            "must not include userinfo".to_string(),
        ));
    }

    // Drop query/fragment
    url.set_query(None);
    url.set_fragment(None);

    let host = url.host_str().unwrap_or_default().to_lowercase();
    let is_local = host == "localhost" || host == "127.0.0.1" || host == "::1";

    match url.scheme() {
        "https" => {}
        "http" if is_local => {}
        other => {
            return Err(DiscoveryError::InvalidBaseUrl(format!(
                "unsupported scheme: {other}"
            )))
        }
    }

    // Normalize path so join("models") hits `.../v1/models` for typical providers
    let path = url.path().trim_end_matches('/').to_string();
    let normalized_path = if path.is_empty() || path == "/" {
        if host.ends_with("venice.ai") {
            "/api/v1".to_string()
        } else {
            "/v1".to_string()
        }
    } else if path.contains("/openai/deployments/") {
        // Azure uses a different layout; keep it as-is
        path
    } else if host.ends_with("venice.ai") && path == "/v1" {
        "/api/v1".to_string()
    } else if path.ends_with("/api/v1") || path.ends_with("/api/v1/") {
        path.trim_end_matches('/').to_string()
    } else if path.ends_with("/v1") {
        path
    } else {
        format!("{path}/v1")
    };

    url.set_path(&normalized_path);
    Ok(url)
}

/// Sanitize user-configured headers (excluding secrets)
///
/// - Header names must be token-like
/// - Header values must not contain control chars/newlines
pub fn sanitize_headers(
    headers: &HashMap<String, String>,
) -> Result<HashMap<String, String>, DiscoveryError> {
    let mut out = HashMap::new();
    for (k, v) in headers {
        let name = k.trim();
        let val = v.trim();

        if name.is_empty() {
            return Err(DiscoveryError::InvalidHeaders(
                "empty header name".to_string(),
            ));
        }
        if name.eq_ignore_ascii_case("authorization") {
            // Authorization is managed by api_key
            warn!("Ignoring Authorization header - use api_key parameter instead");
            continue;
        }
        if name.chars().any(|c| c.is_control()) || val.chars().any(|c| c.is_control()) {
            return Err(DiscoveryError::InvalidHeaders(format!(
                "control characters in header: {name}"
            )));
        }
        // HTTP header name token validation
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(DiscoveryError::InvalidHeaders(format!(
                "invalid header name: {name}"
            )));
        }
        out.insert(name.to_string(), val.to_string());
    }
    Ok(out)
}

/// Heuristic filter to identify chat models vs embeddings/audio/etc
fn is_chat_model_id(id: &str) -> bool {
    let s = id.to_lowercase();
    // Fast denylist
    let deny = [
        "embedding",
        "embed",
        "whisper",
        "tts",
        "audio",
        "transcribe",
        "moderation",
        "realtime",
        "image",
        "vision-preview",
        "dall-e",
        "speech",
        "asr",
        "vision",
        "video",
        "rerank",
        "rank",
        "ocr",
        "inpaint",
        "upscale",
        "tokenizer",
    ];
    if deny.iter().any(|d| s.contains(d)) {
        return false;
    }
    true
}

fn parse_models_response(
    kind: ProviderKind,
    v: &Value,
) -> Result<Vec<DiscoveredModel>, DiscoveryError> {
    // The decision is based on actual JSON shape rather than the detected provider_kind
    let data = v
        .get("data")
        .and_then(|d| d.as_array())
        .ok_or_else(|| DiscoveryError::Json("missing or invalid 'data' array".to_string()))?;

    let mut out = Vec::new();
    for item in data {
        if let Some(model) = parse_model_item(kind, item)? {
            out.push(model);
        }
    }
    Ok(out)
}

fn parse_model_item(
    kind: ProviderKind,
    item: &Value,
) -> Result<Option<DiscoveredModel>, DiscoveryError> {
    if item.get("model_spec").is_none() {
        if let Some(model_type) = extract_model_type(item) {
            if !is_text_model_type(&model_type) {
                return Ok(None);
            }
        }
    }

    let id = item
        .get("id")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string());

    let id = match id {
        Some(id) => id,
        None => return Ok(None),
    };

    // Venice schema has model_spec and type
    if item.get("model_spec").is_some() {
        if let Some(model_type) = item.get("type").and_then(|x| x.as_str()) {
            if model_type != "text" && model_type != "code" {
                return Ok(None);
            }
        }
        let model_spec = item.get("model_spec").unwrap_or(&Value::Null);
        let name = model_spec
            .get("name")
            .and_then(|x| x.as_str())
            .unwrap_or(&id)
            .to_string();
        let ctx = model_spec
            .get("availableContextTokens")
            .and_then(|x| x.as_u64())
            .map(|n| n as u32);

        let caps = model_spec.get("capabilities").unwrap_or(&Value::Null);
        let traits = ProviderModelTraits {
            supports_vision: caps
                .get("supportsVision")
                .and_then(|x| x.as_bool())
                .unwrap_or(false),
            supports_tools: caps
                .get("supportsFunctionCalling")
                .and_then(|x| x.as_bool())
                .unwrap_or(false),
            supports_reasoning: caps
                .get("supportsReasoning")
                .and_then(|x| x.as_bool())
                .unwrap_or(false),
            supports_web_search: caps
                .get("supportsWebSearch")
                .and_then(|x| x.as_bool())
                .unwrap_or(false),
            supports_response_schema: caps
                .get("supportsResponseSchema")
                .and_then(|x| x.as_bool())
                .unwrap_or(false),
            supports_logprobs: caps
                .get("supportsLogProbs")
                .and_then(|x| x.as_bool())
                .unwrap_or(false),
            code_optimized: caps
                .get("optimizedForCode")
                .and_then(|x| x.as_bool())
                .unwrap_or(false),
        };

        let pricing = extract_venice_pricing(model_spec);

        return Ok(Some(DiscoveredModel {
            id,
            name,
            provider_kind: ProviderKind::Venice,
            context_length: ctx,
            traits,
            pricing,
        }));
    }

    // OpenRouter schema has context_length and pricing at top-level
    if item.get("context_length").is_some() || item.get("pricing").is_some() {
        let name = item
            .get("name")
            .and_then(|x| x.as_str())
            .unwrap_or(&id)
            .to_string();
        let ctx = item
            .get("context_length")
            .and_then(|x| x.as_u64())
            .map(|n| n as u32);
        let pricing = extract_openrouter_pricing(item);

        return Ok(Some(DiscoveredModel {
            id,
            name,
            provider_kind: ProviderKind::OpenRouter,
            context_length: ctx,
            traits: ProviderModelTraits::default(),
            pricing,
        }));
    }

    // OpenAI/xAI generic schema: only id is reliable
    Ok(Some(DiscoveredModel {
        name: id.clone(),
        id,
        provider_kind: kind,
        context_length: None,
        traits: ProviderModelTraits::default(),
        pricing: ProviderModelPricing {
            input_usd_per_million: None,
            output_usd_per_million: None,
        },
    }))
}

fn extract_model_type(item: &Value) -> Option<String> {
    let keys = ["type", "category", "modality", "model_type"];
    for key in keys {
        if let Some(value) = item.get(key).and_then(|v| v.as_str()) {
            return Some(value.to_lowercase());
        }
    }
    None
}

fn is_text_model_type(model_type: &str) -> bool {
    matches!(
        model_type,
        "text" | "chat" | "code" | "completion" | "llm"
    )
}

fn join_path(base: &Url, segment: &str) -> Result<Url, url::ParseError> {
    let mut url = base.clone();
    let mut path = url.path().to_string();
    if !path.ends_with('/') {
        path.push('/');
    }
    path.push_str(segment);
    url.set_path(&path);
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

fn extract_venice_pricing(model_spec: &Value) -> ProviderModelPricing {
    let mut out = ProviderModelPricing {
        input_usd_per_million: None,
        output_usd_per_million: None,
    };
    let pricing = model_spec.get("pricing").unwrap_or(&Value::Null);

    if let Some(inp) = pricing
        .get("input")
        .and_then(|x| x.get("usd"))
        .and_then(|x| x.as_f64())
    {
        out.input_usd_per_million = Some(inp);
    }
    if let Some(outp) = pricing
        .get("output")
        .and_then(|x| x.get("usd"))
        .and_then(|x| x.as_f64())
    {
        out.output_usd_per_million = Some(outp);
    }
    out
}

fn extract_openrouter_pricing(item: &Value) -> ProviderModelPricing {
    // OpenRouter pricing values are typically strings, e.g. "0.000002" per token
    // Convert to USD per 1M tokens
    let mut out = ProviderModelPricing {
        input_usd_per_million: None,
        output_usd_per_million: None,
    };

    let pricing = item.get("pricing").unwrap_or(&Value::Null);
    let prompt = pricing
        .get("prompt")
        .and_then(|x| x.as_str())
        .and_then(|s| s.parse::<f64>().ok());
    let completion = pricing
        .get("completion")
        .and_then(|x| x.as_str())
        .and_then(|s| s.parse::<f64>().ok());

    if let Some(p) = prompt {
        out.input_usd_per_million = Some(p * 1_000_000.0);
    }
    if let Some(c) = completion {
        out.output_usd_per_million = Some(c * 1_000_000.0);
    }
    out
}

/// Errors that can occur during provider discovery
#[derive(Debug, Error)]
pub enum DiscoveryError {
    /// Invalid base URL configuration
    #[error("invalid base_url: {0}")]
    InvalidBaseUrl(String),
    /// Invalid headers configuration
    #[error("invalid headers: {0}")]
    InvalidHeaders(String),
    /// Invalid API key
    #[error("invalid api key: {0}")]
    InvalidApiKey(String),
    /// HTTP request failed
    #[error("http error: {0}")]
    Http(String),
    /// JSON parsing failed
    #[error("json error: {0}")]
    Json(String),
    /// Provider returned an error response
    #[error("provider error {0}: {1}")]
    Provider(u16, String),
    /// Cache operation failed
    #[error("cache error: {0}")]
    Cache(String),
}

/// Disk cache for model lists
///
/// Mirrors the Python script's intent (avoid repeated /models calls) but stays minimal
#[derive(Debug, Clone)]
pub struct DiskCache {
    /// Directory to store cache files
    pub dir: PathBuf,
    /// Time-to-live for fresh results (default: 5 mins)
    pub fresh_ttl: Duration,
    /// Time-to-live for stale results (default: 24h)
    pub stale_ttl: Duration,
}

impl Default for DiskCache {
    fn default() -> Self {
        let mut dir = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
        dir.push("hqe-workbench");
        dir.push("model-cache");
        Self {
            dir,
            fresh_ttl: Duration::from_secs(300),   // 5 minutes
            stale_ttl: Duration::from_secs(86400), // 24 hours
        }
    }
}

impl DiskCache {
    fn path(&self, key: &str) -> PathBuf {
        let mut p = self.dir.clone();
        p.push(format!("{key}.json"));
        p
    }

    /// Get a cached entry if it's still fresh (within fresh_ttl)
    pub fn get_fresh(&self, key: &str) -> Result<Option<ProviderModelList>, DiscoveryError> {
        self.get_within(key, self.fresh_ttl)
    }

    /// Get a cached entry even if stale (within stale_ttl)
    pub fn get_stale(&self, key: &str) -> Result<Option<ProviderModelList>, DiscoveryError> {
        self.get_within(key, self.stale_ttl)
    }

    fn get_within(
        &self,
        key: &str,
        ttl: Duration,
    ) -> Result<Option<ProviderModelList>, DiscoveryError> {
        let p = self.path(key);
        if !p.exists() {
            return Ok(None);
        }
        let meta = fs::metadata(&p).map_err(|e| DiscoveryError::Cache(e.to_string()))?;
        let mtime = meta
            .modified()
            .map_err(|e| DiscoveryError::Cache(e.to_string()))?;
        let age = SystemTime::now()
            .duration_since(mtime)
            .unwrap_or(Duration::MAX);
        if age > ttl {
            return Ok(None);
        }
        let s = fs::read_to_string(&p).map_err(|e| DiscoveryError::Cache(e.to_string()))?;
        let v: ProviderModelList =
            serde_json::from_str(&s).map_err(|e| DiscoveryError::Cache(e.to_string()))?;
        Ok(Some(v))
    }

    /// Store a model list in the cache
    pub fn set(&self, key: &str, value: &ProviderModelList) -> Result<(), DiscoveryError> {
        fs::create_dir_all(&self.dir).map_err(|e| DiscoveryError::Cache(e.to_string()))?;
        let p = self.path(key);
        let s = serde_json::to_string(value).map_err(|e| DiscoveryError::Cache(e.to_string()))?;
        fs::write(p, s).map_err(|e| DiscoveryError::Cache(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_base_url_normalizes_v1() -> anyhow::Result<()> {
        let u = sanitize_base_url("https://api.openai.com")?;
        assert_eq!(u.as_str(), "https://api.openai.com/v1");

        let u = sanitize_base_url("https://openrouter.ai/api/v1/")?;
        assert_eq!(u.as_str(), "https://openrouter.ai/api/v1");
        Ok(())
    }

    #[test]
    fn sanitize_base_url_allows_localhost_http() -> anyhow::Result<()> {
        let u = sanitize_base_url("http://localhost:1234")?;
        assert_eq!(u.as_str(), "http://localhost:1234/v1");
        Ok(())
    }

    #[test]
    fn sanitize_base_url_rejects_http_for_non_local() {
        assert!(sanitize_base_url("http://api.openai.com").is_err());
    }

    #[test]
    fn sanitize_headers_rejects_newlines() {
        let mut h = HashMap::new();
        h.insert("X-Test".to_string(), "ok\nno".to_string());
        assert!(sanitize_headers(&h).is_err());
    }

    #[test]
    fn sanitize_headers_skips_authorization() -> anyhow::Result<()> {
        let mut h = HashMap::new();
        h.insert("Authorization".to_string(), "Bearer test".to_string());
        h.insert("X-Custom".to_string(), "value".to_string());
        let result = sanitize_headers(&h)?;
        assert!(!result.contains_key("Authorization"));
        assert_eq!(result.get("X-Custom"), Some(&"value".to_string()));
        Ok(())
    }

    #[test]
    fn chat_filter_drops_embeddings() {
        assert!(!is_chat_model_id("text-embedding-3-small"));
        assert!(!is_chat_model_id("whisper-1"));
        assert!(!is_chat_model_id("tts-1"));
        assert!(is_chat_model_id("gpt-4o-mini"));
        assert!(is_chat_model_id("claude-3-opus"));
    }

    #[test]
    fn provider_kind_detection() -> anyhow::Result<()> {
        let venice = Url::parse("https://api.venice.ai/v1")?;
        assert_eq!(ProviderKind::detect(&venice), ProviderKind::Venice);

        let openrouter = Url::parse("https://openrouter.ai/api/v1")?;
        assert_eq!(ProviderKind::detect(&openrouter), ProviderKind::OpenRouter);

        let xai = Url::parse("https://api.x.ai/v1")?;
        assert_eq!(ProviderKind::detect(&xai), ProviderKind::XAI);

        let openai = Url::parse("https://api.openai.com/v1")?;
        assert_eq!(ProviderKind::detect(&openai), ProviderKind::OpenAI);

        let generic = Url::parse("https://custom.example.com/v1")?;
        assert_eq!(ProviderKind::detect(&generic), ProviderKind::Generic);
        Ok(())
    }

    #[test]
    fn disk_cache_default_path() {
        let cache = DiskCache::default();
        assert!(cache.dir.to_string_lossy().contains("hqe-workbench"));
        assert!(cache.dir.to_string_lossy().contains("model-cache"));
    }

    #[test]
    fn test_parse_model_item_venice_schema() -> anyhow::Result<()> {
        let json = serde_json::json!({
            "id": "venice-model-1",
            "model_spec": {
                "name": "Venice Model",
                "availableContextTokens": 8192,
                "capabilities": {
                    "supportsVision": true,
                    "supportsFunctionCalling": true,
                    "supportsReasoning": false,
                    "supportsWebSearch": true,
                    "supportsResponseSchema": true,
                    "optimizedForCode": false
                },
                "pricing": {
                    "input": {"usd": 0.000002},
                    "output": {"usd": 0.000006}
                }
            }
        });

        let result = parse_model_item(ProviderKind::Generic, &json)?;
        assert!(result.is_some());
        let model = result.unwrap();
        assert_eq!(model.id, "venice-model-1");
        assert_eq!(model.name, "Venice Model");
        assert_eq!(model.provider_kind, ProviderKind::Venice);
        assert_eq!(model.context_length, Some(8192));
        assert!(model.traits.supports_vision);
        assert!(model.traits.supports_tools);
        assert!(!model.traits.supports_reasoning);
        // Venice pricing is already in USD per million tokens
        let input_val = model.pricing.input_usd_per_million.unwrap();
        let output_val = model.pricing.output_usd_per_million.unwrap();
        assert!(
            (input_val - 0.000002).abs() < 0.0000001,
            "Expected ~0.000002, got {}",
            input_val
        );
        assert!(
            (output_val - 0.000006).abs() < 0.0000001,
            "Expected ~0.000006, got {}",
            output_val
        );
        Ok(())
    }

    #[test]
    fn test_parse_model_item_openrouter_schema() -> anyhow::Result<()> {
        let json = serde_json::json!({
            "id": "openrouter-model-1",
            "name": "OpenRouter Model",
            "context_length": 4096,
            "pricing": {
                "prompt": "0.000001",
                "completion": "0.000003"
            }
        });

        let result = parse_model_item(ProviderKind::Generic, &json)?;
        assert!(result.is_some());
        let model = result.unwrap();
        assert_eq!(model.id, "openrouter-model-1");
        assert_eq!(model.name, "OpenRouter Model");
        assert_eq!(model.provider_kind, ProviderKind::OpenRouter);
        assert_eq!(model.context_length, Some(4096));
        // Compare with tolerance for floating point precision
        let input_val = model.pricing.input_usd_per_million.unwrap();
        let output_val = model.pricing.output_usd_per_million.unwrap();
        assert!(
            (input_val - 1.0).abs() < 0.1,
            "Expected ~1.0, got {}",
            input_val
        );
        assert!(
            (output_val - 3.0).abs() < 0.1,
            "Expected ~3.0, got {}",
            output_val
        );
        Ok(())
    }

    #[test]
    fn test_parse_model_item_generic_schema() -> anyhow::Result<()> {
        let json = serde_json::json!({
            "id": "generic-model-1"
        });

        let result = parse_model_item(ProviderKind::OpenAI, &json)?;
        assert!(result.is_some());
        let model = result.unwrap();
        assert_eq!(model.id, "generic-model-1");
        assert_eq!(model.name, "generic-model-1");
        assert_eq!(model.provider_kind, ProviderKind::OpenAI);
        assert_eq!(model.context_length, None);
        Ok(())
    }

    #[test]
    fn test_parse_model_item_invalid() -> anyhow::Result<()> {
        let json = serde_json::json!({
            "name": "some-name"
            // Missing "id" field
        });

        let result = parse_model_item(ProviderKind::OpenAI, &json)?;
        assert!(result.is_none());
        Ok(())
    }

    #[test]
    fn test_sanitize_base_url_various_inputs() {
        // Test valid URLs
        assert!(sanitize_base_url("https://api.openai.com").is_ok());
        assert!(sanitize_base_url("https://openrouter.ai/api/v1/").is_ok());
        assert!(sanitize_base_url("http://localhost:1234").is_ok());

        // Test invalid URLs
        assert!(sanitize_base_url("").is_err());
        assert!(sanitize_base_url("not-a-url").is_err());
        assert!(sanitize_base_url("ftp://example.com").is_err()); // Unsupported scheme
        assert!(sanitize_base_url("https://user:pass@api.example.com").is_err());
        // Contains credentials
    }

    #[test]
    fn test_is_chat_model_id_filtering() {
        // Should be rejected (non-chat models)
        assert!(!is_chat_model_id("text-embedding-ada-002"));
        assert!(!is_chat_model_id("text-embedding-3-small"));
        assert!(!is_chat_model_id("whisper-1"));
        assert!(!is_chat_model_id("tts-1"));
        assert!(!is_chat_model_id("dall-e-2"));

        // Should be accepted (chat models)
        assert!(is_chat_model_id("gpt-4"));
        assert!(is_chat_model_id("gpt-4o-mini"));
        assert!(is_chat_model_id("claude-3-opus"));
        assert!(is_chat_model_id("llama-3.1"));
    }
}
