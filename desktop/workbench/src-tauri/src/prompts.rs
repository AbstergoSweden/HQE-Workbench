use hqe_openai::{
    profile::ProfileManager, provider_discovery::is_local_or_private_base_url, ClientConfig,
    OpenAIClient,
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
        let candidate = ancestor.join("prompts");
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn substitute_template(template: &str, args: &serde_json::Value) -> String {
    let mut result = template.to_string();

    if let Some(obj) = args.as_object() {
        for (k, v) in obj {
            let key = format!("{{{{{}}}}}", k); // {{key}}
            let val = v
                .as_str()
                .map(|s| validate_template_value(s))  // Validate string values
                .unwrap_or_else(|| validate_template_value(&v.to_string())); // Validate non-string values
            result = result.replace(&key, &val);
        }
    }

    result
}

// Validate that a template value doesn't contain dangerous patterns
fn validate_template_value(value: &str) -> String {
    // If the value contains template-like expressions, escape them to prevent processing
    let mut result = value.to_string();

    // Escape template delimiters to prevent them from being processed as templates
    result = result.replace("{{", "\\{\\{");
    result = result.replace("{%", "\\{%");
    result = result.replace("{#", "\\{#");
    result = result.replace("}}", "\\}\\}");
    result = result.replace("%}", "%\\}");
    result = result.replace("#}", "#\\}");

    result
}
