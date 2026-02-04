use hqe_protocol::models::MCPToolDefinition;
use serde::{de::Error, Deserialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use thiserror::Error;
use tracing::{info, warn};
use walkdir::WalkDir;

static PROMPT_CACHE: OnceLock<Mutex<HashMap<PathBuf, Vec<LoadedPromptTool>>>> = OnceLock::new();

/// Errors that can occur during prompt loading
#[derive(Debug, Error)]
pub enum LoaderError {
    /// IO operation failed
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Path traversal detected
    #[error("Path traversal detected: file '{0}' is outside allowed directory")]
    PathTraversal(PathBuf),

    /// Failed to parse TOML
    #[error("Failed to parse TOML in {path}: {source}")]
    ParseToml {
        /// File path
        path: PathBuf,
        /// Underlying error
        source: toml::de::Error,
    },

    /// Failed to parse YAML
    #[error("Failed to parse YAML in {path}: {source}")]
    ParseYaml {
        /// File path
        path: PathBuf,
        /// Underlying error
        source: serde_yaml::Error,
    },

    /// Path canonicalization failed
    #[error("Failed to canonicalize path {path}: {source}")]
    Canonicalization {
        /// The path that failed
        path: PathBuf,
        /// The underlying error
        source: std::io::Error,
    },

    /// Failed to strip prefix
    #[error("Failed to strip prefix from path: {0}")]
    StripPrefix(#[from] std::path::StripPrefixError),
}

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
#[derive(Debug, Clone)]
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
    pub fn load(&self) -> Result<Vec<LoadedPromptTool>, LoaderError> {
        let cache_key = self
            .root_path
            .canonicalize()
            .unwrap_or_else(|_| self.root_path.clone());
        if let Ok(cache) = PROMPT_CACHE
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
        {
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        let mut tools = Vec::new();
        info!("Scanning prompts from: {}", self.root_path.display());

        for entry in WalkDir::new(&self.root_path)
            .follow_links(false)
            .max_depth(5)
            .into_iter()
            // Avoid scanning vendored/build outputs that can explode runtime and log volume
            // (e.g. `node_modules` after installing prompt-server deps).
            .filter_entry(|e| !should_ignore_dir_entry(e))
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
        if let Ok(mut cache) = PROMPT_CACHE
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
        {
            cache.insert(cache_key, tools.clone());
        }
        Ok(tools)
    }

    /// Clear cached prompt results (used when refreshing prompts).
    pub fn clear_cache(root_path: impl AsRef<Path>) {
        if let Ok(mut cache) = PROMPT_CACHE
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
        {
            let key = root_path
                .as_ref()
                .canonicalize()
                .unwrap_or_else(|_| root_path.as_ref().to_path_buf());
            cache.remove(&key);
        }
    }

    fn load_prompt_file(&self, path: &Path) -> Result<LoadedPromptTool, LoaderError> {
        // Security: Validate the file is within the root directory (prevent path traversal)
        let canonical_path = path
            .canonicalize()
            .map_err(|e| LoaderError::Canonicalization {
                path: path.to_path_buf(),
                source: e,
            })?;
        let canonical_root =
            self.root_path
                .canonicalize()
                .map_err(|e| LoaderError::Canonicalization {
                    path: self.root_path.clone(),
                    source: e,
                })?;

        if !canonical_path.starts_with(&canonical_root) {
            return Err(LoaderError::PathTraversal(path.to_path_buf()));
        }

        let content = std::fs::read_to_string(&canonical_path)?;

        let prompt_file: PromptFile = match canonical_path.extension().and_then(|s| s.to_str()) {
            Some("toml") => {
                // Add validation for TOML content to prevent deserialization attacks
                Self::validate_toml_content(&content)?;
                toml::from_str(&content).map_err(|e| LoaderError::ParseToml {
                    path: canonical_path.clone(),
                    source: e,
                })?
            }
            Some("yaml") | Some("yml") => {
                // Add validation for YAML content to prevent deserialization attacks
                Self::validate_yaml_content(&content)?;
                serde_yaml::from_str(&content).map_err(|e| LoaderError::ParseYaml {
                    path: canonical_path.clone(),
                    source: e,
                })?
            }
            _ => {
                return Err(LoaderError::ParseYaml {
                    path: canonical_path.clone(),
                    source: serde_yaml::Error::custom(
                        "Unsupported file extension, expected .toml, .yaml, or .yml",
                    ),
                })
            }
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

        // Validate the prompt template for malicious content
        Self::validate_prompt_template(&prompt_file.prompt)?;

        Ok(LoadedPromptTool {
            definition: MCPToolDefinition {
                name,
                description: prompt_file
                    .description
                    .unwrap_or_else(|| "Prompt template".to_string()),
                input_schema,
            },
            template: prompt_file.prompt,
        })
    }

    // Add validation functions for TOML and YAML content
    fn validate_toml_content(content: &str) -> Result<(), LoaderError> {
        // Check for potentially dangerous patterns in TOML
        if content.contains("!!") || content.contains("!<!") || content.contains("!") {
            // These could indicate custom types or unsafe deserialization
            return Err(LoaderError::ParseToml {
                path: std::path::PathBuf::from("validation"),
                source: toml::de::Error::custom("TOML contains potentially unsafe type indicators"),
            });
        }

        Ok(())
    }

    fn validate_yaml_content(content: &str) -> Result<(), LoaderError> {
        // Check for potentially dangerous patterns in YAML
        if content.contains("!!")
            || content.contains("!")
            || content.contains("&")
            || content.contains("*")
        {
            // These could indicate custom types, aliases, or unsafe deserialization
            return Err(LoaderError::ParseYaml {
                path: std::path::PathBuf::from("validation"),
                source: serde_yaml::Error::custom(
                    "YAML contains potentially unsafe type indicators or anchors",
                ),
            });
        }

        Ok(())
    }

    // Add validation function for prompt templates
    fn validate_prompt_template(template: &str) -> Result<(), LoaderError> {
        // Check for common prompt injection patterns in the template
        let lower_template = template.to_lowercase();

        // Look for patterns that could be used for prompt injection
        if lower_template.contains("ignore")
            && (lower_template.contains("above")
                || lower_template.contains("previous")
                || lower_template.contains("instructions"))
        {
            return Err(LoaderError::ParseYaml {
            path: std::path::PathBuf::from("validation"),
            source: serde_yaml::Error::custom("Prompt template contains potential injection pattern: 'ignore' with instructions"),
        });
        }

        if lower_template.contains("disregard")
            && (lower_template.contains("above")
                || lower_template.contains("previous")
                || lower_template.contains("instructions"))
        {
            return Err(LoaderError::ParseYaml {
            path: std::path::PathBuf::from("validation"),
            source: serde_yaml::Error::custom("Prompt template contains potential injection pattern: 'disregard' with instructions"),
        });
        }

        if lower_template.contains("system")
            && (lower_template.contains("prompt") || lower_template.contains("instructions"))
        {
            return Err(LoaderError::ParseYaml {
            path: std::path::PathBuf::from("validation"),
            source: serde_yaml::Error::custom("Prompt template contains potential injection pattern: 'system' with prompt/instructions"),
        });
        }

        if lower_template.contains("nevermind") || lower_template.contains("actually") {
            return Err(LoaderError::ParseYaml {
            path: std::path::PathBuf::from("validation"),
            source: serde_yaml::Error::custom("Prompt template contains potential injection pattern: 'nevermind' or 'actually'"),
        });
        }

        // Check for template injection patterns
        if template.matches("{{").count() != template.matches("}}").count() {
            return Err(LoaderError::ParseYaml {
                path: std::path::PathBuf::from("validation"),
                source: serde_yaml::Error::custom(
                    "Prompt template has unmatched template delimiters",
                ),
            });
        }

        Ok(())
    }
}

fn should_ignore_dir_entry(entry: &walkdir::DirEntry) -> bool {
    if !entry.file_type().is_dir() {
        return false;
    }

    let name = entry.file_name().to_string_lossy();
    // In this repo, `prompts/prompts/**` is a vendored MCP server project
    // (not Workbench prompt templates). Skipping it avoids parse noise and keeps
    // Thinktank prompt listing fast and relevant.
    if entry.depth() == 1 && name.as_ref() == "prompts" {
        return true;
    }
    matches!(
        name.as_ref(),
        ".git" | "node_modules" | "dist" | "build" | "target" | "__pycache__"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn loader_skips_node_modules() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();

        fs::write(
            root.join("valid.toml"),
            r#"
description = "Valid"
prompt = "Hello {{args}}"
"#,
        )
        .expect("write valid");

        fs::create_dir_all(root.join("node_modules/somepkg")).expect("mkdir node_modules");
        fs::write(
            root.join("node_modules/somepkg/should_not_load.toml"),
            r#"
description = "Should not load"
prompt = "NOPE"
"#,
        )
        .expect("write ignored");

        let loader = PromptLoader::new(root);
        let tools = loader.load().expect("load prompts");

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].definition.name, "valid");
    }
}
