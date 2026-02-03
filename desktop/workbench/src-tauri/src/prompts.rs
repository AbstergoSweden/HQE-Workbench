use crate::llm::{run_llm, LlmResponse};
use hqe_core::prompt_runner::{
    Compatibility, InputSpec, InputType, PromptCategory, PromptExecutionRequest, PromptTemplate,
};
use hqe_openai::prompts::sanitize_for_prompt;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use tauri::{command, AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub template: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptToolInfoWithMetadata {
    pub name: String,
    pub description: String,
    pub explanation: String,
    pub category: String,
    pub version: String,
    pub input_schema: serde_json::Value,
    pub template: String,
}

#[derive(Debug, Deserialize)]
pub struct ExecutePromptRequest {
    pub tool_name: String,
    pub args: serde_json::Value,
    pub profile_name: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExecutePromptResponse {
    pub result: String,
    pub system_prompt_version: String,
}

/// List all available prompt tools
#[command]
pub async fn get_available_prompts(app: AppHandle) -> Result<Vec<PromptToolInfo>, String> {
    let prompts_dir = get_prompts_dir(&app).ok_or("Could not locate prompts directory")?;

    let loader = hqe_mcp::PromptLoader::new(&prompts_dir);
    let loaded_tools = loader.load().map_err(|e| e.to_string())?;

    Ok(loaded_tools
        .into_iter()
        .map(|t| PromptToolInfo {
            name: t.definition.name,
            description: t.definition.description,
            input_schema: t.definition.input_schema,
            template: t.template,
        })
        .collect())
}

/// List all available prompt tools with rich metadata
#[command]
pub async fn get_available_prompts_with_metadata(
    app: AppHandle,
) -> Result<Vec<PromptToolInfoWithMetadata>, String> {
    let prompts_dir = get_prompts_dir(&app).ok_or("Could not locate prompts directory")?;

    // Try to use the enhanced registry if available
    let loader = hqe_mcp::PromptLoader::new(&prompts_dir);
    let mut registry = hqe_mcp::registry_v2::PromptRegistry::new(loader);

    match registry.load_all() {
        Ok(()) => {
            let prompts = registry.all();
            Ok(prompts
                .into_iter()
                .map(|p| PromptToolInfoWithMetadata {
                    name: p.metadata.id.clone(),
                    description: p.metadata.description.clone(),
                    explanation: p.metadata.explanation.clone(),
                    category: format!("{:?}", p.metadata.category).to_lowercase(),
                    version: p.metadata.version.clone(),
                    input_schema: serde_json::json!({
                        "properties": p.metadata.inputs.iter().map(|i| {
                            (i.name.clone(), serde_json::json!({
                                "type": format!("{:?}", i.input_type).to_lowercase(),
                                "description": i.description.clone(),
                            }))
                        }).collect::<serde_json::Map<String, serde_json::Value>>()
                    }),
                    template: p.template.clone(),
                })
                .collect())
        }
        Err(_) => {
            // Fallback to basic loader
            let loader = hqe_mcp::PromptLoader::new(&prompts_dir);
            let loaded_tools = loader.load().map_err(|e| e.to_string())?;

            Ok(loaded_tools
                .into_iter()
                .map(|t| PromptToolInfoWithMetadata {
                    name: t.definition.name.clone(),
                    description: t.definition.description.clone(),
                    explanation: t.definition.description.clone(),
                    category: "custom".to_string(),
                    version: "1.0.0".to_string(),
                    input_schema: t.definition.input_schema.clone(),
                    template: t.template,
                })
                .collect())
        }
    }
}

/// Execute a prompt tool
#[command]
pub async fn execute_prompt(
    request: ExecutePromptRequest,
    app: AppHandle,
) -> Result<ExecutePromptResponse, String> {
    let prompt_template = resolve_prompt_template(&app, &request.tool_name)?;
    let inputs = build_inputs(&request.args);
    let user_message = inputs
        .get("message")
        .cloned()
        .or_else(|| inputs.get("args").cloned())
        .unwrap_or_else(|| request.args.to_string());

    let execution_request = PromptExecutionRequest {
        prompt_template,
        user_message,
        inputs,
        context: Vec::new(),
        max_context_size: None,
    };

    let session_key = if let Some(profile) = &request.profile_name {
        let keys = app.state::<crate::AppState>().session_keys.clone();
        let guard = keys.lock().await;
        guard.get(profile).cloned()
    } else {
        None
    };
    let response: LlmResponse =
        run_llm(execution_request, request.profile_name, session_key, request.model).await?;

    Ok(ExecutePromptResponse {
        result: response.content,
        system_prompt_version: response.system_prompt_version,
    })
}

fn get_prompts_dir(app: &AppHandle) -> Option<PathBuf> {
    // Allow explicit override via environment variable
    if let Ok(dir) = std::env::var("HQE_PROMPTS_DIR") {
        let path = PathBuf::from(dir);
        if path.exists() {
            return path.canonicalize().ok().or(Some(path));
        }
    }

    // Check resource path (production bundle)
    if let Ok(resource_path) = app.path().resource_dir() {
        let prompts = resource_path.join("prompts");
        if prompts.exists() {
            return Some(prompts);
        }
    }

    // Check current working directory and its ancestors (dev)
    if let Ok(cwd) = std::env::current_dir() {
        if let Some(found) = find_prompts_dir(&cwd) {
            return Some(found);
        }
    }

    // Check executable directory and its ancestors (packaged/dev)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            if let Some(found) = find_prompts_dir(parent) {
                return Some(found);
            }
        }
    }

    None
}

fn find_prompts_dir(start: &Path) -> Option<PathBuf> {
    for ancestor in start.ancestors() {
        // Prefer TOML prompt templates in cli-prompt-library (compatible with PromptLoader)
        let cli_library = ancestor.join("mcp-server").join("cli-prompt-library");
        if cli_library.exists() && contains_loadable_prompts(&cli_library) {
            return Some(cli_library);
        }

        // Fallback: repo-root prompts/ only if it contains loadable templates
        let candidate = ancestor.join("prompts");
        if candidate.exists() && contains_loadable_prompts(&candidate) {
            return Some(candidate);
        }
    }
    None
}

/// Check if a directory contains loadable TOML prompt templates.
/// A loadable template must have `prompt = """` (multi-line prompt field).
fn contains_loadable_prompts(dir: &Path) -> bool {
    fn check_dir(dir: &Path, depth: usize) -> bool {
        if depth > 3 {
            return false;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            return false;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if check_dir(&path, depth + 1) {
                    return true;
                }
            } else if path.extension().is_some_and(|e| e == "toml") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if content.contains("prompt = \"\"\"") {
                        return true;
                    }
                }
            }
        }
        false
    }
    check_dir(dir, 0)
}

/// Validate template key names to prevent injection attacks.
/// Only allows alphanumeric characters, underscores, and hyphens.
fn is_valid_key_name(key: &str) -> bool {
    !key.is_empty()
        && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        // Prevent keys that could break out of template delimiters
        && !key.contains("{{")
        && !key.contains("}}")
        && !key.contains('\0')
}

/// Substitute template placeholders with values.
///
/// # Security
/// - Key names are validated to prevent template injection
/// - Values are sanitized to remove potentially dangerous content
/// - Recursive template injection is prevented by single-pass substitution
fn build_inputs(args: &serde_json::Value) -> std::collections::HashMap<String, String> {
    let mut inputs = std::collections::HashMap::new();

    if let Some(obj) = args.as_object() {
        for (k, v) in obj {
            // Validate key name to prevent injection attacks
            if !is_valid_key_name(k) {
                eprintln!(
                    "Warning: Invalid template key name '{}', skipping substitution",
                    k
                );
                continue;
            }

            let val = v
                .as_str()
                .map(sanitize_for_prompt)
                .unwrap_or_else(|| sanitize_for_prompt(&v.to_string()));
            inputs.insert(k.clone(), val);
        }
    }

    inputs
}

fn resolve_prompt_template(app: &AppHandle, tool_name: &str) -> Result<PromptTemplate, String> {
    let prompts_dir = get_prompts_dir(app).ok_or("Could not locate prompts directory")?;
    let loader = hqe_mcp::PromptLoader::new(&prompts_dir);
    let mut registry = hqe_mcp::registry_v2::PromptRegistry::new(loader);
    registry.load_all().map_err(|e| e.to_string())?;

    let prompt = registry
        .get(tool_name)
        .or_else(|| registry.get(&tool_name.replace("prompts__", "")))
        .ok_or_else(|| format!("Tool '{}' not found", tool_name))?;

    Ok(PromptTemplate {
        id: prompt.metadata.id.clone(),
        title: prompt.metadata.title.clone(),
        category: map_prompt_category(&prompt.metadata.category),
        description: prompt.metadata.description.clone(),
        version: prompt.metadata.version.clone(),
        template: prompt.template.clone(),
        required_inputs: prompt
            .metadata
            .inputs
            .iter()
            .map(|input| InputSpec {
                name: input.name.clone(),
                description: input.description.clone(),
                input_type: map_prompt_input_type(&input.input_type),
                required: input.required,
                default: input.default.clone(),
                validation: None,
            })
            .collect(),
        compatibility: Compatibility::default(),
        allowed_tools: prompt.metadata.allowed_tools.clone(),
    })
}

fn map_prompt_category(category: &hqe_mcp::registry_v2::PromptCategory) -> PromptCategory {
    match category {
        hqe_mcp::registry_v2::PromptCategory::Security => PromptCategory::Security,
        hqe_mcp::registry_v2::PromptCategory::Quality => PromptCategory::Quality,
        hqe_mcp::registry_v2::PromptCategory::Refactor => PromptCategory::Refactor,
        hqe_mcp::registry_v2::PromptCategory::Explain => PromptCategory::Explain,
        hqe_mcp::registry_v2::PromptCategory::Test => PromptCategory::Test,
        hqe_mcp::registry_v2::PromptCategory::Document => PromptCategory::Document,
        _ => PromptCategory::Custom,
    }
}

fn map_prompt_input_type(
    input_type: &hqe_mcp::registry_v2::InputType,
) -> hqe_core::prompt_runner::InputType {
    match input_type {
        hqe_mcp::registry_v2::InputType::String => InputType::String,
        hqe_mcp::registry_v2::InputType::Integer => InputType::Integer,
        hqe_mcp::registry_v2::InputType::Boolean => InputType::Boolean,
        hqe_mcp::registry_v2::InputType::Json => InputType::Json,
        hqe_mcp::registry_v2::InputType::Code => InputType::Code,
        hqe_mcp::registry_v2::InputType::FilePath => InputType::FilePath,
        hqe_mcp::registry_v2::InputType::TextArea
        | hqe_mcp::registry_v2::InputType::Select
        | hqe_mcp::registry_v2::InputType::MultiSelect => InputType::String,
    }
}
