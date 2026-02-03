//! Tauri commands for the workbench UI

use crate::AppState;
use hqe_artifacts::ArtifactWriter;
use hqe_core::models::*;
use hqe_core::scan::ScanPipeline;
use hqe_openai::profile::{
    ApiKeyStore, DefaultProfilesStore, KeychainStore, ProfileManager, ProfilesStore,
    ProviderProfile, ProviderProfileExt,
};
use hqe_openai::provider_discovery::{ProviderKind, ProviderModelList};
use hqe_openai::ProviderKindExt;
use secrecy::SecretString;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{command, Manager, State};
use tauri_plugin_dialog::DialogExt;
use url::Url;

/// Select a folder using native dialog
#[command]
pub async fn select_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let path = app
        .dialog()
        .file()
        .set_title("Select Repository")
        .blocking_pick_folder();

    Ok(path.map(|p| p.to_string()))
}

/// Scan a repository
#[command]
pub async fn scan_repo(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    repo_path: String,
    config: ScanConfig,
) -> Result<HqeReport, String> {
    let path = PathBuf::from(&repo_path);

    // Validate the path to prevent directory traversal and ensure it's a valid repository
    if let Err(e) = validate_repo_path(&path) {
        return Err(format!("Invalid repository path: {}", e));
    }

    if !path.exists() {
        return Err("Repository path does not exist".to_string());
    }

    // Update current repo
    {
        let mut current = state.current_repo.lock().await;
        *current = Some(repo_path.clone());
    }

    // Run scan
    let mut pipeline = ScanPipeline::new(&path, config.clone()).map_err(|e| e.to_string())?;
    if config.llm_enabled && !config.local_only {
        let profile_name = config
            .provider_profile
            .clone()
            .ok_or_else(|| "Provider profile required for LLM scans".to_string())?;

        let session_key = {
            let keys = state.session_keys.lock().await;
            keys.get(&profile_name).cloned()
        };

        let (analyzer, profile) = crate::llm::build_scan_analyzer(
            &profile_name,
            config.venice_parameters.clone(),
            config.parallel_tool_calls,
            session_key,
        )?;

        pipeline.set_provider_info(ProviderInfo {
            name: profile.name.clone(),
            base_url: Some(profile.base_url.clone()),
            model: Some(profile.default_model.clone()),
            llm_enabled: true,
        });

        pipeline = pipeline.with_llm_analyzer(Arc::new(analyzer));
    }

    let result = pipeline.run().await.map_err(|e| e.to_string())?;

    let output_root = get_output_root(&app)?;
    std::fs::create_dir_all(&output_root).map_err(|e| e.to_string())?;

    let run_dir = output_root.join(format!("hqe_run_{}", result.manifest.run_id));
    let writer = ArtifactWriter::new(&run_dir);
    writer.write_all(&result).await.map_err(|e| e.to_string())?;

    Ok(result.report)
}

/// Validate repository path to prevent security issues
fn validate_repo_path(path: &Path) -> Result<(), String> {
    // Check for path traversal attempts
    if path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err("Path contains parent directory references ('..')".to_string());
    }

    // Resolve the path to ensure it's absolute and within allowed boundaries
    let canonical_path = path
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize path: {}", e))?;

    // Get the home directory to establish a reasonable boundary
    if let Some(home_dir) = dirs::home_dir() {
        let canonical_home = home_dir
            .canonicalize()
            .map_err(|e| format!("Failed to canonicalize home directory: {}", e))?;

        if !canonical_path.starts_with(&canonical_home) && !canonical_path.starts_with("/tmp") {
            return Err(
                "Repository path must be within home directory or temporary directory".to_string(),
            );
        }
    }

    // Additional checks could be added here based on specific requirements
    Ok(())
}

/// Get repository info
#[command]
pub async fn get_repo_info(repo_path: String) -> Result<RepoInfo, String> {
    let path = PathBuf::from(&repo_path);

    // Check if git repo
    let is_git = path.join(".git").exists();

    // Get git info if available
    let (remote, commit) = if is_git {
        let repo = hqe_git::GitRepo::open(&path)
            .await
            .map_err(|e| e.to_string())?;

        let remote = repo.remote_url("origin").await.ok().flatten();
        let commit = repo.current_commit().await.ok();

        (remote, commit)
    } else {
        (None, None)
    };

    Ok(RepoInfo {
        source: if is_git {
            RepoSource::Git
        } else {
            RepoSource::Local
        },
        path: repo_path,
        git_remote: remote,
        git_commit: commit,
    })
}

/// Load a report by run ID
#[command]
pub async fn load_report(
    app: tauri::AppHandle,
    run_id: String,
) -> Result<Option<HqeReport>, String> {
    // Validate the run_id to prevent path traversal
    if !is_valid_run_id(&run_id) {
        return Err("Invalid run ID format".to_string());
    }

    // Search for the report in default output directory
    let output_dir = get_output_root(&app)?.join(format!("hqe_run_{}", run_id));

    if !output_dir.exists() {
        return Ok(None);
    }

    let report_path = output_dir.join("report.json");

    if !report_path.exists() {
        return Ok(None);
    }

    // Canonicalize the path to prevent path traversal
    let canonical_path = report_path
        .canonicalize()
        .map_err(|_| "Report not found".to_string())?;

    // Verify the canonical path is within the expected directory
    let expected_prefix = get_output_root(&app)?
        .canonicalize()
        .map_err(|e| format!("Could not canonicalize output directory: {}", e))?;

    if !canonical_path.starts_with(&expected_prefix) {
        return Err("Invalid report path".to_string());
    }

    let content = tokio::fs::read_to_string(&canonical_path)
        .await
        .map_err(|e| e.to_string())?;

    let report: HqeReport = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    Ok(Some(report))
}

/// Validate run ID format to prevent path traversal and other attacks
fn is_valid_run_id(run_id: &str) -> bool {
    // Run IDs should match the expected format (timestamp + UUID-like string)
    // e.g., "2023-10-01T12-30-45Z_1a2b3c4d" or similar
    // Only allow alphanumeric characters, dots, underscores, and hyphens
    run_id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-')
}

/// Export artifacts
#[command]
pub async fn export_artifacts(
    app: tauri::AppHandle,
    run_id: String,
    target_dir: String,
) -> Result<(), String> {
    if !is_valid_run_id(&run_id) {
        return Err("Invalid run ID format".to_string());
    }

    let source = get_output_root(&app)?.join(format!("hqe_run_{}", run_id));

    let target = PathBuf::from(target_dir);

    if !source.exists() {
        return Err("Artifacts not found for run ID".to_string());
    }

    let canonical_source = source
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize source: {}", e))?;
    let canonical_root = get_output_root(&app)?
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize output root: {}", e))?;

    if !canonical_source.starts_with(&canonical_root) {
        return Err("Invalid artifact source path".to_string());
    }

    tokio::fs::create_dir_all(&target)
        .await
        .map_err(|e| e.to_string())?;

    // Copy artifacts
    for entry in std::fs::read_dir(&canonical_source).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        let target_file = target.join(entry.file_name());

        tokio::fs::copy(entry.path(), target_file)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn get_output_root(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let base = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(base.join("hqe-output"))
}

// ============================================================================
// Provider Profile Management Commands
// ============================================================================

/// Discover models from a provider
#[command]
pub async fn discover_models(
    state: State<'_, AppState>,
    profile_name: String,
) -> Result<ProviderModelList, String> {
    let manager = ProfileManager::default();
    let profile = manager
        .get_profile_with_key(&profile_name)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Profile not found".to_string())?
        .0;

    let session_key = {
        let keys = state.session_keys.lock().await;
        keys.get(&profile_name).cloned()
    };
    let normalized_url = profile.normalized_base_url().map_err(|e| e.to_string())?;
    let kind = profile
        .provider_kind
        .unwrap_or_else(|| ProviderKind::detect(&normalized_url));

    let models = crate::llm::discover_models(&profile.name, session_key).await?;

    Ok(ProviderModelList {
        provider_kind: kind,
        base_url: profile.base_url.clone(),
        fetched_at_unix_s: chrono::Utc::now().timestamp() as u64,
        models: models
            .into_iter()
            .map(|m| hqe_openai::provider_discovery::DiscoveredModel {
                id: m.id.clone(),
                name: m.name.clone(),
                provider_kind: kind,
                model_type: None,
                context_length: m.context_window,
                traits: Default::default(),
                pricing: hqe_openai::provider_discovery::ProviderModelPricing {
                    input_usd_per_million: None,
                    output_usd_per_million: None,
                },
            })
            .collect(),
    })
}

/// List all saved provider profiles
#[command]
pub async fn list_provider_profiles() -> Result<Vec<ProviderProfile>, String> {
    let store = DefaultProfilesStore;
    store.load_profiles().map_err(|e| e.to_string())
}

/// Import default profiles (only adds new ones, doesn't overwrite existing)
#[command]
pub async fn import_default_profiles(profiles: Vec<ProviderProfile>) -> Result<usize, String> {
    let manager = ProfileManager::default();
    let existing = manager
        .load_profiles()
        .map_err(|e: hqe_openai::profile::ProfileError| e.to_string())?;
    let existing_names: std::collections::HashSet<&str> =
        existing.iter().map(|p| p.name.as_str()).collect();

    let mut imported = 0;
    for profile in profiles {
        if !existing_names.contains(profile.name.as_str()) {
            manager
                .save_profile(profile, None)
                .map_err(|e| e.to_string())?;
            imported += 1;
        }
    }
    Ok(imported)
}

/// Get a single provider profile with key presence indicator (not the key itself)
#[command]
pub async fn get_provider_profile(name: String) -> Result<Option<(ProviderProfile, bool)>, String> {
    let store = DefaultProfilesStore;
    let key_store = KeychainStore::default();

    let profile = store.get_profile(&name).map_err(|e| e.to_string())?;

    match profile {
        Some(p) => {
            let has_key = key_store
                .get_api_key(&name)
                .map_err(|e| e.to_string())?
                .is_some();
            Ok(Some((p, has_key)))
        }
        None => Ok(None),
    }
}

/// Save a provider profile with optional API key
#[command]
pub async fn save_provider_profile(
    profile: ProviderProfile,
    api_key: Option<String>,
) -> Result<(), String> {
    let manager = ProfileManager::default();
    manager
        .save_profile(profile, api_key.as_deref())
        .map_err(|e| e.to_string())
}

/// Delete a provider profile and its API key
#[command]
pub async fn delete_provider_profile(name: String) -> Result<bool, String> {
    let manager = ProfileManager::default();
    manager.delete_profile(&name).map_err(|e| e.to_string())
}

/// Test provider connection using a stored profile
#[command]
pub async fn test_provider_connection(
    state: State<'_, AppState>,
    profile_name: String,
) -> Result<bool, String> {
    let session_key = {
        let keys = state.session_keys.lock().await;
        keys.get(&profile_name).cloned()
    };
    crate::llm::test_connection(&profile_name, session_key).await
}

/// Store a session-only API key (not persisted)
#[command]
pub async fn set_session_api_key(
    state: State<'_, AppState>,
    profile_name: String,
    api_key: String,
) -> Result<(), String> {
    let mut keys = state.session_keys.lock().await;
    keys.insert(profile_name, SecretString::new(api_key));
    Ok(())
}

/// Clear a session-only API key
#[command]
pub async fn clear_session_api_key(
    state: State<'_, AppState>,
    profile_name: String,
) -> Result<(), String> {
    let mut keys = state.session_keys.lock().await;
    keys.remove(&profile_name);
    Ok(())
}

/// Detect provider kind from a URL
#[command]
pub async fn detect_provider_kind(url: String) -> Result<String, String> {
    let url = Url::parse(&url).map_err(|e| format!("Invalid URL: {}", e))?;
    let kind = hqe_openai::provider_discovery::ProviderKind::detect(&url);
    Ok(kind.to_string())
}

// ============================================================================
// Legacy Provider Config Commands (maintained for backward compatibility)
// ============================================================================

/// Save provider configuration (legacy - uses old ProviderProfile structure)
#[command]
pub async fn save_provider_config(
    profile: LegacyProviderProfile,
    api_key: String,
) -> Result<(), String> {
    // Convert to new format
    let new_profile = ProviderProfile {
        name: profile.name.clone(),
        base_url: profile.base_url,
        api_key_id: profile.api_key_id, // Assuming this is the correct field
        default_model: profile.default_model,
        headers: Some(std::collections::HashMap::new()),
        organization: profile.organization,
        project: profile.project,
        provider_kind: None,
        timeout_s: 60,
    };

    // Store using new manager
    let manager = ProfileManager::default();
    manager
        .save_profile(new_profile, Some(&api_key))
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Legacy provider profile structure for backward compatibility
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LegacyProviderProfile {
    pub name: String,
    pub base_url: String,
    pub api_key_id: String,
    pub default_model: String,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub organization: Option<String>,
    pub project: Option<String>,
}
