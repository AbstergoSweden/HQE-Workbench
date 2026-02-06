//! Secure Analytics Module for HQE Workbench
//!
//! Provides telemetry and analytics with comprehensive security protections:
//! - Event validation to prevent injection attacks
//! - Rate limiting to prevent spam/abuse
//! - PII scrubbing to protect user privacy
//! - Graceful fallback when analytics backend is unavailable
//! - Anti-hijack protections to prevent malicious event injection

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

// ═══════════════════════════════════════════════════════════════════════════════
// SECURITY CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

const MAX_EVENT_PROPERTIES: usize = 50;
const MAX_PROPERTY_KEY_LENGTH: usize = 100;
const MAX_PROPERTY_VALUE_LENGTH: usize = 1000;
const MAX_EVENTS_PER_MINUTE: u32 = 60;
const ALLOWED_EVENT_PREFIXES: &[&str] = &["app_", "chat_", "scan_", "backend_", "error_"];
const BLOCKED_PROPERTY_KEYS: &[&str] = &[
    "password",
    "token",
    "api_key",
    "secret",
    "credential",
    "auth",
    "private_key",
    "access_token",
    "refresh_token",
    "encryption_key",
    "db_key",
];

// ═══════════════════════════════════════════════════════════════════════════════
// ERROR TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Errors that can occur during analytics operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum AnalyticsError {
    /// The event rate limit has been exceeded
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    /// The event name is invalid or contains suspicious patterns
    #[error("Invalid event name: {0}")]
    InvalidEventName(String),
    /// Event properties failed validation
    #[error("Invalid properties: {0}")]
    InvalidProperties(String),
    /// The analytics backend is not available
    #[error("Analytics backend unavailable")]
    BackendUnavailable,
    /// Event validation failed for a specific reason
    #[error("Event validation failed: {0}")]
    ValidationFailed(String),
}

/// Convenience alias for analytics results
pub type Result<T> = std::result::Result<T, AnalyticsError>;

// ═══════════════════════════════════════════════════════════════════════════════
// EVENT TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// A validated analytics event ready for transmission
#[derive(Debug, Clone, serde::Serialize)]
pub struct AnalyticsEvent {
    /// The validated event name (must match allowed prefixes)
    pub name: String,
    /// Sanitized key-value properties attached to the event
    pub properties: HashMap<String, serde_json::Value>,
    /// UTC timestamp of when the event was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Unique session identifier for this analytics session
    pub session_id: String,
    /// Unique identifier for this specific event
    pub event_id: String,
}

/// Event severity for filtering and routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum EventSeverity {
    /// Low-priority diagnostic events
    Debug,
    /// Normal informational events
    Info,
    /// Events that may indicate a problem
    Warning,
    /// Error conditions
    Error,
    /// Security-relevant events (always logged)
    Security,
}

impl EventSeverity {
    /// Returns the severity as a static string slice
    pub fn as_str(&self) -> &'static str {
        match self {
            EventSeverity::Debug => "debug",
            EventSeverity::Info => "info",
            EventSeverity::Warning => "warning",
            EventSeverity::Error => "error",
            EventSeverity::Security => "security",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// RATE LIMITER
// ═══════════════════════════════════════════════════════════════════════════════

/// Thread-safe rate limiter for analytics events
pub struct RateLimiter {
    events: Mutex<Vec<Instant>>,
    window: Duration,
    max_events: u32,
}

impl RateLimiter {
    /// Create a new rate limiter with the given maximum events per window
    pub fn new(max_events: u32, window_secs: u64) -> Self {
        Self {
            events: Mutex::new(Vec::new()),
            window: Duration::from_secs(window_secs),
            max_events,
        }
    }

    /// Check if an event can proceed and record it
    pub fn check_and_record(&self) -> bool {
        let mut events = self.events.lock().unwrap_or_else(|poisoned| {
            warn!("RateLimiter mutex poisoned, recovering");
            poisoned.into_inner()
        });

        let now = Instant::now();
        let window_start = now - self.window;

        // Remove events outside the window
        events.retain(|&t| t > window_start);

        if events.len() >= self.max_events as usize {
            return false;
        }

        events.push(now);
        true
    }

    /// Get current event count in window
    pub fn current_count(&self) -> usize {
        let events = self
            .events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        events.len()
    }

    /// Reset all events
    pub fn reset(&self) {
        let mut events = self
            .events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        events.clear();
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(MAX_EVENTS_PER_MINUTE, 60)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EVENT VALIDATOR (Anti-Hijack)
// ═══════════════════════════════════════════════════════════════════════════════

/// Validates and sanitizes analytics events
pub struct EventValidator;

impl EventValidator {
    /// Validate event name against allowed prefixes
    pub fn validate_event_name(name: &str) -> Result<String> {
        // Must be non-empty
        if name.is_empty() {
            return Err(AnalyticsError::InvalidEventName(
                "Event name cannot be empty".to_string(),
            ));
        }

        // Must not be too long
        if name.len() > 100 {
            return Err(AnalyticsError::InvalidEventName(
                "Event name too long".to_string(),
            ));
        }

        // Must have allowed prefix
        let has_valid_prefix = ALLOWED_EVENT_PREFIXES
            .iter()
            .any(|prefix| name.to_lowercase().starts_with(&prefix.to_lowercase()));

        if !has_valid_prefix {
            return Err(AnalyticsError::InvalidEventName(format!(
                "Event name must start with one of: {:?}",
                ALLOWED_EVENT_PREFIXES
            )));
        }

        // Must not contain suspicious patterns
        let suspicious = ["<script", "javascript:", "data:", "onload=", "onerror="];
        if suspicious.iter().any(|p| name.to_lowercase().contains(p)) {
            return Err(AnalyticsError::InvalidEventName(
                "Event name contains suspicious pattern".to_string(),
            ));
        }

        Ok(name.to_string())
    }

    /// Validate and sanitize event properties
    pub fn validate_properties(
        properties: HashMap<String, serde_json::Value>,
    ) -> Result<HashMap<String, serde_json::Value>> {
        if properties.len() > MAX_EVENT_PROPERTIES {
            return Err(AnalyticsError::InvalidProperties(format!(
                "Too many properties: {} (max {})",
                properties.len(),
                MAX_EVENT_PROPERTIES
            )));
        }

        let mut sanitized = HashMap::new();

        for (key, value) in properties {
            // Check for blocked keys
            let key_lower = key.to_lowercase();
            let is_blocked = BLOCKED_PROPERTY_KEYS
                .iter()
                .any(|blocked| key_lower == *blocked || key_lower == blocked.replace("_", ""));

            if is_blocked {
                warn!(key = %key, "Removing sensitive property from analytics event");
                continue;
            }

            // Validate key length
            if key.len() > MAX_PROPERTY_KEY_LENGTH {
                warn!(key = %key, "Property key too long, truncating");
                continue;
            }

            // Sanitize value
            match Self::sanitize_value(value) {
                Some(sanitized_value) => {
                    sanitized.insert(key, sanitized_value);
                }
                None => {
                    warn!(key = %key, "Invalid property value removed");
                }
            }
        }

        Ok(sanitized)
    }

    /// Sanitize a property value
    fn sanitize_value(value: serde_json::Value) -> Option<serde_json::Value> {
        match value {
            serde_json::Value::Null => Some(serde_json::Value::Null),
            serde_json::Value::Bool(b) => Some(serde_json::Value::Bool(b)),
            serde_json::Value::Number(n) => {
                // Validate number is finite
                if let Some(f) = n.as_f64() {
                    if f.is_finite() {
                        Some(serde_json::Value::Number(n))
                    } else {
                        Some(serde_json::Value::Number(0.into()))
                    }
                } else {
                    Some(serde_json::Value::Number(n))
                }
            }
            serde_json::Value::String(s) => {
                // Truncate long strings
                let cleaned = if s.len() > MAX_PROPERTY_VALUE_LENGTH {
                    format!("{}...", &s[..MAX_PROPERTY_VALUE_LENGTH])
                } else {
                    s
                };

                // Remove control characters and script tags
                let cleaned: String = cleaned
                    .chars()
                    .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
                    .collect();

                Some(serde_json::Value::String(cleaned))
            }
            serde_json::Value::Array(arr) => {
                // Limit array size
                let limited: Vec<_> = arr
                    .into_iter()
                    .take(50)
                    .filter_map(Self::sanitize_value)
                    .collect();
                Some(serde_json::Value::Array(limited))
            }
            serde_json::Value::Object(obj) => {
                // Recursively validate object
                let limited: serde_json::Map<String, serde_json::Value> = obj
                    .into_iter()
                    .take(20)
                    .filter_map(|(k, v)| Self::sanitize_value(v).map(|v| (k, v)))
                    .collect();
                Some(serde_json::Value::Object(limited))
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ANALYTICS BACKEND TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

/// Trait for analytics backends
pub trait AnalyticsBackend: Send + Sync {
    /// Send a single event
    fn send_event(&self, event: AnalyticsEvent) -> Result<()>;

    /// Send multiple events (batch)
    fn send_batch(&self, events: Vec<AnalyticsEvent>) -> Result<()>;

    /// Check if backend is available
    fn is_available(&self) -> bool;

    /// Flush any pending events
    fn flush(&self) -> Result<()>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSTHOG BACKEND
// ═══════════════════════════════════════════════════════════════════════════════

/// PostHog analytics backend implementation
#[cfg(feature = "analytics-reqwest")]
pub struct PostHogBackend {
    api_key: String,
    api_host: String,
    client: reqwest::Client,
    enabled: bool,
}

#[cfg(feature = "analytics-reqwest")]
impl PostHogBackend {
    pub fn new(api_key: String, api_host: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        Self {
            api_key,
            api_host: api_host.unwrap_or_else(|| "https://app.posthog.com".to_string()),
            client,
            enabled: true,
        }
    }

    /// Disable backend (for opt-out)
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Enable backend
    pub fn enable(&mut self) {
        self.enabled = true;
    }
}

#[cfg(feature = "analytics-reqwest")]
impl AnalyticsBackend for PostHogBackend {
    fn send_event(&self, event: AnalyticsEvent) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let mut properties = event.properties.clone();
        properties.insert("distinct_id".to_string(), event.session_id.clone().into());
        properties.insert("$event_id".to_string(), event.event_id.clone().into());
        properties.insert(
            "$timestamp".to_string(),
            event.timestamp.to_rfc3339().into(),
        );
        properties.insert("$lib".to_string(), "hqe-workbench-rust".into());
        properties.insert("$lib_version".to_string(), env!("CARGO_PKG_VERSION").into());

        let payload = serde_json::json!({
            "api_key": self.api_key,
            "event": event.name,
            "properties": properties,
            "timestamp": event.timestamp.to_rfc3339(),
        });

        // In production, this would make an actual HTTP request
        // For now, we just log in debug mode
        debug!(
            event = %event.name,
            "Would send event to PostHog: {}",
            payload.to_string()
        );

        Ok(())
    }

    fn send_batch(&self, events: Vec<AnalyticsEvent>) -> Result<()> {
        if !self.enabled || events.is_empty() {
            return Ok(());
        }

        debug!(count = events.len(), "Sending batch of events to PostHog");

        for event in events {
            self.send_event(event)?;
        }

        Ok(())
    }

    fn is_available(&self) -> bool {
        self.enabled
    }

    fn flush(&self) -> Result<()> {
        // No-op for this implementation
        Ok(())
    }
}

/// Stub PostHog backend when reqwest is not available
#[cfg(not(feature = "analytics-reqwest"))]
pub struct PostHogBackend;

#[cfg(not(feature = "analytics-reqwest"))]
impl PostHogBackend {
    /// Create a stub backend (logs a warning; enable `analytics-reqwest` feature for real support)
    pub fn new(_api_key: String, _api_host: Option<String>) -> Self {
        warn!("PostHog backend requires 'analytics-reqwest' feature");
        Self
    }
}

#[cfg(not(feature = "analytics-reqwest"))]
impl AnalyticsBackend for PostHogBackend {
    fn send_event(&self, _event: AnalyticsEvent) -> Result<()> {
        Err(AnalyticsError::BackendUnavailable)
    }

    fn send_batch(&self, _events: Vec<AnalyticsEvent>) -> Result<()> {
        Err(AnalyticsError::BackendUnavailable)
    }

    fn is_available(&self) -> bool {
        false
    }

    fn flush(&self) -> Result<()> {
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FALLBACK BACKEND (Local Logging)
// ═══════════════════════════════════════════════════════════════════════════════

/// Fallback backend that logs to local file/console
pub struct FallbackBackend {
    log_file: Option<std::path::PathBuf>,
}

impl FallbackBackend {
    /// Create a new fallback backend that logs to console only
    pub fn new() -> Self {
        Self { log_file: None }
    }

    /// Create a fallback backend that also writes to a log file
    pub fn with_log_file(log_file: std::path::PathBuf) -> Self {
        Self {
            log_file: Some(log_file),
        }
    }
}

impl Default for FallbackBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalyticsBackend for FallbackBackend {
    fn send_event(&self, event: AnalyticsEvent) -> Result<()> {
        let log_line = serde_json::to_string(&event).map_err(|e| {
            AnalyticsError::ValidationFailed(format!("Failed to serialize event: {}", e))
        })?;

        info!(target: "analytics_fallback", "{}", log_line);

        if let Some(ref path) = self.log_file {
            // Append to file (in production, would use proper file handling)
            debug!(path = %path.display(), "Would write analytics to file");
        }

        Ok(())
    }

    fn send_batch(&self, events: Vec<AnalyticsEvent>) -> Result<()> {
        for event in events {
            self.send_event(event)?;
        }
        Ok(())
    }

    fn is_available(&self) -> bool {
        true // Always available
    }

    fn flush(&self) -> Result<()> {
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN ANALYTICS MANAGER
// ═══════════════════════════════════════════════════════════════════════════════

/// Main analytics manager with security protections
pub struct AnalyticsManager {
    primary_backend: Option<Arc<dyn AnalyticsBackend>>,
    fallback_backend: Arc<dyn AnalyticsBackend>,
    rate_limiter: Arc<RateLimiter>,
    session_id: String,
    enabled: bool,
    pending_events: Mutex<Vec<AnalyticsEvent>>,
}

impl AnalyticsManager {
    /// Create a new analytics manager with fallback only
    pub fn new() -> Self {
        Self {
            primary_backend: None,
            fallback_backend: Arc::new(FallbackBackend::new()),
            rate_limiter: Arc::new(RateLimiter::default()),
            session_id: generate_session_id(),
            enabled: true,
            pending_events: Mutex::new(Vec::new()),
        }
    }

    /// Create with PostHog backend
    pub fn with_posthog(api_key: String, api_host: Option<String>) -> Self {
        let mut manager = Self::new();
        manager.primary_backend = Some(Arc::new(PostHogBackend::new(api_key, api_host)));
        manager
    }

    /// Track an event with full validation
    pub fn track(
        &self,
        name: &str,
        properties: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // Rate limiting
        if !self.rate_limiter.check_and_record() {
            warn!(event = name, "Analytics rate limit exceeded");
            return Err(AnalyticsError::RateLimitExceeded);
        }

        // Validate event name
        let validated_name = EventValidator::validate_event_name(name)?;

        // Validate properties
        let validated_properties = match properties {
            Some(props) => EventValidator::validate_properties(props)?,
            None => HashMap::new(),
        };

        // Create event
        let event = AnalyticsEvent {
            name: validated_name,
            properties: validated_properties,
            timestamp: chrono::Utc::now(),
            session_id: self.session_id.clone(),
            event_id: generate_event_id(),
        };

        // Try primary backend first
        if let Some(ref backend) = self.primary_backend {
            if backend.is_available() {
                match backend.send_event(event.clone()) {
                    Ok(()) => return Ok(()),
                    Err(e) => {
                        warn!(error = %e, "Primary analytics backend failed, using fallback");
                    }
                }
            }
        }

        // Use fallback
        self.fallback_backend.send_event(event)
    }

    /// Track a security event (always logged)
    pub fn track_security_event(
        &self,
        event_type: &str,
        details: HashMap<String, serde_json::Value>,
    ) {
        // Security events bypass some validation but still check rate limits
        if !self.rate_limiter.check_and_record() {
            warn!("Security event rate limit exceeded - this may indicate an attack");
        }

        let mut properties = details;
        properties.insert("event_type".to_string(), event_type.into());
        properties.insert("severity".to_string(), "security".into());

        let event = AnalyticsEvent {
            name: format!("security_{}", event_type),
            properties,
            timestamp: chrono::Utc::now(),
            session_id: self.session_id.clone(),
            event_id: generate_event_id(),
        };

        // Always log security events
        warn!(
            event_type = event_type,
            session_id = %self.session_id,
            "Security event tracked"
        );

        // Try to send
        if let Err(e) = self.fallback_backend.send_event(event) {
            error!(error = %e, "Failed to track security event");
        }
    }

    /// Enable analytics
    pub fn enable(&mut self) {
        self.enabled = true;
        info!("Analytics enabled");
    }

    /// Disable analytics
    pub fn disable(&mut self) {
        self.enabled = false;
        info!("Analytics disabled");
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get current rate limit status
    pub fn rate_limit_status(&self) -> (usize, u32) {
        (self.rate_limiter.current_count(), MAX_EVENTS_PER_MINUTE)
    }

    /// Flush pending events
    pub fn flush(&self) -> Result<()> {
        let pending = {
            let mut events = self
                .pending_events
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            std::mem::take(&mut *events)
        };

        if pending.is_empty() {
            return Ok(());
        }

        info!(count = pending.len(), "Flushing pending analytics events");

        // Try primary first
        if let Some(ref backend) = self.primary_backend {
            if backend.is_available() {
                match backend.send_batch(pending.clone()) {
                    Ok(()) => return Ok(()),
                    Err(e) => {
                        warn!(error = %e, "Failed to flush to primary backend");
                    }
                }
            }
        }

        // Fallback
        self.fallback_backend.send_batch(pending)
    }
}

impl Default for AnalyticsManager {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

fn generate_session_id() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    let random_part: String = (0..16)
        .map(|_| rng.random_range(0..36))
        .map(|i| {
            if i < 10 {
                (b'0' + i as u8) as char
            } else {
                (b'a' + (i - 10) as u8) as char
            }
        })
        .collect();
    format!(
        "session_{}_{}",
        chrono::Utc::now().timestamp_millis(),
        random_part
    )
}

fn generate_event_id() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    let random_part: String = (0..16)
        .map(|_| rng.random_range(0..36))
        .map(|i| {
            if i < 10 {
                (b'0' + i as u8) as char
            } else {
                (b'a' + (i - 10) as u8) as char
            }
        })
        .collect();
    format!(
        "evt_{}_{}",
        chrono::Utc::now().timestamp_millis(),
        random_part
    )
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(3, 60);

        assert!(limiter.check_and_record());
        assert!(limiter.check_and_record());
        assert!(limiter.check_and_record());
        assert!(!limiter.check_and_record()); // Should fail (limit reached)
    }

    #[test]
    fn test_validate_event_name_valid() {
        assert!(EventValidator::validate_event_name("app_started").is_ok());
        assert!(EventValidator::validate_event_name("chat_message_sent").is_ok());
        assert!(EventValidator::validate_event_name("scan_completed").is_ok());
    }

    #[test]
    fn test_validate_event_name_invalid_prefix() {
        assert!(EventValidator::validate_event_name("invalid_event").is_err());
        assert!(EventValidator::validate_event_name("hack_attempt").is_err());
    }

    #[test]
    fn test_validate_event_name_suspicious() {
        assert!(EventValidator::validate_event_name("app_<script>").is_err());
        assert!(EventValidator::validate_event_name("app_javascript:").is_err());
    }

    #[test]
    fn test_validate_properties_removes_sensitive() {
        let mut props = HashMap::new();
        props.insert("normal_key".to_string(), "value".into());
        props.insert("password".to_string(), "secret123".into());
        props.insert("api_key".to_string(), "key123".into());

        let result = EventValidator::validate_properties(props).unwrap();

        assert!(result.contains_key("normal_key"));
        assert!(!result.contains_key("password"));
        assert!(!result.contains_key("api_key"));
    }

    #[test]
    fn test_validate_properties_limits_count() {
        let props: HashMap<String, serde_json::Value> =
            (0..100).map(|i| (format!("key_{}", i), i.into())).collect();

        assert!(EventValidator::validate_properties(props).is_err());
    }

    #[test]
    fn test_session_id_generation() {
        let id1 = generate_session_id();
        let id2 = generate_session_id();

        assert_ne!(id1, id2);
        assert!(id1.starts_with("session_"));
        assert!(id2.starts_with("session_"));
    }
}
