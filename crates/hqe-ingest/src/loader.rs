use anyhow::{Context, Result};
use hqe_protocol::models::TopicManifest;
use std::path::Path;
use tokio::fs;

/// Helper for loading topic manifests
pub struct TopicLoader;

impl TopicLoader {
    /// Loads and parses a TopicManifest from a given path (directory or file).
    /// If a directory is provided, it looks for `manifest.yaml` or `manifest.json`.
    pub async fn load_from_path(path: &Path) -> Result<TopicManifest> {
        let manifest_path = if path.is_dir() {
            let yaml = path.join("manifest.yaml");
            if yaml.exists() {
                yaml
            } else {
                path.join("manifest.json")
            }
        } else {
            path.to_path_buf()
        };

        if !manifest_path.exists() {
            return Err(anyhow::anyhow!(
                "Manifest file not found at {:?}",
                manifest_path
            ));
        }

        let content = fs::read_to_string(&manifest_path)
            .await
            .with_context(|| format!("Failed to read manifest file at {:?}", manifest_path))?;

        let manifest: TopicManifest = if manifest_path.extension().is_some_and(|ext| ext == "json")
        {
            serde_json::from_str(&content)?
        } else {
            serde_yaml::from_str(&content)?
        };

        Ok(manifest)
    }
}
