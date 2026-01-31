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
}

impl OpenAIAnalyzer {
    /// Create a new analyzer from an OpenAI-compatible client.
    pub fn new(client: OpenAIClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl LlmAnalyzer for OpenAIAnalyzer {
    async fn analyze(&self, bundle: EvidenceBundle) -> hqe_core::Result<AnalysisResult> {
        let prompt = build_analysis_json_prompt(&bundle);

        let response = self
            .client
            .chat(ChatRequest {
                model: self.client.default_model().to_string(),
                messages: vec![
                    Message {
                        role: Role::System,
                        content: Some(HQE_SYSTEM_PROMPT.to_string()),
                        tool_calls: None,
                    },
                    Message {
                        role: Role::User,
                        content: Some(prompt),
                        tool_calls: None,
                    },
                ],
                temperature: Some(0.2),
                max_tokens: Some(2000),
                response_format: Some(ResponseFormat::JsonObject),
            })
            .await
            .map_err(|e| HqeError::Provider(e.to_string()))?;

        let content = response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
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

fn extract_json_object(input: &str) -> Option<String> {
    if let Some(fenced) = extract_fenced_json(input) {
        return Some(fenced);
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
                            return Some(input[start..=idx].to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    None
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
