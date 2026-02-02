use hqe_openai::{
    profile::ProfileManager, prompts::sanitize_for_prompt,
    provider_discovery::is_local_or_private_base_url, ClientConfig, OpenAIClient,
};
use secrecy::SecretString;
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
}

#[derive(Debug, Serialize)]
pub struct ExecutePromptResponse {
    pub result: String,
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
pub async fn get_available_prompts_with_metadata(app: AppHandle) -> Result<Vec<PromptToolInfoWithMetadata>, String> {
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
    // 1. Load Prompts & Find Tool
    let prompts_dir = get_prompts_dir(&app).ok_or("Could not locate prompts directory")?;
    let loader = hqe_mcp::PromptLoader::new(&prompts_dir);
    let loaded_tools = loader.load().map_err(|e| e.to_string())?;

    let tool = loaded_tools
        .into_iter()
        .find(|t| {
            t.definition.name == request.tool_name
                || format!("prompts__{}", t.definition.name) == request.tool_name
        })
        .ok_or_else(|| format!("Tool '{}' not found", request.tool_name))?;

    // 2. Initialize Client
    let manager = ProfileManager::default();

    // Use specified profile or first available
    let profile_name = if let Some(name) = request.profile_name {
        name
    } else {
        let profiles = manager.load_profiles().map_err(|e| e.to_string())?;
        profiles
            .first()
            .ok_or("No provider profiles configured")?
            .name
            .clone()
    };

    let (profile, api_key) = manager
        .get_profile_with_key(&profile_name)
        .map_err(|e| e.to_string())?
        .ok_or("Profile not found")?;

    let allow_missing_key = is_local_or_private_base_url(&profile.base_url).unwrap_or(false);
    let api_key = match api_key {
        Some(key) => key,
        None if allow_missing_key => SecretString::new(String::new()),
        None => return Err("API key not found for profile".to_string()),
    };

    let config = ClientConfig {
        base_url: profile.base_url,
        api_key,
        default_model: profile.default_model.clone(),
        headers: profile.headers.clone(),
        organization: profile.organization.clone(),
        project: profile.project.clone(),
        disable_system_proxy: false,
        timeout_seconds: profile.timeout_s,
        max_retries: 1,
        rate_limit_config: None,
        cache_enabled: true,
    };

    let client = OpenAIClient::new(config).map_err(|e| e.to_string())?;

    // 3. Execute
    let prompt_text = substitute_template(&tool.template, &request.args);

    let response = client
        .chat(hqe_openai::ChatRequest {
            model: client.default_model().to_string(),
            messages: vec![hqe_openai::Message {
                role: hqe_openai::Role::User,
                content: Some(prompt_text.into()),
                tool_calls: None,
            }],
            frequency_penalty: None,
            presence_penalty: None,
            repetition_penalty: None,
            logprobs: None,
            top_logprobs: None,
            temperature: Some(0.2),
            min_temp: None,
            max_temp: None,
            top_p: None,
            top_k: None,
            max_tokens: None,
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
        })
        .await
        .map_err(|e| e.to_string())?;

    let first = response
        .choices
        .first()
        .ok_or_else(|| "No response choices returned".to_string())?;

    let content = first
        .message
        .content
        .as_ref()
        .and_then(|c| c.to_text_lossy())
        .ok_or_else(|| "No content returned in response".to_string())?;

    Ok(ExecutePromptResponse { result: content })
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
        && key.len() <= 64  // Reasonable length limit
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
fn substitute_template(template: &str, args: &serde_json::Value) -> String {
    let mut result = template.to_string();

    if let Some(obj) = args.as_object() {
        for (k, v) in obj {
            // Validate key name to prevent injection attacks
            if !is_valid_key_name(k) {
                eprintln!("Warning: Invalid template key name '{}', skipping substitution", k);
                continue;
            }
            
            let key = format!("{{{{{}}}}}", k); // {{key}}
            let val = v
                .as_str()
                .map(sanitize_for_prompt) // Validate string values
                .unwrap_or_else(|| sanitize_for_prompt(&v.to_string())); // Validate non-string values
            result = result.replace(&key, &val);
        }
    }

    result
}
