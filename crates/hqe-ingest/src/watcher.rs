use anyhow::Result;
use notify::{Event, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};

use crate::loader::TopicLoader;
use hqe_protocol::models::TopicManifest;

/// Events emitted by the ingestion engine
pub enum IngestEvent {
    /// A new topic was loaded or updated
    TopicLoaded(TopicManifest),
    /// A topic was removed
    TopicRemoved(String),
    /// An error occurred during ingestion
    Error(String),
}

/// The main ingestion engine that watches for file changes
pub struct IngestEngine {
    root_path: PathBuf,
    event_tx: mpsc::Sender<IngestEvent>,
    /// Tracks loaded topics: manifest_path -> topic_id
    topic_map: Arc<RwLock<HashMap<PathBuf, String>>>,
}

impl IngestEngine {
    /// Create a new ingestion engine
    pub fn new(root_path: PathBuf, event_tx: mpsc::Sender<IngestEvent>) -> Self {
        Self {
            root_path,
            event_tx,
            topic_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Starts the file watcher and processes events.
    /// This function runs indefinitely until the channel is closed.
    pub async fn start(&self) -> Result<()> {
        let (tx, mut rx) = mpsc::channel(100);
        let tx_clone = tx.clone();

        // Bridge notify (sync) to tokio channel
        let mut watcher =
            notify::recommended_watcher(move |res: Result<Event, notify::Error>| match res {
                Ok(event) => {
                    let _ = tx_clone.blocking_send(event);
                }
                Err(e) => {
                    error!("Watch error: {:?}", e);
                }
            })?;

        watcher.watch(&self.root_path, RecursiveMode::Recursive)?;
        info!("Ingestion Engine watching: {:?}", self.root_path);

        // Process file system events
        while let Some(event) = rx.recv().await {
            match event.kind {
                notify::EventKind::Create(_) | notify::EventKind::Modify(_) => {
                    for path in event.paths {
                        // We only care about manifest files for now
                        if is_manifest_file(&path) {
                            self.process_manifest_change(&path).await;
                        }
                    }
                }
                notify::EventKind::Remove(_) => {
                    for path in event.paths {
                        if is_manifest_file(&path) {
                            self.process_manifest_removal(&path).await;
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Scans the root directory immediately for existing topics.
    pub async fn initial_scan(&self) -> Result<()> {
        let mut entries = tokio::fs::read_dir(&self.root_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                self.process_manifest_change(&path).await;
            }
        }
        Ok(())
    }

    async fn process_manifest_change(&self, path: &Path) {
        info!("Processing potential topic at: {:?}", path);
        match TopicLoader::load_from_path(path).await {
            Ok(manifest) => {
                let topic_id = manifest.id.clone();
                let manifest_path = if path.is_dir() {
                    // Try to find the actual manifest file path
                    let yaml_path = path.join("manifest.yaml");
                    let json_path = path.join("manifest.json");
                    if yaml_path.exists() {
                        yaml_path
                    } else {
                        json_path
                    }
                } else {
                    path.to_path_buf()
                };

                // Track the topic_id for this manifest path
                {
                    let mut map = self.topic_map.write().await;
                    map.insert(manifest_path, topic_id.clone());
                }

                info!("Loaded topic: {} ({})", manifest.name, topic_id);
                if let Err(e) = self.event_tx.send(IngestEvent::TopicLoaded(manifest)).await {
                    error!("Failed to send topic loaded event: {:?}", e);
                }
            }
            Err(e) => {
                error!("Failed to load topic from {:?}: {:?}", path, e);
                let _ = self.event_tx.send(IngestEvent::Error(e.to_string())).await;
            }
        }
    }

    async fn process_manifest_removal(&self, path: &Path) {
        info!("Processing manifest removal at: {:?}", path);

        // Look up the topic_id for this manifest path
        let topic_id = {
            let mut map = self.topic_map.write().await;
            map.remove(path)
        };

        if let Some(topic_id) = topic_id {
            info!("Topic removed: {}", topic_id);
            if let Err(e) = self
                .event_tx
                .send(IngestEvent::TopicRemoved(topic_id))
                .await
            {
                error!("Failed to send topic removed event: {:?}", e);
            }
        } else {
            warn!("Removed manifest not tracked: {:?}", path);
        }
    }
}

fn is_manifest_file(path: &Path) -> bool {
    if let Some(name) = path.file_name() {
        return name == "manifest.yaml" || name == "manifest.json";
    }
    false
}
