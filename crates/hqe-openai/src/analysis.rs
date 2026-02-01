//! LLM-backed analysis integration for HQE scans.

use async_trait::async_trait;
use hqe_core::scan::{AnalysisResult, LlmAnalyzer};
use hqe_core::{Blocker, EvidenceBundle, Finding, HqeError, TodoItem};
use serde::Deserialize;

use crate::prompts::{build_analysis_json_prompt, HQE_SYSTEM_PROMPT};
use crate::{ChatRequest, Message, OpenAIClient, ResponseFormat, Role};

#[derive(Debug, Default, Deserialize)]
struct LlmAnalysisPayload {
    #[serde(default)]
    findings: Vec<Finding>,
    #[serde(default)]
    todos: Vec<TodoItem>,
    #[serde(default)]
    blockers: Vec<Blocker>,
    #[serde(default)]
    is_partial: bool,
}

/// LLM-backed analyzer that returns structured findings/todos.
#[derive(Debug, Clone)]
pub struct OpenAIAnalyzer {
    client: OpenAIClient,
    venice_parameters: Option<serde_json::Value>,
    parallel_tool_calls: Option<bool>,
}

impl OpenAIAnalyzer {
    /// Create a new analyzer from an OpenAI-compatible client.
    pub fn new(client: OpenAIClient) -> Self {
        Self {
            client,
            venice_parameters: None,
            parallel_tool_calls: None,
        }
    }

    /// Attach Venice-specific parameters to chat requests.
    pub fn with_venice_parameters(mut self, params: Option<serde_json::Value>) -> Self {
        self.venice_parameters = params;
        self
    }

    /// Override parallel tool calls setting when supported by provider.
    pub fn with_parallel_tool_calls(mut self, value: Option<bool>) -> Self {
        self.parallel_tool_calls = value;
        self
    }
}

#[async_trait]
impl LlmAnalyzer for OpenAIAnalyzer {
    async fn analyze(&self, bundle: EvidenceBundle) -> hqe_core::Result<AnalysisResult> {
        let prompt = build_analysis_json_prompt(&bundle);

        let request = ChatRequest {
            model: self.client.default_model().to_string(),
            messages: vec![
                Message {
                    role: Role::System,
                    content: Some(HQE_SYSTEM_PROMPT.to_string().into()),
                    tool_calls: None,
                },
                Message {
                    role: Role::User,
                    content: Some(prompt.into()),
                    tool_calls: None,
                },
            ],
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
            max_tokens: Some(2000),
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
            venice_parameters: self.venice_parameters.clone(),
            parallel_tool_calls: self.parallel_tool_calls,
            response_format: Some(ResponseFormat::JsonObject),
        };

        let response = match self.client.chat(request.clone()).await {
            Ok(resp) => resp,
            Err(err) => {
                let message = err.to_string();
                if should_retry_without_format(&message) {
                    let mut fallback = request;
                    fallback.response_format = None;
                    self.client
                        .chat(fallback)
                        .await
                        .map_err(|e| HqeError::Provider(e.to_string()))?
                } else {
                    return Err(HqeError::Provider(message));
                }
            }
        };

        let content = response
            .choices
            .first()
            .and_then(|c| c.message.content.as_ref().and_then(|c| c.to_text_lossy()))
            .ok_or_else(|| HqeError::Provider("Empty response content".to_string()))?;

        let json_str = extract_json_object(&content)
            .ok_or_else(|| HqeError::Provider("No JSON object found in response".to_string()))?;

        let payload: LlmAnalysisPayload = serde_json::from_str(&json_str)
            .map_err(|e| HqeError::Provider(format!("Failed to parse JSON: {e}")))?;

        Ok(AnalysisResult {
            findings: payload.findings,
            todos: payload.todos,
            is_partial: payload.is_partial,
            blockers: payload.blockers,
        })
    }
}

fn should_retry_without_format(error: &str) -> bool {
    let msg = error.to_lowercase();
    msg.contains("response_format")
        || msg.contains("json_schema")
        || msg.contains("json object")
        || msg.contains("json_object")
        || msg.contains("unsupported")
        || msg.contains("not supported")
}

fn extract_json_object(input: &str) -> Option<String> {
    if let Some(fenced) = extract_fenced_json(input) {
        return validate_and_extract_json(&fenced);
    }

    let mut in_string = false;
    let mut escape = false;
    let mut depth = 0usize;
    let mut start_idx: Option<usize> = None;

    for (idx, ch) in input.char_indices() {
        if in_string {
            if escape {
                escape = false;
                continue;
            }
            match ch {
                '\\' => escape = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => {
                if depth == 0 {
                    start_idx = Some(idx);
                }
                depth = depth.saturating_add(1);
            }
            '}' => {
                if depth > 0 {
                    depth -= 1;
                    if depth == 0 {
                        if let Some(start) = start_idx {
                            let extracted = input[start..=idx].to_string();
                            return validate_and_extract_json(&extracted);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    None
}

/// Validate and extract JSON after initial extraction to prevent injection
fn validate_and_extract_json(json_str: &str) -> Option<String> {
    // First, try to parse the JSON to ensure it's valid
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
        // Check for suspicious patterns that might indicate injection
        if contains_suspicious_patterns(&value) {
            tracing::warn!("Suspicious patterns detected in LLM response JSON, rejecting");
            return None;
        }

        // Return the validated JSON
        Some(json_str.to_string())
    } else {
        None
    }
}

/// Check for suspicious patterns in the parsed JSON that might indicate injection
fn contains_suspicious_patterns(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::String(s) => {
            // Check for common prompt injection patterns in string values
            let lower = s.to_lowercase();
            lower.contains("[system") ||
            lower.contains("ignore") ||
            lower.contains("disregard") ||
            lower.contains("nevermind") ||
            lower.contains("actually") ||
            lower.contains("instead") ||
            s.contains("{{") ||  // Template injection
            s.contains("{%") ||  // Template injection
            s.contains("{#") // Template injection
        }
        serde_json::Value::Array(arr) => arr.iter().any(contains_suspicious_patterns),
        serde_json::Value::Object(obj) => obj.values().any(contains_suspicious_patterns),
        _ => false,
    }
}

fn extract_fenced_json(input: &str) -> Option<String> {
    let fence_start = input.find("```json")?;
    let after = &input[fence_start + "```json".len()..];
    let fence_end = after.find("```")?;
    let candidate = after[..fence_end].trim();
    if candidate.is_empty() {
        None
    } else {
        Some(candidate.to_string())
    }
}
