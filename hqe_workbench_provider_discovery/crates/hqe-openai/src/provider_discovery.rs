use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use url::Url;

/// Providers that are "OpenAI compatible" but differ in model list schema and/or niceties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    OpenAI,
    Venice,
    OpenRouter,
    XAI,
    Generic,
}

impl ProviderKind {
    pub fn detect(base_url: &Url) -> ProviderKind {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderModelPricing {
    /// USD per 1M input tokens (if known).
    pub input_usd_per_million: Option<f64>,
    /// USD per 1M output tokens (if known).
    pub output_usd_per_million: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderModelTraits {
    pub supports_vision: bool,
    pub supports_tools: bool,
    pub supports_reasoning: bool,
    pub supports_web_search: bool,
    pub supports_response_schema: bool,
    pub code_optimized: bool,
}

impl Default for ProviderModelTraits {
    fn default() -> Self {
        Self {
            supports_vision: false,
            supports_tools: false,
            supports_reasoning: false,
            supports_web_search: false,
            supports_response_schema: false,
            code_optimized: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredModel {
    pub id: String,
    pub name: String,
    pub provider_kind: ProviderKind,
    pub context_length: Option<u32>,
    pub traits: ProviderModelTraits,
    pub pricing: ProviderModelPricing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderModelList {
    pub provider_kind: ProviderKind,
    pub base_url: String,
    pub fetched_at_unix_s: u64,
    pub models: Vec<DiscoveredModel>,
}

#[derive(Debug)]
pub struct ProviderDiscoveryClient {
    http: reqwest::Client,
    base_url: Url,
    provider_kind: ProviderKind,
    headers: HeaderMap,
    api_key: Option<String>,
    timeout: Duration,
    cache: Option<DiskCache>,
}

impl ProviderDiscoveryClient {
    pub fn new(
        base_url_raw: &str,
        headers_raw: &BTreeMap<String, String>,
        api_key: Option<String>,
        timeout: Duration,
        cache: Option<DiskCache>,
    ) -> Result<Self, DiscoveryError> {
        let base_url = sanitize_base_url(base_url_raw)?;
        let provider_kind = ProviderKind::detect(&base_url);

        let mut headers = HeaderMap::new();
        for (k, v) in sanitize_headers(headers_raw)?.into_iter() {
            let name = HeaderName::from_bytes(k.as_bytes())
                .map_err(|_| DiscoveryError::InvalidHeaders(format!("bad header name: {k}")))?;
            let value = HeaderValue::from_str(&v)
                .map_err(|_| DiscoveryError::InvalidHeaders(format!("bad header value for {k}")))?;
            headers.insert(name, value);
        }

        // Prefer bearer auth for OpenAI-compat providers.
        if let Some(ref key) = api_key {
            let hv = HeaderValue::from_str(&format!("Bearer {key}"))
                .map_err(|_| DiscoveryError::InvalidApiKey("api key contains illegal characters".to_string()))?;
            headers.insert(AUTHORIZATION, hv);
        }

        let http = reqwest::Client::builder()
            .timeout(timeout)
            .default_headers(headers.clone())
            .build()
            .map_err(|e| DiscoveryError::Http(e.to_string()))?;

        Ok(Self { http, base_url, provider_kind, headers, api_key, timeout, cache })
    }

    pub fn provider_kind(&self) -> ProviderKind { self.provider_kind }

    pub async fn discover_chat_models(&self) -> Result<ProviderModelList, DiscoveryError> {
        // Cache key is a sanitized slug derived from base_url + provider kind.
        if let Some(cache) = &self.cache {
            if let Some(cached) = cache.get_fresh(&self.cache_key())? {
                return Ok(cached);
            }
        }

        let url = self.base_url.join("models").map_err(|e| DiscoveryError::InvalidBaseUrl(e.to_string()))?;
        let resp = self.http.get(url).send().await.map_err(|e| DiscoveryError::Http(e.to_string()))?;
        let status = resp.status();
        let body = resp.text().await.map_err(|e| DiscoveryError::Http(e.to_string()))?;

        if !status.is_success() {
            return Err(DiscoveryError::Provider(status.as_u16(), truncate(&body, 400)));
        }

        let json: Value = serde_json::from_str(&body).map_err(|e| DiscoveryError::Json(e.to_string()))?;

        let mut models = parse_models_response(self.provider_kind, &json)?;
        // Chat-model-only filter (heuristic where schema is ambiguous).
        models.retain(|m| is_chat_model_id(&m.id));

        models.sort_by(|a, b| a.id.cmp(&b.id));

        let out = ProviderModelList {
            provider_kind: self.provider_kind,
            base_url: self.base_url.to_string(),
            fetched_at_unix_s: unix_now(),
            models,
        };

        if let Some(cache) = &self.cache {
            cache.set(&self.cache_key(), &out)?;
        }

        Ok(out)
    }

    fn cache_key(&self) -> String {
        // URL-safe slug, no secrets.
        let mut s = format!("{:?}_{}", self.provider_kind, self.base_url);
        s.make_ascii_lowercase();
        let slug: String = s.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
        slug.trim_matches('_').to_string()
    }
}

fn unix_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { return s.to_string(); }
    format!("{}…", &s[..max])
}

/// Sanitizes and normalizes an OpenAI-style base URL.
///
/// Rules:
/// - trims whitespace
/// - rejects control chars / newlines
/// - parses as URL
/// - allows http for localhost only; otherwise requires https
/// - normalizes path to include `/v1` if missing (unless already ends with `/v1`)
pub fn sanitize_base_url(input: &str) -> Result<Url, DiscoveryError> {
    let raw = input.trim();
    if raw.is_empty() {
        return Err(DiscoveryError::InvalidBaseUrl("empty".to_string()));
    }
    if raw.chars().any(|c| c.is_control()) {
        return Err(DiscoveryError::InvalidBaseUrl("contains control characters".to_string()));
    }
    let mut url = Url::parse(raw).map_err(|e| DiscoveryError::InvalidBaseUrl(e.to_string()))?;

    if url.username() != "" || url.password().is_some() {
        return Err(DiscoveryError::InvalidBaseUrl("must not include userinfo".to_string()));
    }

    // Drop query/fragment.
    url.set_query(None);
    url.set_fragment(None);

    let host = url.host_str().unwrap_or_default().to_lowercase();
    let is_local = host == "localhost" || host == "127.0.0.1" || host == "::1";

    match url.scheme() {
        "https" => {}
        "http" if is_local => {}
        other => return Err(DiscoveryError::InvalidBaseUrl(format!("unsupported scheme: {other}"))),
    }

    // Normalize path so join("models") hits `.../v1/models` for typical providers.
    let path = url.path().trim_end_matches('/').to_string();
    let normalized_path = if path.is_empty() || path == "/" {
        "/v1".to_string()
    } else if path.ends_with("/v1") {
        path
    } else if path.contains("/openai/deployments/") {
        // Azure uses a different layout; keep it as-is.
        path
    } else if path.ends_with("/api/v1") || path.ends_with("/api/v1/") {
        path.trim_end_matches('/').to_string()
    } else if path.ends_with("/api/v1") {
        path
    } else if path.ends_with("/v1") {
        path
    } else {
        format!("{path}/v1")
    };

    url.set_path(&normalized_path);
    Ok(url)
}

/// Sanitize user-configured headers (excluding secrets).
///
/// - header names must be token-like
/// - header values must not contain control chars/newlines
pub fn sanitize_headers(headers: &BTreeMap<String, String>) -> Result<BTreeMap<String, String>, DiscoveryError> {
    let mut out = BTreeMap::new();
    for (k, v) in headers {
        let name = k.trim();
        let val = v.trim();

        if name.is_empty() {
            return Err(DiscoveryError::InvalidHeaders("empty header name".to_string()));
        }
        if name.eq_ignore_ascii_case("authorization") {
            // Authorization is managed by api_key.
            continue;
        }
        if name.chars().any(|c| c.is_control()) || val.chars().any(|c| c.is_control()) {
            return Err(DiscoveryError::InvalidHeaders(format!("control characters in header: {name}")));
        }
        // Very small allowlist-ish: HTTP header name token.
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' ) {
            return Err(DiscoveryError::InvalidHeaders(format!("invalid header name: {name}")));
        }
        out.insert(name.to_string(), val.to_string());
    }
    Ok(out)
}

fn is_chat_model_id(id: &str) -> bool {
    let s = id.to_lowercase();
    // Fast denylist. This is heuristic by necessity for generic OpenAI-style /models.
    let deny = [
        "embedding", "embed", "whisper", "tts", "audio", "transcribe", "moderation", "realtime",
        "image", "vision-preview", "dall-e", "speech", "asr",
    ];
    if deny.iter().any(|d| s.contains(d)) {
        return false;
    }
    true
}

fn parse_models_response(kind: ProviderKind, v: &Value) -> Result<Vec<DiscoveredModel>, DiscoveryError> {
    // The decision is based on actual JSON shape rather than the detected provider_kind.
    // This makes "api injection" robust even if the user points Venice at a custom domain etc.
    let data = v.get("data").and_then(|d| d.as_array()).ok_or_else(|| {
        DiscoveryError::Json("missing or invalid 'data' array".to_string())
    })?;

    let mut out = Vec::new();
    for item in data {
        if let Some(model) = parse_model_item(kind, item)? {
            out.push(model);
        }
    }
    Ok(out)
}

fn parse_model_item(kind: ProviderKind, item: &Value) -> Result<Option<DiscoveredModel>, DiscoveryError> {
    let id = item.get("id").and_then(|x| x.as_str()).map(|s| s.to_string());
    if id.is_none() { return Ok(None); }
    let id = id.unwrap();

    // Venice schema has model_spec and type
    if item.get("model_spec").is_some() {
        let model_spec = item.get("model_spec").unwrap_or(&Value::Null);
        let name = model_spec.get("name").and_then(|x| x.as_str()).unwrap_or(&id).to_string();
        let ctx = model_spec.get("availableContextTokens").and_then(|x| x.as_u64()).map(|n| n as u32);

        let caps = model_spec.get("capabilities").unwrap_or(&Value::Null);
        let traits = ProviderModelTraits {
            supports_vision: caps.get("supportsVision").and_then(|x| x.as_bool()).unwrap_or(false),
            supports_tools: caps.get("supportsFunctionCalling").and_then(|x| x.as_bool()).unwrap_or(false),
            supports_reasoning: caps.get("supportsReasoning").and_then(|x| x.as_bool()).unwrap_or(false),
            supports_web_search: caps.get("supportsWebSearch").and_then(|x| x.as_bool()).unwrap_or(false),
            supports_response_schema: caps.get("supportsResponseSchema").and_then(|x| x.as_bool()).unwrap_or(false),
            code_optimized: caps.get("optimizedForCode").and_then(|x| x.as_bool()).unwrap_or(false),
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

    // OpenRouter schema has context_length and pricing at top-level.
    if item.get("context_length").is_some() || item.get("pricing").is_some() {
        let name = item.get("name").and_then(|x| x.as_str()).unwrap_or(&id).to_string();
        let ctx = item.get("context_length").and_then(|x| x.as_u64()).map(|n| n as u32);
        let pricing = extract_openrouter_pricing(item);

        // OpenRouter doesn't expose Venice-style capabilities in /models;
        // treat as unknown/false.
        return Ok(Some(DiscoveredModel {
            id,
            name,
            provider_kind: ProviderKind::OpenRouter,
            context_length: ctx,
            traits: ProviderModelTraits::default(),
            pricing,
        }));
    }

    // OpenAI/xAI generic schema:
    // - only id is reliable
    // - no context_length
    // - no pricing
    Ok(Some(DiscoveredModel {
        name: id.clone(),
        id,
        provider_kind: kind,
        context_length: None,
        traits: ProviderModelTraits::default(),
        pricing: ProviderModelPricing { input_usd_per_million: None, output_usd_per_million: None },
    }))
}

fn extract_venice_pricing(model_spec: &Value) -> ProviderModelPricing {
    let mut out = ProviderModelPricing { input_usd_per_million: None, output_usd_per_million: None };
    let pricing = model_spec.get("pricing").unwrap_or(&Value::Null);

    if let Some(inp) = pricing.get("input").and_then(|x| x.get("usd")).and_then(|x| x.as_f64()) {
        out.input_usd_per_million = Some(inp);
    }
    if let Some(outp) = pricing.get("output").and_then(|x| x.get("usd")).and_then(|x| x.as_f64()) {
        out.output_usd_per_million = Some(outp);
    }
    out
}

fn extract_openrouter_pricing(item: &Value) -> ProviderModelPricing {
    // OpenRouter pricing values are typically strings, e.g. "0.000002" per token.
    // We convert to USD per 1M tokens when possible.
    let mut out = ProviderModelPricing { input_usd_per_million: None, output_usd_per_million: None };

    let pricing = item.get("pricing").unwrap_or(&Value::Null);
    let prompt = pricing.get("prompt").and_then(|x| x.as_str()).and_then(|s| s.parse::<f64>().ok());
    let completion = pricing.get("completion").and_then(|x| x.as_str()).and_then(|s| s.parse::<f64>().ok());

    if let Some(p) = prompt {
        out.input_usd_per_million = Some(p * 1_000_000.0);
    }
    if let Some(c) = completion {
        out.output_usd_per_million = Some(c * 1_000_000.0);
    }
    out
}

#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("invalid base_url: {0}")]
    InvalidBaseUrl(String),
    #[error("invalid headers: {0}")]
    InvalidHeaders(String),
    #[error("invalid api key: {0}")]
    InvalidApiKey(String),
    #[error("http error: {0}")]
    Http(String),
    #[error("json error: {0}")]
    Json(String),
    #[error("provider error {0}: {1}")]
    Provider(u16, String),
    #[error("cache error: {0}")]
    Cache(String),
}

/// Disk cache for model lists.
/// This mirrors the Python script's intent (avoid repeated /models calls) but stays minimal.
#[derive(Debug, Clone)]
pub struct DiskCache {
    pub dir: PathBuf,
    pub fresh_ttl: Duration,
    pub stale_ttl: Duration,
}

impl Default for DiskCache {
    fn default() -> Self {
        let mut dir = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
        dir.push("hqe-workbench");
        dir.push("model-cache");
        Self {
            dir,
            fresh_ttl: Duration::from_secs(300),
            stale_ttl: Duration::from_secs(86400),
        }
    }
}

impl DiskCache {
    fn path(&self, key: &str) -> PathBuf {
        let mut p = self.dir.clone();
        p.push(format!("{key}.json"));
        p
    }

    pub fn get_fresh(&self, key: &str) -> Result<Option<ProviderModelList>, DiscoveryError> {
        self.get_within(key, self.fresh_ttl)
    }

    fn get_within(&self, key: &str, ttl: Duration) -> Result<Option<ProviderModelList>, DiscoveryError> {
        let p = self.path(key);
        if !p.exists() {
            return Ok(None);
        }
        let meta = fs::metadata(&p).map_err(|e| DiscoveryError::Cache(e.to_string()))?;
        let mtime = meta.modified().map_err(|e| DiscoveryError::Cache(e.to_string()))?;
        let age = SystemTime::now().duration_since(mtime).unwrap_or(Duration::MAX);
        if age > ttl {
            return Ok(None);
        }
        let s = fs::read_to_string(&p).map_err(|e| DiscoveryError::Cache(e.to_string()))?;
        let v: ProviderModelList = serde_json::from_str(&s).map_err(|e| DiscoveryError::Cache(e.to_string()))?;
        Ok(Some(v))
    }

    pub fn get_stale(&self, key: &str) -> Result<Option<ProviderModelList>, DiscoveryError> {
        self.get_within(key, self.stale_ttl)
    }

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
    fn sanitize_base_url_normalizes_v1() {
        let u = sanitize_base_url("https://api.openai.com").unwrap();
        assert_eq!(u.as_str(), "https://api.openai.com/v1");
        let u = sanitize_base_url("https://openrouter.ai/api/v1/").unwrap();
        assert_eq!(u.as_str(), "https://openrouter.ai/api/v1");
    }

    #[test]
    fn sanitize_headers_rejects_newlines() {
        let mut h = BTreeMap::new();
        h.insert("X-Test".to_string(), "ok\nno".to_string());
        assert!(sanitize_headers(&h).is_err());
    }

    #[test]
    fn chat_filter_drops_embeddings() {
        assert!(!is_chat_model_id("text-embedding-3-small"));
        assert!(is_chat_model_id("gpt-4o-mini"));
    }
}
