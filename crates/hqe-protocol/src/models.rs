use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Provider kind enumeration for supported LLM providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    /// OpenAI API provider (api.openai.com)
    OpenAI,
    /// Venice AI provider (venice.ai)
    Venice,
    /// OpenRouter provider (openrouter.ai)
    OpenRouter,
    /// XAI (Grok) provider
    XAI,
    /// Generic OpenAI-compatible provider
    Generic,
}

impl std::fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderKind::OpenAI => write!(f, "openai"),
            ProviderKind::Venice => write!(f, "venice"),
            ProviderKind::OpenRouter => write!(f, "openrouter"),
            ProviderKind::XAI => write!(f, "xai"),
            ProviderKind::Generic => write!(f, "generic"),
        }
    }
}

/// Unified Provider Profile definition
///
/// This is the single source of truth for provider configuration across all crates.
/// Stored in: `~/.local/share/hqe-workbench/profiles.json`
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProviderProfile {
    /// Profile name (unique identifier)
    pub name: String,
    /// Provider base URL (e.g., "https://api.openai.com/v1")
    pub base_url: String,
    /// Reference to API key stored in keychain (format: "api_key:{profile_name}")
    pub api_key_id: String,
    /// Default model for this provider (e.g., "gpt-4o-mini")
    pub default_model: String,
    /// Additional HTTP headers (excluding Authorization)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    /// Organization identifier (for providers that support it)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    /// Project identifier (for providers that support it)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// Provider kind override (auto-detected from URL if not specified)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_kind: Option<ProviderKind>,
    /// HTTP timeout in seconds
    #[serde(default = "default_timeout_s")]
    pub timeout_s: u64,
}

fn default_timeout_s() -> u64 {
    60
}

impl ProviderProfile {
    /// Create a new profile with the given name and base URL
    pub fn new(name: impl Into<String>, base_url: impl Into<String>) -> Self {
        let name_str = name.into();
        Self {
            name: name_str.clone(),
            base_url: base_url.into(),
            api_key_id: format!("api_key:{}", name_str),
            default_model: "gpt-4o-mini".to_string(),
            headers: None,
            organization: None,
            project: None,
            provider_kind: None,
            timeout_s: default_timeout_s(),
        }
    }

    /// Set the default model for this profile
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = model.into();
        self
    }

    /// Set a custom header for this profile
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }

    /// Set the provider kind override
    pub fn with_provider_kind(mut self, kind: ProviderKind) -> Self {
        self.provider_kind = Some(kind);
        self
    }

    /// Set the timeout for this profile
    pub fn with_timeout(mut self, timeout_s: u64) -> Self {
        self.timeout_s = timeout_s;
        self
    }

    /// Set the API key ID
    pub fn with_api_key_id(mut self, api_key_id: impl Into<String>) -> Self {
        self.api_key_id = api_key_id.into();
        self
    }

    /// Validate the base URL
    pub fn validate_base_url(&self) -> Result<(), String> {
        if self.base_url.is_empty() {
            return Err("Base URL cannot be empty".to_string());
        }
        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            return Err("Base URL must start with http:// or https://".to_string());
        }
        Ok(())
    }

    /// Validate the headers
    pub fn validate_headers(&self) -> Result<(), String> {
        if let Some(headers) = &self.headers {
            for (key, value) in headers {
                if key.chars().any(|c| c.is_control() || c == ':') {
                    return Err(format!("Invalid header name: {}", key));
                }
                if value.chars().any(|c| c.is_control()) {
                    return Err(format!("Invalid header value for key: {}", key));
                }
            }
        }
        Ok(())
    }
}

/// The polymorphic entity that stores data for any topic.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Entity {
    /// Unique ID of the entity (UUID v4)
    pub id: String,
    /// The ID of the topic this entity belongs to (e.g., "finance", "code_audit")
    pub topic_id: String,
    /// The specific kind of entity within the topic (e.g., "StockTicker", "Vulnerability")
    pub kind: String,
    /// The actual data payload, validated against the Topic's schema for this kind.
    pub data: Value,
    /// Optional vector embedding for semantic search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_embedding: Option<Vec<f32>>,
    /// Timestamp of creation
    pub created_at: DateTime<Utc>,
    /// Timestamp of last update
    pub updated_at: DateTime<Utc>,
}

/// Defines a "Topic" (Plugin) that expands the capabilities of the Workbench.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TopicManifest {
    /// Unique ID of the topic (e.g., "finance-core")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Semantic version
    pub version: String,
    /// Capabilities provided by this topic
    pub capabilities: TopicCapabilities,
    /// JSON Schemas for the data 'kind's this topic manages.
    /// Key is the 'kind' name, Value is the JSON Schema.
    pub data_schemas: HashMap<String, Value>,
}

/// Capabilities exposed by a topic (tools, prompts, and workflows)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TopicCapabilities {
    /// MCP Tools exposed by this topic
    pub tools: Vec<MCPToolDefinition>,
    /// Prompt templates provided by this topic
    pub prompts: Vec<PromptTemplate>,
    /// Pre-defined workflows provided by this topic
    pub flows: Vec<WorkflowDefinition>,
}

/// Definition of an MCP (Model Context Protocol) tool
///
/// Tools are functions that can be called by the LLM to perform actions
/// or retrieve information.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MCPToolDefinition {
    /// Unique name of the tool
    pub name: String,
    /// Human-readable description of what the tool does
    pub description: String,
    /// JSON Schema defining the tool's input parameters
    pub input_schema: Value,
}

/// A prompt template with variable substitution
///
/// Templates use standard string interpolation with `{variable_name}` syntax.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PromptTemplate {
    /// Unique name of the template
    pub name: String,
    /// The template string with `{variable}` placeholders
    pub template: String,
    /// List of variable names that must be provided when rendering
    pub input_variables: Vec<String>,
}

/// Definition of a pre-configured workflow
///
/// Workflows are sequences of actions that can be executed together.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkflowDefinition {
    /// Unique identifier for the workflow
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Ordered sequence of workflow steps
    pub steps: Vec<WorkflowStep>,
}

/// A single step in a workflow
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkflowStep {
    /// Unique identifier for this step
    pub id: String,
    /// Action type, e.g., "call_tool", "prompt_llm"
    pub action: String,
    /// Parameters for the action (action-specific)
    pub params: Value,
}
