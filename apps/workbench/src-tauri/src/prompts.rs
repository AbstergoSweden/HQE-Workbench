use hqe_protocol::models::MCPToolDefinition;
use hqe_openai::{OpenAIClient, ClientConfig, profile::{ProfileManager, ApiKeyStore, KeychainStore}};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{command, AppHandle, Manager};
use serde_json::json;
use tracing::{info, error};

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

    Ok(loaded_tools.into_iter().map(|t| PromptToolInfo {
        name: t.definition.name,
        description: t.definition.description,
        input_schema: t.definition.input_schema,
        template: t.template,
    }).collect())
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
    
    let tool = loaded_tools.into_iter()
        .find(|t| t.definition.name == request.tool_name || format!("prompts__{}", t.definition.name) == request.tool_name)
        .ok_or_else(|| format!("Tool '{}' not found", request.tool_name))?;

    // 2. Initialize Client
    let manager = ProfileManager::default();
    
    // Use specified profile or first available
    let profile_name = if let Some(name) = request.profile_name {
        name
    } else {
        let profiles = manager.load_profiles().map_err(|e| e.to_string())?;
        profiles.first()
            .ok_or("No provider profiles configured")?
            .name.clone()
    };

    let (profile, api_key) = manager.get_profile_with_key(&profile_name)
        .map_err(|e| e.to_string())?
        .ok_or("Profile not found")?;
        
    let api_key = api_key.ok_or("API key not found for profile")?;

    let config = ClientConfig {
        base_url: profile.base_url,
        api_key,
        default_model: profile.default_model.clone(),
        timeout_seconds: 120,
        max_retries: 1,
        rate_limit_config: None,
    };

    let client = OpenAIClient::new(config).map_err(|e| e.to_string())?;

    // 3. Execute
    let prompt_text = substitute_template(&tool.template, &request.args);
    
    let response = client.chat(hqe_openai::ChatRequest {
        model: client.default_model().to_string(),
        messages: vec![
            hqe_openai::Message {
                role: hqe_openai::Role::User,
                content: prompt_text,
            }
        ],
        temperature: Some(0.2),
        max_tokens: None,
        response_format: None,
    }).await.map_err(|e| e.to_string())?;

    Ok(ExecutePromptResponse {
        result: response.choices[0].message.content.clone(),
    })
}

fn get_prompts_dir(app: &AppHandle) -> Option<PathBuf> {
    // Logic to find prompts dir relative to app resource dir or executable
    // For dev, it's in project root "prompts"
    // For build, it might be in resource dir
    
    // Check local dev path first
    let dev_path = PathBuf::from("../../../prompts");
    if dev_path.exists() {
        return Some(dev_path.canonicalize().ok()?);
    }
    
    // Check resource path (production)
    if let Ok(resource_path) = app.path().resource_dir() {
        let prompts = resource_path.join("prompts");
        if prompts.exists() {
            return Some(prompts);
        }
    }
    
    // Fallback: Check current dir
    let cwd = std::env::current_dir().ok()?;
    let local = cwd.join("prompts");
    if local.exists() {
        return Some(local);
    }

    None
}

fn substitute_template(template: &str, args: &serde_json::Value) -> String {
    let mut result = template.to_string();
    
    if let Some(obj) = args.as_object() {
        for (k, v) in obj {
            let key = format!("{{{{{}}}}}", k); // {{key}}
            let val = v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string());
            result = result.replace(&key, &val);
        }
    }
    
    result
}
