//! HQE Workbench - Tauri App Library
//!
//! This is the library entry point for the Tauri application.
//! It exports commands that can be invoked from the frontend.

pub mod chat;
pub mod commands;
pub mod prompts;
use chat::*;
use commands::*;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Application state shared across commands
pub struct AppState {
    pub current_repo: Arc<Mutex<Option<String>>>,
}

/// Run the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            current_repo: Arc::new(Mutex::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            select_folder,
            scan_repo,
            get_repo_info,
            load_report,
            export_artifacts,
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
