//! Provider Profile Management and API Key Storage
//!
//! Provides:
//! - Profile configuration (base_url, headers, timeouts)
//! - Secure API key storage via macOS Keychain
//! - Persistent profile storage in ~/.local/share/hqe-workbench/

use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::PathBuf,
};

use secrecy::SecretString;
use thiserror::Error;
use tracing::{debug, info, instrument, warn};
use url::Url;

use crate::provider_discovery::{
    sanitize_base_url, sanitize_headers, DiscoveryError, ProviderKind, ProviderKindExt,
};
// Re-export ProviderProfile from hqe-protocol
pub use hqe_protocol::models::ProviderProfile;

/// Extension trait for ProviderProfile with additional methods
pub trait ProviderProfileExt {
    /// Get the normalized base URL
    fn normalized_base_url(&self) -> Result<Url, ProfileError>;
    /// Get sanitized headers
    fn sanitized_headers(&self) -> Result<HashMap<String, String>, ProfileError>;
    /// Get the effective provider kind (auto-detected if not overridden)
    fn effective_kind(&self) -> Result<ProviderKind, ProfileError>;
}

impl ProviderProfileExt for ProviderProfile {
    fn normalized_base_url(&self) -> Result<Url, ProfileError> {
        sanitize_base_url(&self.base_url).map_err(ProfileError::InvalidBaseUrl)
    }

    fn sanitized_headers(&self) -> Result<HashMap<String, String>, ProfileError> {
        let headers = self.headers.clone().unwrap_or_default();
        sanitize_headers(&headers).map_err(ProfileError::InvalidHeaders)
    }

    fn effective_kind(&self) -> Result<ProviderKind, ProfileError> {
        if let Some(k) = self.provider_kind {
            return Ok(k);
        }
        let url = self.normalized_base_url()?;
        Ok(ProviderKind::detect(&url))
    }
}

/// Errors that can occur during profile operations
#[derive(Debug, Error)]
pub enum ProfileError {
    /// Invalid base URL configuration
    #[error("invalid base_url: {0}")]
    InvalidBaseUrl(#[source] DiscoveryError),

    /// Invalid headers configuration
    #[error("invalid headers: {0}")]
    InvalidHeaders(#[source] DiscoveryError),

    /// IO operation failed
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization failed
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// Key store operation failed
    #[error("key store error: {0}")]
    KeyStore(#[from] KeyStoreError),
}

/// Trait for profile persistence
pub trait ProfilesStore: Send + Sync {
    /// Get the path to the profiles file
    fn profiles_path(&self) -> PathBuf;

    /// Load all profiles from disk
    fn load_profiles(&self) -> Result<Vec<ProviderProfile>, ProfileError> {
        let p = self.profiles_path();
        if !p.exists() {
            debug!(path = %p.display(), "Profiles file does not exist, returning empty list");
            return Ok(vec![]);
        }
        let data = fs::read_to_string(p)?;
        let profiles: Vec<ProviderProfile> = serde_json::from_str(&data)?;
        info!(count = profiles.len(), "Loaded profiles");
        Ok(profiles)
    }

    /// Save all profiles to disk
    fn save_profiles(&self, profiles: &[ProviderProfile]) -> Result<(), ProfileError> {
        let p = self.profiles_path();
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(profiles)?;
        fs::write(p, data)?;
        info!(count = profiles.len(), "Saved profiles");
        Ok(())
    }

    /// Get a single profile by name
    fn get_profile(&self, name: &str) -> Result<Option<ProviderProfile>, ProfileError> {
        let profiles = self.load_profiles()?;
        Ok(profiles.into_iter().find(|p| p.name == name))
    }

    /// Add or update a profile
    fn upsert_profile(&self, profile: ProviderProfile) -> Result<(), ProfileError> {
        let mut profiles = self.load_profiles()?;

        // Remove existing profile with same name
        profiles.retain(|p| p.name != profile.name);
        profiles.push(profile);

        self.save_profiles(&profiles)
    }

    /// Delete a profile by name
    fn delete_profile(&self, name: &str) -> Result<bool, ProfileError> {
        let mut profiles = self.load_profiles()?;
        let original_len = profiles.len();
        profiles.retain(|p| p.name != name);

        if profiles.len() < original_len {
            self.save_profiles(&profiles)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Default implementation using the standard data directory
#[derive(Debug, Clone, Default)]
pub struct DefaultProfilesStore;

impl ProfilesStore for DefaultProfilesStore {
    fn profiles_path(&self) -> PathBuf {
        let mut base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        base.push("hqe-workbench");
        base.push("profiles.json");
        base
    }
}

/// In-memory store for testing
#[derive(Debug, Clone, Default)]
pub struct MemoryProfilesStore {
    profiles: std::sync::Arc<std::sync::Mutex<Vec<ProviderProfile>>>,
}

impl ProfilesStore for MemoryProfilesStore {
    fn profiles_path(&self) -> PathBuf {
        PathBuf::from(":memory:")
    }

    fn load_profiles(&self) -> Result<Vec<ProviderProfile>, ProfileError> {
        let profiles = self
            .profiles
            .lock()
            .map_err(|_| ProfileError::Io(std::io::Error::other("Mutex poisoned")))?;
        Ok(profiles.clone())
    }

    fn save_profiles(&self, profiles: &[ProviderProfile]) -> Result<(), ProfileError> {
        let mut stored = self
            .profiles
            .lock()
            .map_err(|_| ProfileError::Io(std::io::Error::other("Mutex poisoned")))?;
        *stored = profiles.to_vec();
        Ok(())
    }
}

/// API key storage abstraction
///
/// The docs call for macOS Keychain integration
pub trait ApiKeyStore: Send + Sync {
    /// Get the API key for a profile
    fn get_api_key(&self, profile_name: &str) -> Result<Option<SecretString>, KeyStoreError>;

    /// Store the API key for a profile
    fn set_api_key(&self, profile_name: &str, api_key: &str) -> Result<(), KeyStoreError>;

    /// Delete the API key for a profile
    fn delete_api_key(&self, profile_name: &str) -> Result<(), KeyStoreError>;
}

/// Errors that can occur during key store operations
#[derive(Debug, Error)]
pub enum KeyStoreError {
    /// Underlying keyring/keychain error
    #[error("keyring error: {0}")]
    Keyring(String),

    /// Operation not supported on current platform
    #[error("not supported on this platform")]
    NotSupported,
}

/// macOS Keychain-backed API key storage
#[derive(Debug, Clone)]
pub struct KeychainStore {
    service: String,
}

impl Default for KeychainStore {
    fn default() -> Self {
        Self {
            service: "hqe-workbench".to_string(),
        }
    }
}

impl KeychainStore {
    /// Create a new keychain store with a custom service name
    pub fn with_service(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }

    fn entry(&self, profile_name: &str) -> Result<keyring::Entry, KeyStoreError> {
        let account = format!("api_key:{profile_name}");
        keyring::Entry::new(&self.service, &account)
            .map_err(|e| KeyStoreError::Keyring(e.to_string()))
    }
}

impl ApiKeyStore for KeychainStore {
    #[instrument(skip(self), fields(profile_name))]
    fn get_api_key(&self, profile_name: &str) -> Result<Option<SecretString>, KeyStoreError> {
        let e = self.entry(profile_name)?;
        match e.get_password() {
            Ok(pw) => {
                debug!("Retrieved API key from keychain");
                Ok(Some(SecretString::new(pw.into_boxed_str())))
            }
            Err(keyring::Error::NoEntry) => {
                debug!("No API key found in keychain");
                Ok(None)
            }
            Err(e) => {
                warn!(error = %e, "Failed to retrieve API key from keychain");
                Err(KeyStoreError::Keyring(e.to_string()))
            }
        }
    }

    #[instrument(skip(self, api_key), fields(profile_name))]
    fn set_api_key(&self, profile_name: &str, api_key: &str) -> Result<(), KeyStoreError> {
        let e = self.entry(profile_name)?;
        e.set_password(api_key)
            .map_err(|e| KeyStoreError::Keyring(e.to_string()))?;
        info!("Stored API key in keychain");
        Ok(())
    }

    #[instrument(skip(self), fields(profile_name))]
    fn delete_api_key(&self, profile_name: &str) -> Result<(), KeyStoreError> {
        let e = self.entry(profile_name)?;
        match e.delete_credential() {
            Ok(_) => {
                info!("Deleted API key from keychain");
                Ok(())
            }
            Err(keyring::Error::NoEntry) => {
                debug!("No API key to delete");
                Ok(())
            }
            Err(e) => {
                warn!(error = %e, "Failed to delete API key from keychain");
                Err(KeyStoreError::Keyring(e.to_string()))
            }
        }
    }
}

/// In-memory API key store for testing
#[derive(Debug, Clone, Default)]
pub struct MemoryKeyStore {
    keys: std::sync::Arc<std::sync::Mutex<BTreeMap<String, SecretString>>>,
}

impl ApiKeyStore for MemoryKeyStore {
    fn get_api_key(&self, profile_name: &str) -> Result<Option<SecretString>, KeyStoreError> {
        let keys = self
            .keys
            .lock()
            .map_err(|_| KeyStoreError::Keyring("Mutex poisoned".to_string()))?;
        Ok(keys.get(profile_name).cloned())
    }

    fn set_api_key(&self, profile_name: &str, api_key: &str) -> Result<(), KeyStoreError> {
        let mut keys = self
            .keys
            .lock()
            .map_err(|_| KeyStoreError::Keyring("Mutex poisoned".to_string()))?;
        keys.insert(profile_name.to_string(), SecretString::new(api_key.into()));
        Ok(())
    }

    fn delete_api_key(&self, profile_name: &str) -> Result<(), KeyStoreError> {
        let mut keys = self
            .keys
            .lock()
            .map_err(|_| KeyStoreError::Keyring("Mutex poisoned".to_string()))?;
        keys.remove(profile_name);
        Ok(())
    }
}

/// Complete profile manager combining storage and key management
#[derive(Debug)]
pub struct ProfileManager<S: ProfilesStore, K: ApiKeyStore> {
    store: S,
    key_store: K,
}

impl<S: ProfilesStore, K: ApiKeyStore> ProfileManager<S, K> {
    /// Create a new profile manager
    pub fn new(store: S, key_store: K) -> Self {
        Self { store, key_store }
    }

    /// Load all profiles (without API keys)
    pub fn load_profiles(&self) -> Result<Vec<ProviderProfile>, ProfileError> {
        self.store.load_profiles()
    }

    /// Get a profile with its API key
    #[instrument(skip(self), fields(profile_name))]
    pub fn get_profile_with_key(
        &self,
        name: &str,
    ) -> Result<Option<(ProviderProfile, Option<SecretString>)>, ProfileError> {
        let profile = self.store.get_profile(name)?;
        match profile {
            Some(p) => {
                let key = self
                    .key_store
                    .get_api_key(name)
                    .map_err(ProfileError::KeyStore)?;
                Ok(Some((p, key)))
            }
            None => Ok(None),
        }
    }

    /// Save a profile and optionally its API key
    #[instrument(skip(self, api_key), fields(profile_name = %profile.name))]
    pub fn save_profile(
        &self,
        profile: ProviderProfile,
        api_key: Option<&str>,
    ) -> Result<(), ProfileError> {
        let profile_name = profile.name.clone();

        // Save profile first
        self.store.upsert_profile(profile)?;

        // Save API key if provided
        if let Some(key) = api_key {
            self.key_store
                .set_api_key(&profile_name, key)
                .map_err(ProfileError::KeyStore)?;
        }

        Ok(())
    }

    /// Delete a profile and its API key
    #[instrument(skip(self), fields(profile_name))]
    pub fn delete_profile(&self, name: &str) -> Result<bool, ProfileError> {
        let deleted = self.store.delete_profile(name)?;
        if deleted {
            // Also delete the API key
            if let Err(e) = self.key_store.delete_api_key(name) {
                warn!(error = %e, "Failed to delete API key, but profile was deleted");
            }
        }
        Ok(deleted)
    }
}

impl Default for ProfileManager<DefaultProfilesStore, KeychainStore> {
    fn default() -> Self {
        Self::new(DefaultProfilesStore, KeychainStore::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;

    #[test]
    fn provider_profile_builder() {
        let profile = ProviderProfile::new("test", "https://api.openai.com")
            .with_model("gpt-4o")
            .with_header("X-Custom", "value")
            .with_timeout(30);

        assert_eq!(profile.name, "test");
        assert_eq!(profile.base_url, "https://api.openai.com");
        assert_eq!(profile.default_model, "gpt-4o");
        assert_eq!(
            profile.headers.as_ref().and_then(|h| h.get("X-Custom")),
            Some(&"value".to_string())
        );
        assert_eq!(profile.timeout_s, 30);
    }

    #[test]
    fn memory_profiles_store() -> anyhow::Result<()> {
        let store = MemoryProfilesStore::default();

        let profile = ProviderProfile::new("test", "https://api.example.com");
        store.upsert_profile(profile.clone())?;

        let loaded = store.get_profile("test")?;
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().name, "test");

        let deleted = store.delete_profile("test")?;
        assert!(deleted);

        let not_found = store.get_profile("test")?;
        assert!(not_found.is_none());
        Ok(())
    }

    #[test]
    fn memory_key_store() -> anyhow::Result<()> {
        let store = MemoryKeyStore::default();

        store.set_api_key("test", "secret123")?;

        let key = store.get_api_key("test")?;
        assert!(key.is_some());
        assert_eq!(key.unwrap().expose_secret(), "secret123");

        store.delete_api_key("test")?;

        let gone = store.get_api_key("test")?;
        assert!(gone.is_none());
        Ok(())
    }

    #[test]
    fn profile_manager_save_and_load() -> anyhow::Result<()> {
        let store = MemoryProfilesStore::default();
        let key_store = MemoryKeyStore::default();
        let manager = ProfileManager::new(store, key_store);

        let profile = ProviderProfile::new("test", "https://api.example.com");
        manager.save_profile(profile, Some("secret123"))?;

        let (loaded_profile, key) = manager.get_profile_with_key("test")?.unwrap();
        assert_eq!(loaded_profile.name, "test");
        assert_eq!(key.unwrap().expose_secret(), "secret123");
        Ok(())
    }

    #[test]
    fn default_profiles_store_path() {
        let store = DefaultProfilesStore;
        let path = store.profiles_path();
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("hqe-workbench"));
        assert!(path_str.contains("profiles.json"));
    }
}
