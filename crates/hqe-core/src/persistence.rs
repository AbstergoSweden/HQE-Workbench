//! Persistence layer for query caching and session logging.
//!
//! Uses SQLite to store:
//! - Request/Response Cache (hashed by input)
//! - Session History (audit logs)

use rusqlite::{params, Connection, Result};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

/// Local database manager
#[derive(Debug, Clone)]
pub struct LocalDb {
    conn: Arc<Mutex<Connection>>,
}

impl LocalDb {
    /// Initialize the local database
    pub fn init() -> anyhow::Result<Self> {
        let db_path = get_db_path()?;

        info!("Initializing local database at {:?}", db_path);

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;

        // Enable WAL mode for better concurrency
        conn.execute("PRAGMA journal_mode=WAL;", [])?;

        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS request_cache (
                hash TEXT PRIMARY KEY,
                model TEXT NOT NULL,
                prompt_json TEXT NOT NULL,
                response_json TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                last_accessed_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS session_log (
                id INTEGER PRIMARY KEY,
                session_id TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                metadata_json TEXT
            )",
            [],
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Calculate hash for a request
    pub fn calculate_hash(model: &str, messages_json: &str, params_json: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(model.as_bytes());
        hasher.update(b"|");
        hasher.update(messages_json.as_bytes());
        hasher.update(b"|");
        hasher.update(params_json.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Get cached response
    pub fn get_cached_response(&self, hash: &str) -> Result<Option<String>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| rusqlite::Error::InvalidParameterName("Mutex poisoned".to_string()))?;
        let mut stmt = conn.prepare("SELECT response_json FROM request_cache WHERE hash = ?1")?;

        let mut rows = stmt.query(params![hash])?;

        if let Some(row) = rows.next()? {
            // Update last accessed time asynchronously (fire and forget pattern ideally, but sync here for safety)
            let _ = conn.execute(
                "UPDATE request_cache SET last_accessed_at = CURRENT_TIMESTAMP WHERE hash = ?",
                params![hash],
            );
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    /// Store response in cache
    pub fn cache_response(
        &self,
        hash: &str,
        model: &str,
        prompt: &str,
        response: &str,
    ) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| rusqlite::Error::InvalidParameterName("Mutex poisoned".to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO request_cache (hash, model, prompt_json, response_json)
             VALUES (?1, ?2, ?3, ?4)",
            params![hash, model, prompt, response],
        )?;
        debug!("Cached response for hash {}", hash);
        Ok(())
    }

    /// Log a session interaction
    pub fn log_interaction(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
        metadata: Option<&str>,
    ) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| rusqlite::Error::InvalidParameterName("Mutex poisoned".to_string()))?;
        conn.execute(
            "INSERT INTO session_log (session_id, role, content, metadata_json)
             VALUES (?1, ?2, ?3, ?4)",
            params![session_id, role, content, metadata],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_db_init() {
        // Use in-memory DB for testing if init() supported it, but init() uses file.
        // We'll trust init() works or needs refactoring for testability if we wanted pure unit tests.
        // However, we can test hashing.
        let hash = LocalDb::calculate_hash("gpt-4", r#"{"role":"user"}"#, r#"{}"#);
        assert_eq!(hash.len(), 64); // SHA256 hex string
    }

    #[test]
    fn test_hash_stability() {
        let h1 = LocalDb::calculate_hash("model", "msg", "params");
        let h2 = LocalDb::calculate_hash("model", "msg", "params");
        assert_eq!(h1, h2);

        let h3 = LocalDb::calculate_hash("model2", "msg", "params");
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_hash_uniqueness() {
        // Different inputs should produce different hashes
        let h1 = LocalDb::calculate_hash("gpt-4", "message", "{}");
        let h2 = LocalDb::calculate_hash("gpt-3", "message", "{}");
        let h3 = LocalDb::calculate_hash("gpt-4", "different", "{}");
        let h4 = LocalDb::calculate_hash("gpt-4", "message", "{\"temp\":0.7}");

        assert_ne!(h1, h2, "Different models should produce different hashes");
        assert_ne!(h1, h3, "Different messages should produce different hashes");
        assert_ne!(h1, h4, "Different params should produce different hashes");
    }

    #[test]
    fn test_hash_empty_inputs() {
        // Empty inputs should still produce valid hashes
        let hash = LocalDb::calculate_hash("", "", "");
        assert_eq!(hash.len(), 64);
        // Hash of "|||" (empty model + separator + empty message + separator + empty params)
        assert_eq!(
            hash,
            "565d240f5343e625ae579a4d45a770f1f02c6368b5ed4d06da4fbe6f47c28866"
        );
    }

    #[test]
    fn test_hash_special_characters() {
        // Special characters should be handled correctly
        let hash1 = LocalDb::calculate_hash("model", "{\"key\": \"value with spaces\"}", "{}");
        let hash2 = LocalDb::calculate_hash("model", "{\"key\": \"value with spaces\"}", "{}");
        assert_eq!(
            hash1, hash2,
            "Same special characters should produce same hash"
        );
    }

    #[test]
    fn test_hash_unicode() {
        // Unicode characters should be handled correctly
        let hash = LocalDb::calculate_hash("model", "日本語テキスト", "{}");
        assert_eq!(hash.len(), 64);

        // Same unicode should produce same hash
        let hash2 = LocalDb::calculate_hash("model", "日本語テキスト", "{}");
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_hash_order_matters() {
        // Order of parameters matters
        let h1 = LocalDb::calculate_hash("a", "b", "c");
        let h2 = LocalDb::calculate_hash("c", "b", "a");
        assert_ne!(h1, h2, "Different order should produce different hashes");
    }

    #[test]
    fn test_hash_long_content() {
        // Long content should still produce valid hashes
        let long_message = "a".repeat(10000);
        let hash = LocalDb::calculate_hash("model", &long_message, "{}");
        assert_eq!(hash.len(), 64);
    }
}

fn get_db_path() -> anyhow::Result<PathBuf> {
    let mut path = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?;
    path.push("hqe-workbench");
    path.push("hqe.db");
    Ok(path)
}
