//! Single LLM execution module for Tauri commands.
// INVARIANT: Only provider HTTP call site

use hqe_core::prompt_runner::{PromptExecutionRequest, PromptRunner};
use hqe_openai::profile::{ProfileManager, ProviderProfileExt};
use hqe_openai::provider_discovery::{is_local_or_private_base_url, ProviderDiscoveryClient};
use hqe_openai::{ChatRequest, Message, MessageContent, OpenAIClient, Role};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub usage: Option<hqe_core::prompt_runner::TokenUsage>,
    pub raw_model_id: String,
    pub provider_kind: String,
    pub system_prompt_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub context_window: Option<u32>,
}

pub async fn run_llm(
    request: PromptExecutionRequest,
    profile_name: Option<String>,
    session_key: Option<SecretString>,
    model_override: Option<String>,
) -> Result<LlmResponse, String> {
    let runner = PromptRunner::default();
    let prompt = runner.build_prompt(&request).map_err(|e| e.to_string())?;

    let (profile, api_key) = resolve_profile(profile_name, session_key)?;
    let allow_missing_key = is_local_or_private_base_url(&profile.base_url).unwrap_or(false);
    let api_key = match api_key {
        Some(key) => key,
        None if allow_missing_key => SecretString::new(String::new().into_boxed_str()),
        None => return Err("No API key stored for profile".to_string()),
    };

    let headers = profile.sanitized_headers().map_err(|e| e.to_string())?;
    let model = normalize_model_override(&profile.default_model, model_override);
    let config = hqe_openai::ClientConfig {
        base_url: profile.base_url.clone(),
        api_key,
        default_model: model.clone(),
        headers: Some(headers),
        organization: profile.organization.clone(),
        project: profile.project.clone(),
        disable_system_proxy: false,
        timeout_seconds: profile.timeout_s,
        max_retries: 1,
        rate_limit_config: None,
        cache_enabled: true,
        daily_budget: 1.0,
    };

    let client = OpenAIClient::new(config).map_err(|e| e.to_string())?;

    let response = client
        .chat(ChatRequest {
            model,
            messages: vec![Message {
                role: Role::User,
                content: Some(MessageContent::Text(prompt)),
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

    let content = response
        .choices
        .first()
        .and_then(|c| c.message.content.as_ref().and_then(|c| c.to_text_lossy()))
        .ok_or_else(|| "No content returned in response".to_string())?;

    Ok(LlmResponse {
        content,
        usage: response.usage.map(|u| hqe_core::prompt_runner::TokenUsage {
            prompt_tokens: u.prompt_tokens as u32,
            completion_tokens: u.completion_tokens as u32,
            total_tokens: u.total_tokens as u32,
        }),
        raw_model_id: response.model,
        provider_kind: profile
            .effective_kind()
            .map(|k| k.to_string())
            .unwrap_or_else(|_| "unknown".to_string()),
        system_prompt_version: runner.system_prompt_version().to_string(),
    })
}

pub async fn test_connection(
    profile_name: &str,
    session_key: Option<SecretString>,
) -> Result<bool, String> {
    let (profile, api_key) = resolve_profile(Some(profile_name.to_string()), session_key)?;
    let allow_missing_key = is_local_or_private_base_url(&profile.base_url).unwrap_or(false);
    let api_key = match api_key {
        Some(key) => key,
        None if allow_missing_key => SecretString::new(String::new().into_boxed_str()),
        None => return Err("No API key stored for profile".to_string()),
    };

    let headers = profile.sanitized_headers().map_err(|e| e.to_string())?;
    let config = hqe_openai::ClientConfig {
        base_url: profile.base_url.clone(),
        api_key,
        default_model: profile.default_model.clone(),
        headers: Some(headers),
        organization: profile.organization.clone(),
        project: profile.project.clone(),
        disable_system_proxy: false,
        timeout_seconds: profile.timeout_s,
        max_retries: 1,
        rate_limit_config: None,
        cache_enabled: true,
        daily_budget: 1.0,
    };

    let client = OpenAIClient::new(config).map_err(|e| e.to_string())?;
    client.test_connection().await.map_err(|e| e.to_string())
}

pub async fn discover_models(
    profile_name: &str,
    session_key: Option<SecretString>,
) -> Result<Vec<ModelInfo>, String> {
    let (profile, api_key) = resolve_profile(Some(profile_name.to_string()), session_key)?;
    let allow_missing_key = is_local_or_private_base_url(&profile.base_url).unwrap_or(false);
    if api_key.is_none() && !allow_missing_key {
        return Err("No API key stored for profile".to_string());
    }
    let allow_missing_key = is_local_or_private_base_url(&profile.base_url).unwrap_or(false);
    let api_key = match api_key {
        Some(key) => Some(key),
        None if allow_missing_key => None,
        None => return Err("No API key stored for profile".to_string()),
    };

    let headers: BTreeMap<String, String> = profile
        .sanitized_headers()
        .map_err(|e| e.to_string())?
        .into_iter()
        .collect();

    let client = ProviderDiscoveryClient::new(
        &profile.base_url,
        &headers,
        api_key,
        Duration::from_secs(profile.timeout_s),
        Some(hqe_openai::provider_discovery::DiskCache::default()),
    )
    .map_err(|e| e.to_string())?;

    let models = client
        .discover_chat_models()
        .await
        .map_err(|e| e.to_string())?;

    Ok(models
        .models
        .into_iter()
        .map(|m| ModelInfo {
            id: m.id.clone(),
            name: if m.name.is_empty() { m.id } else { m.name },
            context_window: m.context_length,
        })
        .collect())
}

pub fn build_scan_analyzer(
    profile_name: &str,
    venice_parameters: Option<serde_json::Value>,
    parallel_tool_calls: Option<bool>,
    session_key: Option<SecretString>,
) -> Result<(hqe_openai::OpenAIAnalyzer, hqe_openai::ProviderProfile), String> {
    let (profile, api_key) = resolve_profile(Some(profile_name.to_string()), session_key)?;
    let allow_missing_key = is_local_or_private_base_url(&profile.base_url).unwrap_or(false);
    let api_key = match api_key {
        Some(key) => key,
        None if allow_missing_key => SecretString::new(String::new().into_boxed_str()),
        None => return Err("No API key stored for profile".to_string()),
    };

    let headers = profile.sanitized_headers().map_err(|e| e.to_string())?;
    let config = hqe_openai::ClientConfig {
        base_url: profile.base_url.clone(),
        api_key,
        default_model: profile.default_model.clone(),
        headers: Some(headers),
        organization: profile.organization.clone(),
        project: profile.project.clone(),
        disable_system_proxy: false,
        timeout_seconds: profile.timeout_s,
        max_retries: 1,
        rate_limit_config: None,
        cache_enabled: true,
        daily_budget: 1.0,
    };

    let client = OpenAIClient::new(config).map_err(|e| e.to_string())?;
    let analyzer = hqe_openai::OpenAIAnalyzer::new(client)
        .with_venice_parameters(venice_parameters)
        .with_parallel_tool_calls(parallel_tool_calls);

    Ok((analyzer, profile))
}

fn resolve_profile(
    profile_name: Option<String>,
    session_key: Option<SecretString>,
) -> Result<(hqe_openai::ProviderProfile, Option<SecretString>), String> {
    let manager = ProfileManager::default();
    let name = match profile_name {
        Some(name) => name,
        None => {
            let profiles = manager.load_profiles().map_err(|e| e.to_string())?;
            let profile = profiles
                .first()
                .ok_or_else(|| "No provider profiles configured".to_string())?;
            profile.name.clone()
        }
    };
    let (profile, api_key) = manager
        .get_profile_with_key(&name)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Profile not found".to_string())?;
    Ok((profile, session_key.or(api_key)))
}

fn normalize_model_override(default_model: &str, model_override: Option<String>) -> String {
    let trimmed = model_override.as_deref().map(str::trim).unwrap_or("");
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("default") {
        default_model.to_string()
    } else {
        trimmed.to_string()
    }
}
