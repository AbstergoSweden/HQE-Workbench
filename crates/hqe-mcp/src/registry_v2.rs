//! Enhanced Prompt Registry (v2)
//!
//! This module provides an enhanced prompt registry with:
//! - Rich metadata (explanations, categories, compatibility)
//! - Schema validation
//! - Provider capability filtering
//! - Complete prompt discovery from all directories

use crate::loader::{LoadedPromptTool, PromptLoader};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};

/// Errors that can occur during registry operations
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Load error: {0}")]
    Load(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Prompt not found: {0}")]
    NotFound(String),
}

/// Categories for prompt classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum PromptCategory {
    /// Uncategorized/default
    #[default]
    Uncategorized,
    /// Security analysis prompts
    Security,
    /// Code quality analysis
    Quality,
    /// Code refactoring
    Refactor,
    /// Code explanation
    Explain,
    /// Testing and test generation
    Test,
    /// Documentation generation
    Document,
    /// Architecture and design
    Architecture,
    /// Performance analysis
    Performance,
    /// Dependency analysis
    Dependencies,
    /// Custom user-defined prompts
    Custom,
    /// Agent-specific prompts (internal)
    Agent,
}

impl std::fmt::Display for PromptCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PromptCategory::Uncategorized => write!(f, "Uncategorized"),
            PromptCategory::Security => write!(f, "Security"),
            PromptCategory::Quality => write!(f, "Quality"),
            PromptCategory::Refactor => write!(f, "Refactor"),
            PromptCategory::Explain => write!(f, "Explain"),
            PromptCategory::Test => write!(f, "Test"),
            PromptCategory::Document => write!(f, "Document"),
            PromptCategory::Architecture => write!(f, "Architecture"),
            PromptCategory::Performance => write!(f, "Performance"),
            PromptCategory::Dependencies => write!(f, "Dependencies"),
            PromptCategory::Custom => write!(f, "Custom"),
            PromptCategory::Agent => write!(f, "Agent"),
        }
    }
}

impl PromptCategory {
    /// Get the emoji/icon for this category
    pub fn icon(&self) -> &'static str {
        match self {
            PromptCategory::Uncategorized => "❓",
            PromptCategory::Security => "🔒",
            PromptCategory::Quality => "✨",
            PromptCategory::Refactor => "🔧",
            PromptCategory::Explain => "📖",
            PromptCategory::Test => "🧪",
            PromptCategory::Document => "📝",
            PromptCategory::Architecture => "🏗️",
            PromptCategory::Performance => "⚡",
            PromptCategory::Dependencies => "📦",
            PromptCategory::Custom => "🎨",
            PromptCategory::Agent => "🤖",
        }
    }

    /// Get the default sort order
    pub fn sort_order(&self) -> u8 {
        match self {
            PromptCategory::Uncategorized => 255, // Always last
            PromptCategory::Security => 0,
            PromptCategory::Quality => 1,
            PromptCategory::Performance => 2,
            PromptCategory::Architecture => 3,
            PromptCategory::Refactor => 4,
            PromptCategory::Test => 5,
            PromptCategory::Document => 6,
            PromptCategory::Explain => 7,
            PromptCategory::Dependencies => 8,
            PromptCategory::Custom => 9,
            PromptCategory::Agent => 10,
        }
    }
}

/// Compatibility requirements for a prompt
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Compatibility {
    /// Required provider kinds (e.g., "openai", "anthropic")
    #[serde(default)]
    pub providers: Vec<String>,
    /// Required model capabilities (e.g., "code", "reasoning")
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Minimum context window in tokens
    #[serde(default)]
    pub min_context_window: Option<u32>,
    /// Feature flags required
    #[serde(default)]
    pub required_features: Vec<String>,
}

/// Specification for a required input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSpec {
    /// Field name (matches template placeholder)
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Expected type
    #[serde(rename = "type")]
    pub input_type: InputType,
    /// Whether this input is required
    #[serde(default = "default_true")]
    pub required: bool,
    /// Default value (if optional)
    #[serde(default)]
    pub default: Option<String>,
    /// Example value for documentation
    #[serde(default)]
    pub example: Option<String>,
}

fn default_true() -> bool {
    true
}

/// Input type for validation and UI rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputType {
    String,
    Integer,
    Boolean,
    Json,
    Code,
    FilePath,
    TextArea,
    Select,
    MultiSelect,
}

/// Rich prompt metadata with explanations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMetadata {
    /// Unique identifier (derived from file path)
    pub id: String,
    /// Human-readable title
    pub title: String,
    /// Category for grouping
    #[serde(default)]
    pub category: PromptCategory,
    /// Brief description (one line)
    pub description: String,
    /// Full explanation (multi-line, markdown supported)
    pub explanation: String,
    /// Semantic version
    #[serde(default = "default_version")]
    pub version: String,
    /// Required input specifications
    #[serde(default)]
    pub inputs: Vec<InputSpec>,
    /// Compatibility requirements
    #[serde(default)]
    pub compatibility: Compatibility,
    /// Allowed MCP tools (if any)
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    /// Example usage
    #[serde(default)]
    pub example: Option<PromptExample>,
    /// Author/maintainer
    #[serde(default)]
    pub author: Option<String>,
    /// Tags for search/filtering
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Example usage for a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptExample {
    /// Description of what this example does
    pub description: String,
    /// Example input values
    pub inputs: HashMap<String, String>,
    /// Expected output (if deterministic)
    #[serde(default)]
    pub expected_output: Option<String>,
}

/// Enhanced loaded prompt with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedPrompt {
    /// Core metadata
    pub metadata: PromptMetadata,
    /// The prompt template string
    pub template: String,
    /// Input schema (JSON Schema format)
    pub input_schema: serde_json::Value,
    /// Source file path
    pub source_path: String,
    /// Whether this is an agent prompt (internal)
    pub is_agent_prompt: bool,
}

/// Registry of all available prompts
#[derive(Debug, Clone)]
pub struct PromptRegistry {
    prompts: HashMap<String, EnrichedPrompt>,
    loader: PromptLoader,
}

impl PromptRegistry {
    /// Create a new registry with the given loader
    pub fn new(loader: PromptLoader) -> Self {
        Self {
            prompts: HashMap::new(),
            loader,
        }
    }

    /// Load all prompts from the loader
    pub fn load_all(&mut self) -> Result<(), RegistryError> {
        info!("Loading all prompts into registry");

        let loaded_tools = self
            .loader
            .load()
            .map_err(|e| RegistryError::Load(e.to_string()))?;

        for tool in loaded_tools {
            match self.enrich_prompt(tool) {
                Ok(enriched) => {
                    debug!(prompt_id = %enriched.metadata.id, "Loaded prompt");
                    self.prompts.insert(enriched.metadata.id.clone(), enriched);
                }
                Err(e) => {
                    warn!(error = %e, "Failed to enrich prompt");
                }
            }
        }

        info!(count = self.prompts.len(), "Prompt registry loaded");
        Ok(())
    }

    /// Enrich a loaded prompt with metadata
    fn enrich_prompt(&self, tool: LoadedPromptTool) -> Result<EnrichedPrompt, RegistryError> {
        // Derive ID from name
        let id = tool.definition.name.clone();

        // Detect if agent prompt
        let is_agent_prompt =
            id.starts_with("conductor_") || id.starts_with("cli_security_") || id.starts_with("agent_");

        // Determine category from name patterns
        let category = self.detect_category(&id);

        // Build input specs from schema
        let inputs = self.extract_input_specs(&tool.definition.input_schema)?;

        // Build explanation from description and schema
        let explanation = self.build_explanation(&tool.definition.description, &inputs);

        // Build title from ID
        let title = id
            .replace('_', " ")
            .replace('-', " ")
            .split_whitespace()
            .map(|w| {
                let mut chars = w.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        let metadata = PromptMetadata {
            id: id.clone(),
            title,
            category,
            description: tool.definition.description.clone(),
            explanation,
            version: "1.0.0".to_string(),
            inputs,
            compatibility: Compatibility::default(),
            allowed_tools: vec![],
            example: None,
            author: None,
            tags: vec![],
        };

        Ok(EnrichedPrompt {
            metadata,
            template: tool.template,
            input_schema: tool.definition.input_schema,
            source_path: id.clone(), // Simplified; could store actual path
            is_agent_prompt,
        })
    }

    /// Detect category from prompt name
    fn detect_category(&self, name: &str) -> PromptCategory {
        let lower = name.to_lowercase();
        if lower.contains("secur") || lower.contains("vuln") || lower.contains("audit") {
            PromptCategory::Security
        } else if lower.contains("quality") || lower.contains("lint") || lower.contains("clean") {
            PromptCategory::Quality
        } else if lower.contains("refactor") || lower.contains("rewrite") {
            PromptCategory::Refactor
        } else if lower.contains("explain") || lower.contains("doc") || lower.contains("describe") {
            PromptCategory::Explain
        } else if lower.contains("test") {
            PromptCategory::Test
        } else if lower.contains("arch") || lower.contains("design") {
            PromptCategory::Architecture
        } else if lower.contains("perf") || lower.contains("optim") {
            PromptCategory::Performance
        } else if lower.contains("dep") || lower.contains("package") || lower.contains("import") {
            PromptCategory::Dependencies
        } else if lower.contains("agent") || lower.contains("conductor") || lower.contains("cli_security") {
            PromptCategory::Agent
        } else {
            PromptCategory::Custom
        }
    }

    /// Extract input specs from JSON schema
    fn extract_input_specs(&self, schema: &serde_json::Value) -> Result<Vec<InputSpec>, RegistryError> {
        let mut specs = Vec::new();

        if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
            let required: Vec<String> = schema
                .get("required")
                .and_then(|r| r.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            for (name, prop) in properties {
                let input_type = match prop.get("type").and_then(|t| t.as_str()) {
                    Some("integer") => InputType::Integer,
                    Some("number") => InputType::Integer,
                    Some("boolean") => InputType::Boolean,
                    Some("object") => InputType::Json,
                    Some("array") => InputType::Json,
                    _ => InputType::String,
                };

                let description = prop
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or(name)
                    .to_string();

                let default = prop.get("default").map(|d| d.to_string());

                specs.push(InputSpec {
                    name: name.clone(),
                    description,
                    input_type,
                    required: required.contains(name),
                    default,
                    example: None,
                });
            }
        }

        Ok(specs)
    }

    /// Build a rich explanation from description and inputs
    fn build_explanation(&self, description: &str, inputs: &[InputSpec]) -> String {
        let mut explanation = format!("## Purpose\n\n{}", description);

        if !inputs.is_empty() {
            explanation.push_str("\n\n## Inputs\n\n");
            for input in inputs {
                let req_indicator = if input.required { "**Required**" } else { "Optional" };
                explanation.push_str(&format!(
                    "- **{}** ({}): {}\n",
                    input.name, req_indicator, input.description
                ));
                if let Some(default) = &input.default {
                    explanation.push_str(&format!("  - Default: `{}`\n", default));
                }
                if let Some(example) = &input.example {
                    explanation.push_str(&format!("  - Example: `{}`\n", example));
                }
            }
        }

        explanation.push_str("\n## Output\n\nThe AI will provide analysis based on the inputs provided.");

        explanation
    }

    /// Get a prompt by ID
    pub fn get(&self, id: &str) -> Option<&EnrichedPrompt> {
        self.prompts.get(id)
    }

    /// Get all prompts
    pub fn all(&self) -> Vec<&EnrichedPrompt> {
        self.prompts.values().collect()
    }

    /// Get prompts by category
    pub fn by_category(&self, category: PromptCategory) -> Vec<&EnrichedPrompt> {
        self.prompts
            .values()
            .filter(|p| p.metadata.category == category)
            .collect()
    }

    /// Filter prompts by search query
    pub fn search(&self, query: &str) -> Vec<&EnrichedPrompt> {
        let lower_query = query.to_lowercase();
        self.prompts
            .values()
            .filter(|p| {
                p.metadata.id.to_lowercase().contains(&lower_query)
                    || p.metadata.title.to_lowercase().contains(&lower_query)
                    || p.metadata.description.to_lowercase().contains(&lower_query)
                    || p.metadata.tags.iter().any(|t| t.to_lowercase().contains(&lower_query))
            })
            .collect()
    }

    /// Get prompts sorted by category then title
    pub fn sorted(&self) -> Vec<&EnrichedPrompt> {
        let mut prompts: Vec<_> = self.prompts.values().collect();
        prompts.sort_by(|a, b| {
            let cat_ord = a.metadata.category.sort_order().cmp(&b.metadata.category.sort_order());
            if cat_ord != std::cmp::Ordering::Equal {
                return cat_ord;
            }
            a.metadata.title.cmp(&b.metadata.title)
        });
        prompts
    }

    /// Get count of prompts
    pub fn count(&self) -> usize {
        self.prompts.len()
    }

    /// Get count by category
    pub fn count_by_category(&self) -> HashMap<PromptCategory, usize> {
        let mut counts = HashMap::new();
        for prompt in self.prompts.values() {
            *counts.entry(prompt.metadata.category).or_insert(0) += 1;
        }
        counts
    }

    /// Filter prompts by provider compatibility
    pub fn compatible_with_provider(&self, provider_kind: &str) -> Vec<&EnrichedPrompt> {
        self.prompts
            .values()
            .filter(|p| {
                p.metadata.compatibility.providers.is_empty()
                    || p.metadata.compatibility.providers.contains(&provider_kind.to_string())
            })
            .collect()
    }

    /// Get all agent prompts (for internal use)
    pub fn agent_prompts(&self) -> Vec<&EnrichedPrompt> {
        self.prompts.values().filter(|p| p.is_agent_prompt).collect()
    }

    /// Get all non-agent prompts (for user display)
    pub fn user_prompts(&self) -> Vec<&EnrichedPrompt> {
        self.prompts
            .values()
            .filter(|p| !p.is_agent_prompt)
            .collect()
    }
}

/// Create a registry from a prompts directory
pub fn create_registry<P: AsRef<Path>>(prompts_dir: P) -> Result<PromptRegistry, RegistryError> {
    let loader = PromptLoader::new(prompts_dir);
    let mut registry = PromptRegistry::new(loader);
    registry.load_all()?;
    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_category_detection() {
        let registry = PromptRegistry::new(PromptLoader::new("/tmp"));

        assert_eq!(
            registry.detect_category("security_audit"),
            PromptCategory::Security
        );
        assert_eq!(
            registry.detect_category("code_quality_check"),
            PromptCategory::Quality
        );
        assert_eq!(registry.detect_category("explain_function"), PromptCategory::Explain);
        assert_eq!(registry.detect_category("refactor_code"), PromptCategory::Refactor);
        assert_eq!(registry.detect_category("generate_tests"), PromptCategory::Test);
        assert_eq!(registry.detect_category("conductor_task"), PromptCategory::Agent);
    }

    #[test]
    fn test_extract_input_specs() {
        let registry = PromptRegistry::new(PromptLoader::new("/tmp"));

        let schema = json!({
            "type": "object",
            "properties": {
                "language": {
                    "type": "string",
                    "description": "Programming language"
                },
                "code": {
                    "type": "string",
                    "description": "Code to analyze"
                },
                "strict": {
                    "type": "boolean",
                    "description": "Strict mode"
                }
            },
            "required": ["language", "code"]
        });

        let specs = registry.extract_input_specs(&schema).unwrap();

        assert_eq!(specs.len(), 3);
        assert!(specs.iter().any(|s| s.name == "language" && s.required));
        assert!(specs.iter().any(|s| s.name == "code" && s.required));
        assert!(specs.iter().any(|s| s.name == "strict" && !s.required));
    }

    #[test]
    fn test_build_explanation() {
        let registry = PromptRegistry::new(PromptLoader::new("/tmp"));

        let inputs = vec![
            InputSpec {
                name: "code".to_string(),
                description: "Code to analyze".to_string(),
                input_type: InputType::Code,
                required: true,
                default: None,
                example: Some("fn main() {}".to_string()),
            },
            InputSpec {
                name: "strict".to_string(),
                description: "Enable strict mode".to_string(),
                input_type: InputType::Boolean,
                required: false,
                default: Some("false".to_string()),
                example: None,
            },
        ];

        let explanation = registry.build_explanation("Analyze code for issues", &inputs);

        assert!(explanation.contains("## Purpose"));
        assert!(explanation.contains("## Inputs"));
        assert!(explanation.contains("## Output"));
        assert!(explanation.contains("code"));
        assert!(explanation.contains("**Required**"));
        assert!(explanation.contains("Optional"));
    }

    #[test]
    fn test_category_sort_order() {
        let security = PromptCategory::Security;
        let custom = PromptCategory::Custom;
        let agent = PromptCategory::Agent;

        assert!(security.sort_order() < custom.sort_order());
        assert!(custom.sort_order() < agent.sort_order());
    }
}
