//! HQE Workbench - Tauri App Library
//!
//! This is the library entry point for the Tauri application.
//! It exports commands that can be invoked from the frontend.

pub mod chat;
pub mod commands;
pub mod llm;
pub mod prompts;
use chat::*;
use commands::*;
use hqe_core::encrypted_db::EncryptedDb;
use secrecy::SecretString;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::error;

/// Log an internal error and return a generic user-facing message.
///
/// This prevents leaking implementation details to the frontend while
/// preserving the full error in structured logs for debugging.
pub(crate) fn log_and_wrap_error(context: &str, error: impl std::fmt::Display) -> String {
    error!(error = %error, "{context}");
    context.to_string()
}

/// Application state shared across commands
pub struct AppState {
    pub current_repo: Arc<Mutex<Option<String>>>,
    /// Shared database instance for all chat operations
    pub db: Arc<Mutex<EncryptedDb>>,
    /// Session-only API keys (not persisted)
    pub session_keys: Arc<Mutex<HashMap<String, SecretString>>>,
}

/// Run the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize the encrypted database once at startup
    let db = EncryptedDb::init().expect("Failed to initialize encrypted database");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            current_repo: Arc::new(Mutex::new(None)),
            db: Arc::new(Mutex::new(db)),
            session_keys: Arc::new(Mutex::new(HashMap::new())),
        })
        .invoke_handler(tauri::generate_handler![
            select_folder,
            scan_repo,
            get_repo_info,
            load_report,
            export_artifacts,
            set_session_api_key,
            clear_session_api_key,
            save_provider_config,
            test_provider_connection,
            // Provider discovery commands
            discover_models,
            list_provider_profiles,
            get_provider_profile,
            save_provider_profile,
            delete_provider_profile,
            detect_provider_kind,
            import_default_profiles,
            // Prefilled provider specs
            get_provider_specs,
            apply_provider_spec,
            // Prompt commands
            prompts::get_available_prompts,
            prompts::get_available_prompts_with_metadata,
            prompts::execute_prompt,
            // Chat commands
            create_chat_session,
            list_chat_sessions,
            get_chat_session,
            get_chat_messages,
            add_chat_message,
            send_chat_message,
            delete_chat_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
