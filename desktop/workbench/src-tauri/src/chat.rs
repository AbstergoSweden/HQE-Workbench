//! Chat Tauri Commands
//!
//! Provides chat session management, message handling, and persistence
//! via the encrypted local database.

use crate::llm::run_llm;
use hqe_core::encrypted_db::{ChatMessage, ChatOperations, ChatSession, MessageRole};
use hqe_core::prompt_runner::{
    Compatibility, ContentType, InputSpec, InputType, PromptCategory, PromptExecutionRequest,
    PromptTemplate, UntrustedContext,
};
use hqe_core::redaction::RedactionEngine;
use hqe_core::repo::RepoScanner;
use serde::{Deserialize, Serialize};
use tauri::command;
use tracing::{debug, info};
use uuid::Uuid;

/// Chat session DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSessionDto {
    pub id: String,
    pub repo_path: Option<String>,
    pub prompt_id: Option<String>,
    pub name: String,
    pub provider: String,
    pub model: String,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: usize,
}

/// Chat message DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageDto {
    pub id: String,
    pub session_id: String,
    pub parent_id: Option<String>,
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendChatMessageResponse {
    pub user_message: ChatMessageDto,
    pub assistant_message: ChatMessageDto,
}

const MAX_CONTEXT_BYTES: usize = 100_000;
const MAX_CONTEXT_FILES: usize = 50;
const MAX_HISTORY_CHARS: usize = 8_000;

/// Create a new chat session
#[command]
pub async fn create_chat_session(
    state: tauri::State<'_, crate::AppState>,
    repo_path: Option<String>,
    prompt_id: Option<String>,
    provider: String,
    model: String,
) -> Result<ChatSessionDto, String> {
    let db = state.db.lock().await;
    let now = chrono::Utc::now();
    let session = ChatSession {
        id: Uuid::new_v4().to_string(),
        repo_path,
        prompt_id,
        name: "New Chat".to_string(),
        provider,
        model,
        created_at: now,
        updated_at: now,
        metadata: None,
    };

    db.create_session(&session)
        .map_err(|e: hqe_core::encrypted_db::EncryptedDbError| e.to_string())?;

    Ok(ChatSessionDto {
        id: session.id,
        repo_path: session.repo_path,
        prompt_id: session.prompt_id,
        name: session.name,
        provider: session.provider,
        model: session.model,
        created_at: session.created_at.to_rfc3339(),
        updated_at: session.updated_at.to_rfc3339(),
        message_count: 0,
    })
}

/// List chat sessions
#[command]
pub async fn list_chat_sessions(
    state: tauri::State<'_, crate::AppState>,
    repo_path: Option<String>,
) -> Result<Vec<ChatSessionDto>, String> {
    debug!(repo_path = ?repo_path, "Listing chat sessions");

    let db = state.db.lock().await;

    let sessions = db
        .list_sessions(repo_path.as_deref())
        .map_err(|e| e.to_string())?;

    let dtos: Vec<ChatSessionDto> = sessions
        .into_iter()
        .map(|s| ChatSessionDto {
            id: s.id.clone(),
            repo_path: s.repo_path,
            prompt_id: s.prompt_id,
            name: s.name,
            provider: s.provider,
            model: s.model,
            created_at: s.created_at.to_rfc3339(),
            updated_at: s.updated_at.to_rfc3339(),
            message_count: db.get_messages(&s.id).map(|m| m.len()).unwrap_or(0),
        })
        .collect();

    Ok(dtos)
}

/// Get a single chat session with messages
#[command]
pub async fn get_chat_session(
    state: tauri::State<'_, crate::AppState>,
    session_id: String,
) -> Result<(ChatSessionDto, Vec<ChatMessageDto>), String> {
    debug!(session_id = %session_id, "Getting chat session");

    let db = state.db.lock().await;

    let session = db
        .get_session(&session_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Session not found".to_string())?;

    let messages = db.get_messages(&session_id).map_err(|e| e.to_string())?;

    let session_dto = ChatSessionDto {
        id: session.id.clone(),
        repo_path: session.repo_path,
        prompt_id: session.prompt_id,
        name: session.name,
        provider: session.provider,
        model: session.model,
        created_at: session.created_at.to_rfc3339(),
        updated_at: session.updated_at.to_rfc3339(),
        message_count: messages.len(),
    };

    let message_dtos: Vec<ChatMessageDto> = messages
        .into_iter()
        .map(|m| ChatMessageDto {
            id: m.id,
            session_id: m.session_id,
            parent_id: m.parent_id,
            role: match m.role {
                MessageRole::System => "system".to_string(),
                MessageRole::User => "user".to_string(),
                MessageRole::Assistant => "assistant".to_string(),
                MessageRole::Tool => "tool".to_string(),
            },
            content: m.content,
            timestamp: m.timestamp.to_rfc3339(),
        })
        .collect();

    Ok((session_dto, message_dtos))
}

/// Add a message to a chat session
#[command]
pub async fn add_chat_message(
    state: tauri::State<'_, crate::AppState>,
    session_id: String,
    role: String,
    content: String,
    parent_id: Option<String>,
) -> Result<ChatMessageDto, String> {
    debug!(session_id = %session_id, role = %role, "Adding chat message");

    let db = state.db.lock().await;

    let role_enum = match role.as_str() {
        "system" => MessageRole::System,
        "user" => MessageRole::User,
        "assistant" => MessageRole::Assistant,
        "tool" => MessageRole::Tool,
        _ => MessageRole::User,
    };

    let message = ChatMessage {
        id: Uuid::new_v4().to_string(),
        session_id: session_id.clone(),
        parent_id,
        role: role_enum,
        content: content.clone(),
        context_refs: None,
        timestamp: chrono::Utc::now(),
        metadata: None,
    };

    db.add_message(&message).map_err(|e| e.to_string())?;

    Ok(ChatMessageDto {
        id: message.id,
        session_id: message.session_id,
        parent_id: message.parent_id,
        role,
        content,
        timestamp: message.timestamp.to_rfc3339(),
    })
}

/// Get messages for a session
#[command]
pub async fn get_chat_messages(
    state: tauri::State<'_, crate::AppState>,
    session_id: String,
) -> Result<Vec<ChatMessageDto>, String> {
    debug!(session_id = %session_id, "Getting chat messages");

    let db = state.db.lock().await;
    let messages = db.get_messages(&session_id).map_err(|e| e.to_string())?;

    let dtos: Vec<ChatMessageDto> = messages
        .into_iter()
        .map(|m| ChatMessageDto {
            id: m.id,
            session_id: m.session_id,
            parent_id: m.parent_id,
            role: match m.role {
                MessageRole::System => "system".to_string(),
                MessageRole::User => "user".to_string(),
                MessageRole::Assistant => "assistant".to_string(),
                MessageRole::Tool => "tool".to_string(),
            },
            content: m.content,
            timestamp: m.timestamp.to_rfc3339(),
        })
        .collect();

    Ok(dtos)
}

/// Send a chat message and get response
#[command]
pub async fn send_chat_message(
    state: tauri::State<'_, crate::AppState>,
    session_id: String,
    content: String,
    parent_id: Option<String>,
) -> Result<SendChatMessageResponse, String> {
    info!(session_id = %session_id, "Sending chat message");

    let db = state.db.lock().await;

    let session = db
        .get_session(&session_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Session not found".to_string())?;

    // Add user message
    let user_message = ChatMessage {
        id: Uuid::new_v4().to_string(),
        session_id: session_id.clone(),
        parent_id: parent_id.clone(),
        role: MessageRole::User,
        content: content.clone(),
        context_refs: None,
        timestamp: chrono::Utc::now(),
        metadata: None,
    };

    db.add_message(&user_message).map_err(|e| e.to_string())?;

    let prompt_template = if let Some(prompt_id) = &session.prompt_id {
        let mut registry = load_prompt_registry()?;
        registry.load_all().map_err(|e| e.to_string())?;
        let prompt = registry
            .get(prompt_id)
            .ok_or_else(|| format!("Prompt '{}' not found", prompt_id))?;

        PromptTemplate {
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
        }
    } else {
        PromptTemplate {
            id: "chat".to_string(),
            title: "Chat".to_string(),
            category: PromptCategory::Custom,
            description: "Chat follow-up".to_string(),
            version: "1.0.0".to_string(),
            template: "{{message}}".to_string(),
            required_inputs: vec![InputSpec {
                name: "message".to_string(),
                description: "User message".to_string(),
                input_type: InputType::String,
                required: true,
                default: None,
                validation: None,
            }],
            compatibility: Compatibility::default(),
            allowed_tools: vec![],
        }
    };

    let context = if let Some(repo_path) = &session.repo_path {
        load_repo_context(repo_path).await?
    } else {
        Vec::new()
    };

    let history_messages = db.get_messages(&session_id).map_err(|e| e.to_string())?;
    let mut inputs = build_inputs(&content, &prompt_template);
    let user_message_payload = build_user_message(&history_messages, &content);

    let execution_request = PromptExecutionRequest {
        prompt_template,
        user_message: user_message_payload,
        inputs,
        context,
        max_context_size: None,
    };

    let session_key = {
        let keys = state.session_keys.lock().await;
        keys.get(&session.provider).cloned()
    };
    let response = run_llm(
        execution_request,
        Some(session.provider.clone()),
        session_key,
        Some(session.model.clone()),
    )
    .await?;

    let assistant_message = ChatMessage {
        id: Uuid::new_v4().to_string(),
        session_id: session_id.clone(),
        parent_id: Some(user_message.id.clone()),
        role: MessageRole::Assistant,
        content: response.content,
        context_refs: Some(
            context
                .iter()
                .map(|ctx| hqe_core::encrypted_db::ContextRef {
                    file_path: ctx.source.clone(),
                    line_start: None,
                    line_end: None,
                    snippet: None,
                })
                .collect(),
        ),
        timestamp: chrono::Utc::now(),
        metadata: None,
    };

    db.add_message(&assistant_message)
        .map_err(|e| e.to_string())?;

    let user_dto = ChatMessageDto {
        id: user_message.id,
        session_id: user_message.session_id,
        parent_id: user_message.parent_id,
        role: "user".to_string(),
        content: user_message.content,
        timestamp: user_message.timestamp.to_rfc3339(),
    };

    let assistant_dto = ChatMessageDto {
        id: assistant_message.id,
        session_id: assistant_message.session_id,
        parent_id: assistant_message.parent_id,
        role: "assistant".to_string(),
        content: assistant_message.content,
        timestamp: assistant_message.timestamp.to_rfc3339(),
    };

    Ok(SendChatMessageResponse {
        user_message: user_dto,
        assistant_message: assistant_dto,
    })
}

fn build_inputs(
    content: &str,
    prompt_template: &PromptTemplate,
) -> std::collections::HashMap<String, String> {
    let mut inputs = std::collections::HashMap::new();
    inputs.insert("message".to_string(), content.to_string());
    if prompt_template.required_inputs.iter().any(|input| input.name == "args") {
        inputs.insert("args".to_string(), content.to_string());
    }
    inputs
}

fn build_user_message(history: &[ChatMessage], content: &str) -> String {
    let history_block = build_history_block(history);
    if history_block.is_empty() {
        content.to_string()
    } else {
        format!(
            "--- BEGIN CONVERSATION HISTORY ---\n{}\n--- END CONVERSATION HISTORY ---\n\nUser message:\n{}",
            history_block, content
        )
    }
}

fn build_history_block(messages: &[ChatMessage]) -> String {
    let mut lines = Vec::new();
    let mut total = 0usize;

    for message in messages.iter().rev() {
        let role = match message.role {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::Tool => "tool",
            MessageRole::System => continue,
        };
        let entry = format!("{}: {}", role, message.content.trim());
        if total + entry.len() > MAX_HISTORY_CHARS {
            break;
        }
        total += entry.len();
        lines.push(entry);
    }

    lines.reverse();
    lines.join("\n")
}

async fn load_repo_context(repo_path: &str) -> Result<Vec<UntrustedContext>, String> {
    let scanner = RepoScanner::new(repo_path);
    let scan = scanner.scan().map_err(|e| e.to_string())?;
    let mut redaction = RedactionEngine::new();
    let mut contexts = Vec::new();
    let mut total_size = 0usize;

    for file in scan.files.into_iter().take(MAX_CONTEXT_FILES) {
        if total_size >= MAX_CONTEXT_BYTES {
            break;
        }
        if let Some(content) = scanner.read_file(&file).await.map_err(|e| e.to_string())? {
            let redacted = redaction.redact(&content);
            let size = redacted.len();
            if total_size + size > MAX_CONTEXT_BYTES {
                break;
            }
            total_size += size;
            contexts.push(UntrustedContext {
                source: file.clone(),
                content_type: detect_content_type(&file),
                content: redacted,
                size_bytes: size,
            });
        }
    }

    Ok(contexts)
}

fn detect_content_type(path: &str) -> ContentType {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "md" | "markdown" | "txt" | "rst" | "adoc" => ContentType::Documentation,
        "toml" | "yaml" | "yml" | "json" => ContentType::Configuration,
        "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "java" | "kt" | "swift"
        | "rb" | "php" | "c" | "cpp" | "cc" | "h" | "hpp" => ContentType::SourceCode,
        "test" => ContentType::TestFile,
        _ => ContentType::Unknown,
    }
}

fn load_prompt_registry() -> Result<hqe_mcp::registry_v2::PromptRegistry, String> {
    let prompts_dir = resolve_prompts_dir()?;
    let loader = hqe_mcp::PromptLoader::new(&prompts_dir);
    Ok(hqe_mcp::registry_v2::PromptRegistry::new(loader))
}

fn resolve_prompts_dir() -> Result<std::path::PathBuf, String> {
    if let Ok(dir) = std::env::var("HQE_PROMPTS_DIR") {
        let path = std::path::PathBuf::from(dir);
        if path.exists() {
            return path.canonicalize().or(Ok(path)).map_err(|e| e.to_string());
        }
    }

    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    for ancestor in cwd.ancestors() {
        let cli_library = ancestor.join("mcp-server").join("cli-prompt-library");
        if cli_library.exists() {
            return Ok(cli_library);
        }

        let prompts_dir = ancestor.join("prompts");
        if prompts_dir.exists() {
            return Ok(prompts_dir);
        }
    }

    Err("Could not locate prompts directory".to_string())
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

/// Delete a chat session
#[command]
pub async fn delete_chat_session(
    state: tauri::State<'_, crate::AppState>,
    session_id: String,
) -> Result<(), String> {
    info!(session_id = %session_id, "Deleting chat session");

    let db = state.db.lock().await;
    db.delete_session(&session_id).map_err(|e| e.to_string())?;

    Ok(())
}

/// Get available provider specs
#[command]
pub async fn get_provider_specs() -> Result<Vec<serde_json::Value>, String> {
    use hqe_openai::prefilled::all_specs;

    let specs = all_specs();
    let json_specs: Vec<serde_json::Value> = specs
        .into_iter()
        .map(|s| {
            serde_json::json!({
                "id": s.id,
                "display_name": s.display_name,
                "base_url": s.base_url,
                "auth_scheme": format!("{:?}", s.auth_scheme).to_lowercase(),
                "default_model": s.default_model,
                "default_headers": s.default_headers,
                "recommended_timeout_s": s.recommended_timeout_s,
                "quirks": s.quirks,
                "website_url": s.website_url,
                "docs_url": s.docs_url,
                "supports_streaming": s.supports_streaming,
                "supports_tools": s.supports_tools,
            })
        })
        .collect();

    Ok(json_specs)
}

/// Apply a prefilled provider spec to create/update a profile
#[command]
pub async fn apply_provider_spec(
    spec_id: String,
    api_key: String,
    profile_name: Option<String>,
) -> Result<serde_json::Value, String> {
    use hqe_openai::prefilled::get_spec;
    use hqe_openai::profile::{ProfileError, ProfileManager};
    use hqe_protocol::models::ProviderProfile;

    info!(spec_id = %spec_id, "Applying provider spec");

    let spec = get_spec(&spec_id).ok_or_else(|| "Spec not found".to_string())?;

    let profile_name = profile_name.unwrap_or_else(|| spec.display_name.clone());

    let profile = ProviderProfile {
        name: profile_name.clone(),
        base_url: spec.base_url,
        api_key_id: format!("api_key:{}", profile_name),
        default_model: spec.default_model,
        headers: Some(spec.default_headers),
        organization: None,
        project: None,
        provider_kind: Some(spec.kind),
        timeout_s: spec.recommended_timeout_s,
    };

    let manager = ProfileManager::default();
    manager
        .save_profile(profile, Some(&api_key))
        .map_err(|e: ProfileError| e.to_string())?;

    Ok(serde_json::json!({
        "profile_name": profile_name,
        "spec_id": spec_id,
        "success": true,
    }))
}
