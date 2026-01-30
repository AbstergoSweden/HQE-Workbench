use std::{collections::BTreeMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::provider_discovery::{sanitize_base_url, sanitize_headers, ProviderKind};

/// Profile settings are stored (per docs) in:
/// `~/.local/share/hqe-workbench/profiles.json`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderProfile {
    pub name: String,
    pub base_url: String,
    /// Optional: user-selected default model for chat.
    pub model: Option<String>,
    /// Additional headers *excluding* secrets. API keys are stored in Keychain.
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    /// Optional override for provider type; otherwise auto-detected from URL.
    pub provider_kind: Option<ProviderKind>,
    /// HTTP timeout seconds for this provider.
    #[serde(default = "default_timeout_s")]
    pub timeout_s: u64,
}

fn default_timeout_s() -> u64 { 60 }

impl ProviderProfile {
    pub fn normalized_base_url(&self) -> Result<Url, ProfileError> {
        sanitize_base_url(&self.base_url).map_err(ProfileError::InvalidBaseUrl)
    }

    pub fn sanitized_headers(&self) -> Result<BTreeMap<String, String>, ProfileError> {
        sanitize_headers(&self.headers).map_err(ProfileError::InvalidHeaders)
    }

    pub fn effective_kind(&self) -> Result<ProviderKind, ProfileError> {
        if let Some(k) = self.provider_kind {
            return Ok(k);
        }
        let url = self.normalized_base_url()?;
        Ok(ProviderKind::detect(&url))
    }
}

#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("invalid base_url: {0}")]
    InvalidBaseUrl(String),
    #[error("invalid headers: {0}")]
    InvalidHeaders(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

pub trait ProfilesStore {
    fn profiles_path(&self) -> PathBuf;

    fn load_profiles(&self) -> Result<Vec<ProviderProfile>, ProfileError> {
        let p = self.profiles_path();
        if !p.exists() {
            return Ok(vec![]);
        }
        let data = fs::read_to_string(p)?;
        Ok(serde_json::from_str::<Vec<ProviderProfile>>(&data)?)
    }

    fn save_profiles(&self, profiles: &[ProviderProfile]) -> Result<(), ProfileError> {
        let p = self.profiles_path();
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(profiles)?;
        fs::write(p, data)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DefaultProfilesStore;

impl ProfilesStore for DefaultProfilesStore {
    fn profiles_path(&self) -> PathBuf {
        let mut base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        base.push("hqe-workbench");
        base.push("profiles.json");
        base
    }
}

/// API key storage abstraction.
/// The docs call for macOS Keychain.
pub trait ApiKeyStore: Send + Sync {
    fn get_api_key(&self, profile_name: &str) -> Result<Option<String>, KeyStoreError>;
    fn set_api_key(&self, profile_name: &str, api_key: &str) -> Result<(), KeyStoreError>;
    fn delete_api_key(&self, profile_name: &str) -> Result<(), KeyStoreError>;
}

#[derive(Debug)]
pub struct KeychainStore {
    service: String,
}

impl Default for KeychainStore {
    fn default() -> Self {
        Self { service: "hqe-workbench".to_string() }
    }
}

impl KeychainStore {
    fn entry(&self, profile_name: &str) -> keyring::Entry {
        let account = format!("api_key:{profile_name}");
        keyring::Entry::new(&self.service, &account).expect("keyring entry")
    }
}

#[derive(Debug, Error)]
pub enum KeyStoreError {
    #[error("keyring error: {0}")]
    Keyring(String),
}

impl ApiKeyStore for KeychainStore {
    fn get_api_key(&self, profile_name: &str) -> Result<Option<String>, KeyStoreError> {
        let e = self.entry(profile_name);
        match e.get_password() {
            Ok(pw) => Ok(Some(pw)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(KeyStoreError::Keyring(e.to_string())),
        }
    }

    fn set_api_key(&self, profile_name: &str, api_key: &str) -> Result<(), KeyStoreError> {
        let e = self.entry(profile_name);
        e.set_password(api_key).map_err(|e| KeyStoreError::Keyring(e.to_string()))
    }

    fn delete_api_key(&self, profile_name: &str) -> Result<(), KeyStoreError> {
        let e = self.entry(profile_name);
        match e.delete_password() {
            Ok(_) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(KeyStoreError::Keyring(e.to_string())),
        }
    }
}
