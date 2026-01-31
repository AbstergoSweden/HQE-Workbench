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
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT response_json FROM request_cache WHERE hash = ?1"
        )?;
        
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
    pub fn cache_response(&self, hash: &str, model: &str, prompt: &str, response: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO request_cache (hash, model, prompt_json, response_json)
             VALUES (?1, ?2, ?3, ?4)",
            params![hash, model, prompt, response],
        )?;
        debug!("Cached response for hash {}", hash);
        Ok(())
    }

    /// Log a session interaction
    pub fn log_interaction(&self, session_id: &str, role: &str, content: &str, metadata: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
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
}

fn get_db_path() -> anyhow::Result<PathBuf> {
    let mut path = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?;
    path.push("hqe-workbench");
    path.push("hqe.db");
    Ok(path)
}
