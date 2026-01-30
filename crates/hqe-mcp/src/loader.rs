use anyhow::{Context, Result};
use hqe_protocol::models::MCPToolDefinition;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use walkdir::WalkDir;
use serde_json::json;

/// A loaded prompt file parsed from disk
#[derive(Debug, Clone, Deserialize)]
pub struct PromptFile {
    /// Optional description of what the prompt does
    pub description: Option<String>,
    /// The prompt template string
    pub prompt: String,
    /// Arguments that can be substituted into the template
    pub args: Option<Vec<PromptArg>>,
}

/// Argument definition for a prompt
#[derive(Debug, Clone, Deserialize)]
pub struct PromptArg {
    /// Name of the argument
    pub name: String,
    /// Description of the argument
    pub description: Option<String>,
    /// Whether the argument is required (default: true)
    pub required: Option<bool>,
}

/// A loaded prompt tool ready for registration
#[derive(Debug, Clone)]
pub struct LoadedPromptTool {
    /// The tool definition derived from the prompt file
    pub definition: MCPToolDefinition,
    /// The raw template string
    pub template: String,
}

/// Loader for file-based prompt templates
pub struct PromptLoader {
    root_path: PathBuf,
}

impl PromptLoader {
    /// Create a new prompt loader for the given directory
    pub fn new(root_path: impl AsRef<Path>) -> Self {
        Self {
            root_path: root_path.as_ref().to_path_buf(),
        }
    }

    /// Load all prompt files from the root directory
    pub fn load(&self) -> Result<Vec<LoadedPromptTool>> {
        let mut tools = Vec::new();
        info!("Scanning prompts from: {}", self.root_path.display());

        for entry in WalkDir::new(&self.root_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if matches!(ext, "toml" | "yaml" | "yml") {
                        match self.load_prompt_file(path) {
                            Ok(tool) => tools.push(tool),
                            Err(e) => warn!("Failed to load prompt file {}: {}", path.display(), e),
                        }
                    }
                }
            }
        }

        info!("Loaded {} prompt tools", tools.len());
        Ok(tools)
    }

    fn load_prompt_file(&self, path: &Path) -> Result<LoadedPromptTool> {
        // Security: Validate the file is within the root directory (prevent path traversal)
        let canonical_path = path.canonicalize()
            .with_context(|| format!("Failed to canonicalize path: {}", path.display()))?;
        let canonical_root = self.root_path.canonicalize()
            .with_context(|| format!("Failed to canonicalize root path: {}", self.root_path.display()))?;
        
        if !canonical_path.starts_with(&canonical_root) {
            anyhow::bail!("Path traversal detected: file '{}' is outside the allowed directory", 
                path.display());
        }

        let content = std::fs::read_to_string(&canonical_path)
            .with_context(|| format!("Failed to read file: {}", canonical_path.display()))?;

        let prompt_file: PromptFile = if canonical_path.extension().is_some_and(|e| e == "toml") {
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse TOML: {}", canonical_path.display()))?
        } else {
            serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse YAML: {}", canonical_path.display()))?
        };

        // Determine tool name from file path relative to root
        let relative_path = canonical_path.strip_prefix(&canonical_root)?;
        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let parent = relative_path
            .parent()
            .map(|p| p.to_string_lossy().replace("/", "_"))
            .unwrap_or_default();

        let name = if parent.is_empty() {
            file_stem.to_string()
        } else {
            format!("{}_{}", parent, file_stem)
        }
        .replace("-", "_")
        .trim_start_matches('_')
        .to_string();

        // Build JSON Schema for properties
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        // Add explicit args if defined
        if let Some(args) = &prompt_file.args {
            for arg in args {
                properties.insert(
                    arg.name.clone(),
                    json!({
                        "type": "string",
                        "description": arg.description.clone().unwrap_or_default()
                    }),
                );
                if arg.required.unwrap_or(true) {
                    required.push(arg.name.clone());
                }
            }
        } 
        
        // Add implicit "args" if template uses {{args}} and it wasn't defined
        if prompt_file.prompt.contains("{{args}}") && !properties.contains_key("args") {
             properties.insert(
                "args".to_string(),
                json!({
                    "type": "string",
                    "description": "Arguments for the prompt"
                }),
            );
            required.push("args".to_string());
        }

        let input_schema = json!({
            "type": "object",
            "properties": properties,
            "required": required
        });

        Ok(LoadedPromptTool {
            definition: MCPToolDefinition {
                name,
                description: prompt_file.description.unwrap_or_else(|| "Prompt template".to_string()),
                input_schema,
            },
            template: prompt_file.prompt,
        })
    }
}