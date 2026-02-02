//! Chat Tauri Commands
//!
//! Provides chat session management, message handling, and persistence
//! via the encrypted local database.

use hqe_core::encrypted_db::{
    ChatMessage, ChatOperations, ChatSession, EncryptedDb, MessageRole,
};
use serde::{Deserialize, Serialize};
use tauri::command;
use tracing::{debug, error, info};
use uuid::Uuid;

/// Chat session DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSessionDto {
    pub id: String,
    pub repo_path: Option<String>,
    pub prompt_id: Option<String>,
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

/// Create a new chat session
#[command]
pub async fn create_chat_session(
    repo_path: Option<String>,
    prompt_id: Option<String>,
    provider: String,
    model: String,
) -> Result<ChatSessionDto, String> {
    info!(repo_path = ?repo_path, provider = %provider, "Creating chat session");

    let db = EncryptedDb::init().map_err(|e| e.to_string())?;

    let session = ChatSession {
        id: Uuid::new_v4().to_string(),
        repo_path,
        prompt_id,
        provider,
        model,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        metadata: None,
    };

    db.create_session(&session).map_err(|e| e.to_string())?;

    Ok(ChatSessionDto {
        id: session.id,
        repo_path: session.repo_path,
        prompt_id: session.prompt_id,
        provider: session.provider,
        model: session.model,
        created_at: session.created_at.to_rfc3339(),
        updated_at: session.updated_at.to_rfc3339(),
        message_count: 0,
    })
}

/// List chat sessions
#[command]
pub async fn list_chat_sessions(repo_path: Option<String>) -> Result<Vec<ChatSessionDto>, String> {
    debug!(repo_path = ?repo_path, "Listing chat sessions");

    let db = EncryptedDb::init().map_err(|e| e.to_string())?;

    let sessions = db
        .list_sessions(repo_path.as_deref())
        .map_err(|e| e.to_string())?;

    let dtos: Vec<ChatSessionDto> = sessions
        .into_iter()
        .map(|s| ChatSessionDto {
            id: s.id.clone(),
            repo_path: s.repo_path,
            prompt_id: s.prompt_id,
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
pub async fn get_chat_session(session_id: String) -> Result<(ChatSessionDto, Vec<ChatMessageDto>), String> {
    debug!(session_id = %session_id, "Getting chat session");

    let db = EncryptedDb::init().map_err(|e| e.to_string())?;

    let session = db
        .get_session(&session_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Session not found".to_string())?;

    let messages = db.get_messages(&session_id).map_err(|e| e.to_string())?;

    let session_dto = ChatSessionDto {
        id: session.id.clone(),
        repo_path: session.repo_path,
        prompt_id: session.prompt_id,
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
    session_id: String,
    role: String,
    content: String,
    parent_id: Option<String>,
) -> Result<ChatMessageDto, String> {
    debug!(session_id = %session_id, role = %role, "Adding chat message");

    let db = EncryptedDb::init().map_err(|e| e.to_string())?;

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

/// Delete a chat session
#[command]
pub async fn delete_chat_session(session_id: String) -> Result<(), String> {
    info!(session_id = %session_id, "Deleting chat session");

    let db = EncryptedDb::init().map_err(|e| e.to_string())?;
    db.delete_session(&session_id).map_err(|e| e.to_string())?;

    Ok(())
}

/// Get available provider specs
#[command]
pub async fn get_provider_specs() -> Result<Vec<serde_json::Value>, String> {
    use hqe_openai::prefilled::{all_specs, AuthScheme};

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
    use hqe_openai::profile::{DefaultProfilesStore, ProfileManager, ProfilesStore};
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
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "profile_name": profile_name,
        "spec_id": spec_id,
        "success": true,
    }))
}
