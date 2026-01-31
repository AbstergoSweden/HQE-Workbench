use hqe_ingest::{IngestEngine, IngestEvent};
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_ingestion_engine_detects_new_topic() {
    // 1. Setup temp directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root_path = temp_dir.path().to_path_buf();

    // 2. Create channel and engine
    let (tx, mut rx) = mpsc::channel(10);
    let engine = IngestEngine::new(root_path.clone(), tx);

    // 3. Create a topic directory and manifest
    let topic_dir = root_path.join("finance-test");
    tokio::fs::create_dir(&topic_dir)
        .await
        .expect("Failed to create topic dir");

    let manifest_content = r#"
id: "finance-test"
name: "Financial Analysis Suite"
version: "1.0.0"
capabilities:
  tools: []
  prompts: []
  flows: []
data_schemas: {}
"#;

    tokio::fs::write(topic_dir.join("manifest.yaml"), manifest_content)
        .await
        .expect("Failed to write manifest");

    // 4. Run a deterministic scan to load new topics
    engine
        .initial_scan()
        .await
        .expect("Initial scan failed");

    // 5. Wait for event
    let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("Timed out waiting for event");

    match event {
        Some(IngestEvent::TopicLoaded(manifest)) => {
            assert_eq!(manifest.id, "finance-test");
            assert_eq!(manifest.name, "Financial Analysis Suite");
        }
        Some(IngestEvent::Error(e)) => panic!("Received error event: {}", e),
        _ => panic!("Received unexpected event or channel closed"),
    }
}
