use std::{collections::BTreeMap, time::Duration};

use serde::{Deserialize, Serialize};

use hqe_openai::provider_discovery::{DiskCache, ProviderDiscoveryClient, ProviderModelList};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderProfileInput {
    pub base_url: String,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    pub api_key: String,
    #[serde(default = "default_timeout_s")]
    pub timeout_s: u64,
    #[serde(default)]
    pub no_cache: bool,
}

fn default_timeout_s() -> u64 { 60 }

#[tauri::command]
pub async fn discover_models(input: ProviderProfileInput) -> Result<ProviderModelList, String> {
    let cache = if input.no_cache { None } else { Some(DiskCache::default()) };
    let client = ProviderDiscoveryClient::new(
        &input.base_url,
        &input.headers,
        Some(input.api_key),
        Duration::from_secs(input.timeout_s),
        cache,
    ).map_err(|e| e.to_string())?;

    client.discover_chat_models().await.map_err(|e| e.to_string())
}
