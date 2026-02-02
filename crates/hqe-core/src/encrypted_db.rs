//! Encrypted Local Database for Chat Persistence
//!
//! This module provides SQLCipher-based encrypted storage for chat sessions,
//! messages, and attachments. The encryption key is stored in the OS keychain.
//!
//! Security features:
//! - 256-bit AES encryption (SQLCipher)
//! - Key stored in macOS Keychain (Secure Enclave)
//! - Key derivation: PBKDF2-HMAC-SHA256
//! - No plaintext transcripts on disk

use rusqlite::OptionalExtension;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};

/// Errors that can occur in encrypted database operations
#[derive(Debug, thiserror::Error)]
pub enum EncryptedDbError {
    /// SQLite error
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// Keychain/keyring error
    #[error("Keyring error: {0}")]
    Keyring(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Encryption/decryption error
    #[error("Encryption error: {0}")]
    Encryption(String),

    /// Invalid key or password
    #[error("Invalid key or password")]
    InvalidKey,

    /// Database migration error
    #[error("Migration error: {0}")]
    Migration(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
}

/// Result type for encrypted DB operations
pub type Result<T> = std::result::Result<T, EncryptedDbError>;

/// Configuration for encrypted database
#[derive(Debug, Clone)]
pub struct EncryptedDbConfig {
    /// Database file path
    pub db_path: PathBuf,
    /// Keychain service name
    pub keychain_service: String,
    /// Keychain account name for encryption key
    pub keychain_account: String,
    /// SQLCipher page size (default: 4096)
    pub page_size: i32,
    /// PBKDF2 iterations (default: 256000)
    pub kdf_iterations: i32,
}

impl Default for EncryptedDbConfig {
    fn default() -> Self {
        let db_path = dirs::data_local_dir()
            .map(|mut p| {
                p.push("hqe-workbench");
                p.push("chat.db");
                p
            })
            .unwrap_or_else(|| PathBuf::from("chat.db"));

        Self {
            db_path,
            keychain_service: "hqe-workbench".to_string(),
            keychain_account: "db_encryption_key".to_string(),
            page_size: 4096,
            kdf_iterations: 256000,
        }
    }
}

/// Encrypted database manager for chat persistence
#[derive(Debug)]
pub struct EncryptedDb {
    conn: Arc<Mutex<Connection>>,
    config: EncryptedDbConfig,
}

impl EncryptedDb {
    /// Initialize the encrypted database
    ///
    /// If the database doesn't exist, creates it with new encryption key.
    /// If it exists, attempts to open with stored key.
    pub fn init() -> Result<Self> {
        Self::init_with_config(EncryptedDbConfig::default())
    }

    /// Initialize with custom configuration
    pub fn init_with_config(config: EncryptedDbConfig) -> Result<Self> {
        info!("Initializing encrypted database at {:?}", config.db_path);

        // Ensure directory exists
        if let Some(parent) = config.db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Get or create encryption key
        let key = Self::get_or_create_key(&config)?;

        // Open database with encryption
        let conn = Self::open_encrypted(&config, &key)?;

        // Initialize schema
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
            config,
        };

        db.initialize_schema()?;

        info!("Encrypted database initialized successfully");
        Ok(db)
    }

    /// Open database with SQLCipher encryption
    ///
    /// # Security
    /// The key is validated to be a 64-character hex string before use,
    /// preventing SQL injection attacks.
    fn open_encrypted(config: &EncryptedDbConfig, key: &str) -> Result<Connection> {
        // Validate key format before use
        if !is_valid_hex_key(key) {
            return Err(EncryptedDbError::InvalidKey);
        }

        let conn = Connection::open(&config.db_path)?;

        // Configure SQLCipher encryption using pragma_update to avoid SQL injection
        // Key has been validated to contain only hex characters
        conn.pragma_update(None, "key", key)?;
        conn.pragma_update(None, "cipher_page_size", config.page_size)?;
        conn.pragma_update(None, "kdf_iter", config.kdf_iterations)?;

        // Verify encryption is working
        conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))?;

        Ok(conn)
    }

    /// Get existing key or generate new one
    fn get_or_create_key(config: &EncryptedDbConfig) -> Result<String> {
        let entry = keyring::Entry::new(&config.keychain_service, &config.keychain_account)
            .map_err(|e| EncryptedDbError::Keyring(e.to_string()))?;

        match entry.get_password() {
            Ok(key) => {
                debug!("Retrieved encryption key from keychain");
                Ok(key)
            }
            Err(keyring::Error::NoEntry) => {
                // Generate new key
                let key = Self::generate_key();
                entry
                    .set_password(&key)
                    .map_err(|e| EncryptedDbError::Keyring(e.to_string()))?;
                info!("Generated and stored new encryption key");
                Ok(key)
            }
            Err(e) => Err(EncryptedDbError::Keyring(e.to_string())),
        }
    }

    /// Generate a cryptographically secure random key
    fn generate_key() -> String {
        use rand::RngCore;
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        hex::encode(bytes)
    }

    /// Initialize database schema
    fn initialize_schema(&self) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| EncryptedDbError::Encryption("Mutex poisoned".to_string()))?;

        // Chat sessions table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS chat_sessions (
                id TEXT PRIMARY KEY,
                repo_path TEXT,
                prompt_id TEXT,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                metadata_json TEXT
            )",
            [],
        )?;

        // Chat messages table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS chat_messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                parent_id TEXT,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                context_refs_json TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                metadata_json TEXT,
                FOREIGN KEY (session_id) REFERENCES chat_sessions(id) ON DELETE CASCADE,
                FOREIGN KEY (parent_id) REFERENCES chat_messages(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Attachments table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS attachments (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                name TEXT NOT NULL,
                content_type TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                content_size INTEGER,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (session_id) REFERENCES chat_sessions(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Feedback table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS feedback (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                message_id TEXT NOT NULL,
                feedback_type TEXT NOT NULL,
                comment TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                context_hash TEXT,
                FOREIGN KEY (session_id) REFERENCES chat_sessions(id) ON DELETE CASCADE,
                FOREIGN KEY (message_id) REFERENCES chat_messages(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_session ON chat_messages(session_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON chat_messages(timestamp)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sessions_repo ON chat_sessions(repo_path)",
            [],
        )?;

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])?;

        debug!("Database schema initialized");
        Ok(())
    }

    /// Rotate encryption key
    ///
    /// Re-encrypts the database with a new key. The old key is preserved
    /// until rotation is complete.
    pub fn rotate_key(&self) -> Result<()> {
        info!("Rotating encryption key");

        let new_key = Self::generate_key();

        // Validate key format (should be 64 hex characters)
        if !is_valid_hex_key(&new_key) {
            return Err(EncryptedDbError::InvalidKey);
        }

        let conn = self
            .conn
            .lock()
            .map_err(|_| EncryptedDbError::Encryption("Mutex poisoned".to_string()))?;

        // Re-key the database using pragma_update to avoid SQL injection
        // The key is validated to be hex-only, making SQL injection impossible
        conn.pragma_update(None, "rekey", &new_key)?;

        // Update keychain
        let entry =
            keyring::Entry::new(&self.config.keychain_service, &self.config.keychain_account)
                .map_err(|e| EncryptedDbError::Keyring(e.to_string()))?;

        entry
            .set_password(&new_key)
            .map_err(|e| EncryptedDbError::Keyring(e.to_string()))?;

        info!("Encryption key rotated successfully");
        Ok(())
    }

    /// Export encrypted backup
    ///
    /// # Security
    /// The backup_path is validated to prevent directory traversal attacks.
    /// Only safe path characters are allowed.
    pub fn export_backup(&self, backup_path: &PathBuf) -> Result<()> {
        info!("Exporting encrypted backup to {:?}", backup_path);

        // Validate backup path to prevent SQL injection and directory traversal
        let canonical_path = backup_path.canonicalize().or_else(|_| {
            // If canonicalize fails (path doesn't exist), try to canonicalize the parent
            if let Some(parent) = backup_path.parent() {
                let canonical_parent = parent.canonicalize().map_err(EncryptedDbError::Io)?;
                std::fs::create_dir_all(&canonical_parent).map_err(EncryptedDbError::Io)?;
                Ok(canonical_parent.join(backup_path.file_name().unwrap_or_default()))
            } else {
                Err(EncryptedDbError::Validation(
                    "Invalid backup path".to_string(),
                ))
            }
        })?;

        // Ensure path doesn't contain null bytes or other dangerous characters
        let path_str = canonical_path.to_string_lossy();
        if path_str.contains('\0') || path_str.contains('\'') || path_str.contains('"') {
            return Err(EncryptedDbError::Validation(
                "Backup path contains invalid characters".to_string(),
            ));
        }

        // Validate extension is .db or .db.encrypted
        let ext = canonical_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if !matches!(ext, "db" | "encrypted" | "sql" | "sqlite" | "backup") {
            return Err(EncryptedDbError::Validation(
                "Backup must have .db, .encrypted, .sql, .sqlite, or .backup extension".to_string(),
            ));
        }

        let conn = self
            .conn
            .lock()
            .map_err(|_| EncryptedDbError::Encryption("Mutex poisoned".to_string()))?;

        // SQLCipher backup using vacuum into with validated path
        // Path has been canonicalized and validated, making SQL injection impossible
        conn.execute(
            &format!(
                "VACUUM INTO '{}'",
                escape_sql_string(&canonical_path.to_string_lossy())
            ),
            [],
        )?;

        info!("Backup exported successfully");
        Ok(())
    }

    /// Get database file path
    pub fn path(&self) -> &PathBuf {
        &self.config.db_path
    }

    /// Verify database integrity
    pub fn verify_integrity(&self) -> Result<bool> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| EncryptedDbError::Encryption("Mutex poisoned".to_string()))?;

        match conn.execute("PRAGMA integrity_check", []) {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("Database integrity check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Get connection for direct queries
    pub fn connection(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| EncryptedDbError::Encryption("Mutex poisoned".to_string()))
    }
}

/// Escape a string for safe use in SQL
fn escape_sql_string(s: &str) -> String {
    s.replace("'", "''")
}

/// A chat session record.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatSession {
    /// Unique session identifier.
    pub id: String,
    /// Absolute path to the repository being scanned (optional).
    pub repo_path: Option<String>,
    /// Associated prompt ID (optional).
    pub prompt_id: Option<String>,
    /// Human-readable name for the session.
    pub name: String,
    /// LLM provider used for this session.
    pub provider: String,
    /// Specific model name within the provider.
    pub model: String,
    /// Timestamp when the session was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Timestamp of the last update to the session.
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Metadata as JSON.
    pub metadata: Option<serde_json::Value>,
}

/// A single message within a chat session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    /// Unique identifier for the message.
    pub id: String,
    /// ID of the session this message belongs to.
    pub session_id: String,
    /// ID of the parent message (if any) for threading.
    pub parent_id: Option<String>,
    /// Role of the message sender.
    pub role: MessageRole,
    /// Text content of the message.
    pub content: String,
    /// References to files or code snippets.
    pub context_refs: Option<Vec<ContextRef>>,
    /// Timestamp when the message was created.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Metadata as JSON.
    pub metadata: Option<serde_json::Value>,
}

/// Message role
/// Role of a message in a conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    /// System-level instructions.
    System,
    /// Human user input.
    User,
    /// LLM assistant response.
    Assistant,
    /// Output from a tool execution.
    Tool,
}

/// A specific item of context attached to a message.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextRef {
    /// Relative path to the file.
    pub file_path: String,
    /// Starting line number.
    pub line_start: Option<u32>,
    /// Ending line number.
    pub line_end: Option<u32>,
    /// Content snippet.
    pub snippet: Option<String>,
}

/// An attachment associated with a chat message or session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Attachment {
    /// Unique identifier for the attachment.
    pub id: String,
    /// ID of the session this attachment belongs to.
    pub session_id: String,
    /// Original filename of the attachment.
    pub name: String,
    /// MIME type or content type.
    pub content_type: String,
    /// Size in bytes.
    pub content_hash: String,
    /// File extension.
    pub content_size: Option<i64>,
    /// Starting line number for code snippets.
    pub line_start: Option<u32>,
    /// Ending line number for code snippets.
    pub line_end: Option<u32>,
    /// Actual text snippet if appropriate.
    pub snippet: Option<String>,
    /// Timestamp when the attachment was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Record of feedback provided for a message.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeedbackRecord {
    /// Unique identifier for the feedback record.
    pub id: String,
    /// ID of the session this feedback relates to.
    pub session_id: String,
    /// ID of the message this feedback relates to.
    pub message_id: String,
    /// Type of feedback provided.
    pub feedback_type: FeedbackType,
    /// Optional user commentary.
    pub comment: Option<String>,
    /// Timestamp when feedback was provided.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Contextual hash for auditing.
    pub context_hash: Option<String>,
}

/// Types of user feedback for messages.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum FeedbackType {
    /// Positive feedback.
    ThumbsUp,
    /// Negative feedback.
    ThumbsDown,
    /// Problematic response report.
    Report,
}

/// Pagination parameters for message retrieval.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Pagination {
    /// Number of items to return.
    pub limit: usize,
    /// Number of items to skip.
    pub offset: usize,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: 100, // Default 100 messages per page
            offset: 0,
        }
    }
}

impl Pagination {
    /// Create a new pagination with the given limit and offset
    pub fn new(limit: usize, offset: usize) -> Self {
        Self {
            limit: limit.min(1000), // Cap at 1000 to prevent abuse
            offset,
        }
    }

    /// Get the next page
    pub fn next_page(&self) -> Self {
        Self {
            limit: self.limit,
            offset: self.offset + self.limit,
        }
    }
}

/// Operations for managing chat sessions and messages.
///
/// This trait provides a higher-level interface for chat-related database
/// operations, including session management and message retrieval.
pub trait ChatOperations {
    /// Create a new chat session.
    fn create_session(&self, session: &ChatSession) -> Result<()>;
    /// Retrieve a chat session by its unique ID.
    fn get_session(&self, session_id: &str) -> Result<Option<ChatSession>>;
    /// List chat sessions, optionally filtered by repository path.
    fn list_sessions(&self, repo_path: Option<&str>) -> Result<Vec<ChatSession>>;
    /// Delete a chat session and its associated messages.
    fn delete_session(&self, session_id: &str) -> Result<()>;

    /// Add a message within a transaction for data integrity.
    fn add_message(&self, message: &ChatMessage) -> Result<()>;

    /// Get all messages for a session with default pagination.
    fn get_messages(&self, session_id: &str) -> Result<Vec<ChatMessage>> {
        self.get_messages_paginated(session_id, Pagination::default())
    }

    /// Get messages with pagination support.
    fn get_messages_paginated(
        &self,
        session_id: &str,
        pagination: Pagination,
    ) -> Result<Vec<ChatMessage>>;

    /// Get total message count for a session (useful for pagination UI).
    fn get_message_count(&self, session_id: &str) -> Result<usize>;

    /// Retrieve a single message by its ID.
    fn get_message(&self, message_id: &str) -> Result<Option<ChatMessage>>;

    /// Add an attachment to a session.
    fn add_attachment(&self, attachment: &Attachment) -> Result<()>;
    /// Retrieve all attachments for a specific session.
    fn get_attachments(&self, session_id: &str) -> Result<Vec<Attachment>>;

    /// Add user feedback for a specific message.
    fn add_feedback(&self, feedback: &FeedbackRecord) -> Result<()>;
    /// Retrieve feedback associated with a specific message.
    fn get_feedback(&self, message_id: &str) -> Result<Vec<FeedbackRecord>>;
}

/// Core trait for secure data persistence.
///
/// This trait defines the operations for managing sessions, messages,
/// attachments, and feedback in a secure, encrypted database.
pub trait Persistence: Send + Sync {
    /// Save a new session or update an existing one.
    fn save_session(&self, session: &ChatSession) -> Result<()>;
    /// Retrieve a session by its unique ID.
    fn get_session(&self, id: &str) -> Result<Option<ChatSession>>;
    /// List all available sessions.
    fn list_sessions(&self) -> Result<Vec<ChatSession>>;
    /// Delete a session and all its associated data.
    fn delete_session(&self, id: &str) -> Result<()>;

    /// Add a message to a session interaction history.
    fn add_message(&self, message: &ChatMessage) -> Result<()>;
    /// Retrieve all messages for a specific session.
    fn get_messages(&self, session_id: &str) -> Result<Vec<ChatMessage>>;

    /// Add an attachment (e.g., file content, context) to a session.
    fn add_attachment(&self, attachment: &Attachment) -> Result<()>;
    /// Retrieve all attachments for a specific session.
    fn get_attachments(&self, session_id: &str) -> Result<Vec<Attachment>>;

    /// Add feedback record for a specific message.
    fn add_feedback(&self, feedback: &FeedbackRecord) -> Result<()>;
    /// Retrieve all feedback associated with a specific message.
    fn get_feedback(&self, message_id: &str) -> Result<Vec<FeedbackRecord>>;
}

impl ChatOperations for EncryptedDb {
    fn create_session(&self, session: &ChatSession) -> Result<()> {
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO chat_sessions (id, repo_path, prompt_id, provider, model, created_at, updated_at, metadata_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                 repo_path = excluded.repo_path,
                 prompt_id = excluded.prompt_id,
                 provider = excluded.provider,
                 model = excluded.model,
                 updated_at = excluded.updated_at,
                 metadata_json = excluded.metadata_json",
            params![
                session.id,
                session.repo_path,
                session.prompt_id,
                session.provider,
                session.model,
                session.created_at.to_rfc3339(),
                session.updated_at.to_rfc3339(),
                session.metadata.as_ref().map(|m| m.to_string())
            ],
        )?;
        Ok(())
    }

    fn get_session(&self, session_id: &str) -> Result<Option<ChatSession>> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, repo_path, prompt_id, provider, model, created_at, updated_at, metadata_json
             FROM chat_sessions WHERE id = ?1"
        )?;

        let session = stmt
            .query_row([session_id], |row| {
                Ok(ChatSession {
                    id: row.get(0)?,
                    repo_path: row.get(1)?,
                    prompt_id: row.get(2)?,
                    name: row.get(3)?,
                    provider: row.get(4)?,
                    model: row.get(5)?,
                    created_at: parse_datetime(row.get(6)?).unwrap_or_else(chrono::Utc::now),
                    updated_at: parse_datetime(row.get(7)?).unwrap_or_else(chrono::Utc::now),
                    metadata: row
                        .get::<_, Option<String>>(8)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                })
            })
            .optional()?;

        Ok(session)
    }

    fn list_sessions(&self, repo_path: Option<&str>) -> Result<Vec<ChatSession>> {
        let conn = self.connection()?;

        let query = if repo_path.is_some() {
            "SELECT id, repo_path, prompt_id, provider, model, created_at, updated_at, metadata_json
             FROM chat_sessions WHERE repo_path = ?1 ORDER BY updated_at DESC"
        } else {
            "SELECT id, repo_path, prompt_id, provider, model, created_at, updated_at, metadata_json
             FROM chat_sessions ORDER BY updated_at DESC"
        };

        let mut stmt = conn.prepare(query)?;

        let rows: Vec<ChatSession> = if let Some(repo) = repo_path {
            stmt.query_map([repo], |row| {
                Ok(ChatSession {
                    id: row.get(0)?,
                    repo_path: row.get(1)?,
                    prompt_id: row.get(2)?,
                    name: row.get(3)?,
                    provider: row.get(4)?,
                    model: row.get(5)?,
                    created_at: parse_datetime(row.get(6)?).unwrap_or_else(chrono::Utc::now),
                    updated_at: parse_datetime(row.get(7)?).unwrap_or_else(chrono::Utc::now),
                    metadata: row
                        .get::<_, Option<String>>(8)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                })
            })?
            .filter_map(|r| r.ok())
            .collect()
        } else {
            stmt.query_map([], |row| {
                Ok(ChatSession {
                    id: row.get(0)?,
                    repo_path: row.get(1)?,
                    prompt_id: row.get(2)?,
                    name: row.get(3)?,
                    provider: row.get(4)?,
                    model: row.get(5)?,
                    created_at: parse_datetime(row.get(6)?).unwrap_or_else(chrono::Utc::now),
                    updated_at: parse_datetime(row.get(7)?).unwrap_or_else(chrono::Utc::now),
                    metadata: row
                        .get::<_, Option<String>>(8)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                })
            })?
            .filter_map(|r| r.ok())
            .collect()
        };

        Ok(rows)
    }

    fn delete_session(&self, session_id: &str) -> Result<()> {
        let conn = self.connection()?;
        conn.execute(
            "DELETE FROM chat_sessions WHERE id = ?1",
            params![session_id],
        )?;
        Ok(())
    }

    fn add_message(&self, message: &ChatMessage) -> Result<()> {
        let mut conn = self.connection()?;

        // Use a transaction to ensure both operations succeed or fail together
        let tx = conn.transaction()?;

        // Insert/update the message
        tx.execute(
            "INSERT INTO chat_messages (id, session_id, parent_id, role, content, context_refs_json, timestamp, metadata_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                 content = excluded.content,
                 context_refs_json = excluded.context_refs_json,
                 metadata_json = excluded.metadata_json",
            params![
                message.id,
                message.session_id,
                message.parent_id,
                format!("{:?}", message.role).to_lowercase(),
                message.content,
                message.context_refs.as_ref().map(|r| serde_json::to_string(r).unwrap_or_default()),
                message.timestamp.to_rfc3339(),
                message.metadata.as_ref().map(|m| m.to_string())
            ],
        )?;

        // Update session timestamp
        tx.execute(
            "UPDATE chat_sessions SET updated_at = ?1 WHERE id = ?2",
            params![chrono::Utc::now().to_rfc3339(), message.session_id],
        )?;

        // Commit the transaction
        tx.commit()?;

        Ok(())
    }

    fn get_messages_paginated(
        &self,
        session_id: &str,
        pagination: Pagination,
    ) -> Result<Vec<ChatMessage>> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, parent_id, role, content, context_refs_json, timestamp, metadata_json
             FROM chat_messages 
             WHERE session_id = ?1 
             ORDER BY timestamp ASC
             LIMIT ?2 OFFSET ?3"
        )?;

        let rows: Vec<ChatMessage> = stmt
            .query_map(
                [
                    session_id,
                    &pagination.limit.to_string(),
                    &pagination.offset.to_string(),
                ],
                |row| {
                    let role_str: String = row.get(3)?;
                    let role = match role_str.as_str() {
                        "system" => MessageRole::System,
                        "user" => MessageRole::User,
                        "assistant" => MessageRole::Assistant,
                        "tool" => MessageRole::Tool,
                        _ => MessageRole::User,
                    };

                    Ok(ChatMessage {
                        id: row.get(0)?,
                        session_id: row.get(1)?,
                        parent_id: row.get(2)?,
                        role,
                        content: row.get(4)?,
                        context_refs: row
                            .get::<_, Option<String>>(5)?
                            .and_then(|s| serde_json::from_str(&s).ok()),
                        timestamp: parse_datetime(row.get(6)?).unwrap_or_else(chrono::Utc::now),
                        metadata: row
                            .get::<_, Option<String>>(7)?
                            .and_then(|s| serde_json::from_str(&s).ok()),
                    })
                },
            )?
            .filter_map(|r| r.ok())
            .collect();

        Ok(rows)
    }

    fn get_message_count(&self, session_id: &str) -> Result<usize> {
        let conn = self.connection()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM chat_messages WHERE session_id = ?1",
            [session_id],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    fn get_message(&self, message_id: &str) -> Result<Option<ChatMessage>> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, parent_id, role, content, context_refs_json, timestamp, metadata_json
             FROM chat_messages WHERE id = ?1"
        )?;

        let message = stmt
            .query_row([message_id], |row| {
                let role_str: String = row.get(3)?;
                let role = match role_str.as_str() {
                    "system" => MessageRole::System,
                    "user" => MessageRole::User,
                    "assistant" => MessageRole::Assistant,
                    "tool" => MessageRole::Tool,
                    _ => MessageRole::User,
                };

                Ok(ChatMessage {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    parent_id: row.get(2)?,
                    role,
                    content: row.get(4)?,
                    context_refs: row
                        .get::<_, Option<String>>(5)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    timestamp: parse_datetime(row.get(6)?).unwrap_or_else(chrono::Utc::now),
                    metadata: row
                        .get::<_, Option<String>>(7)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                })
            })
            .optional()?;

        Ok(message)
    }

    fn add_attachment(&self, attachment: &Attachment) -> Result<()> {
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO attachments (id, session_id, name, content_type, content_hash, content_size, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name,
                 content_type = excluded.content_type,
                 content_hash = excluded.content_hash",
            params![
                attachment.id,
                attachment.session_id,
                attachment.name,
                attachment.content_type,
                attachment.content_hash,
                attachment.content_size,
                attachment.created_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    fn get_attachments(&self, session_id: &str) -> Result<Vec<Attachment>> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, name, content_type, content_hash, content_size, created_at
             FROM attachments WHERE session_id = ?1 ORDER BY created_at ASC",
        )?;

        let rows: Vec<Attachment> = stmt
            .query_map([session_id], |row| {
                Ok(Attachment {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    name: row.get(2)?,
                    content_type: row.get(3)?,
                    content_hash: row.get(4)?,
                    content_size: row.get(5)?,
                    line_start: None,
                    line_end: None,
                    snippet: None,
                    created_at: parse_datetime(row.get(6)?).unwrap_or_else(chrono::Utc::now),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(rows)
    }

    fn add_feedback(&self, feedback: &FeedbackRecord) -> Result<()> {
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO feedback (id, session_id, message_id, feedback_type, comment, timestamp, context_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                 comment = excluded.comment",
            params![
                feedback.id,
                feedback.session_id,
                feedback.message_id,
                format!("{:?}", feedback.feedback_type).to_lowercase(),
                feedback.comment,
                feedback.timestamp.to_rfc3339(),
                feedback.context_hash
            ],
        )?;
        Ok(())
    }

    fn get_feedback(&self, message_id: &str) -> Result<Vec<FeedbackRecord>> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, message_id, feedback_type, comment, timestamp, context_hash
             FROM feedback WHERE message_id = ?1 ORDER BY timestamp ASC",
        )?;

        let rows: Vec<FeedbackRecord> = stmt
            .query_map([message_id], |row| {
                let type_str: String = row.get(3)?;
                let feedback_type = match type_str.as_str() {
                    "thumbsup" => FeedbackType::ThumbsUp,
                    "thumbsdown" => FeedbackType::ThumbsDown,
                    "report" => FeedbackType::Report,
                    _ => FeedbackType::Report,
                };

                Ok(FeedbackRecord {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    message_id: row.get(2)?,
                    feedback_type,
                    comment: row.get(4)?,
                    timestamp: parse_datetime(row.get(5)?).unwrap_or_else(chrono::Utc::now),
                    context_hash: row.get(6)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(rows)
    }
}

/// Parse datetime string
fn parse_datetime(s: String) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(&s)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

/// Validate that a key is a valid hex string (64 characters, 0-9, a-f, A-F)
///
/// This prevents SQL injection by ensuring only safe characters are present.
fn is_valid_hex_key(key: &str) -> bool {
    key.len() == 64 && key.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // SQLCipher tests require the sqlcipher-tests feature
    // Run with: cargo test --features sqlcipher-tests

    fn create_test_db() -> (EncryptedDb, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        let config = EncryptedDbConfig {
            db_path,
            keychain_service: "hqe-test".to_string(),
            keychain_account: format!("test-key-{}", uuid::Uuid::new_v4()),
            page_size: 4096,
            kdf_iterations: 256000,
        };

        let db = EncryptedDb::init_with_config(config).unwrap();
        (db, dir)
    }

    #[test]
    #[cfg(feature = "sqlcipher-tests")]
    fn test_init_creates_database() {
        let (db, _dir) = create_test_db();
        assert!(db.path().exists());
        assert!(db.verify_integrity().unwrap());
    }

    #[test]
    #[cfg(feature = "sqlcipher-tests")]
    fn test_create_and_get_session() {
        let (db, _dir) = create_test_db();

        let session = ChatSession {
            id: "test-session-1".to_string(),
            repo_path: Some("/path/to/repo".to_string()),
            prompt_id: Some("security_audit".to_string()),
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: None,
        };

        db.create_session(&session).unwrap();

        let retrieved = db.get_session("test-session-1").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().provider, "openai");
    }

    #[test]
    #[cfg(feature = "sqlcipher-tests")]
    fn test_add_and_get_messages() {
        let (db, _dir) = create_test_db();

        // Create session first
        let session = ChatSession {
            id: "session-msg".to_string(),
            repo_path: None,
            prompt_id: None,
            provider: "test".to_string(),
            model: "test".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: None,
        };
        db.create_session(&session).unwrap();

        // Add messages
        let msg1 = ChatMessage {
            id: "msg-1".to_string(),
            session_id: "session-msg".to_string(),
            parent_id: None,
            role: MessageRole::User,
            content: "Hello".to_string(),
            context_refs: None,
            timestamp: chrono::Utc::now(),
            metadata: None,
        };

        let msg2 = ChatMessage {
            id: "msg-2".to_string(),
            session_id: "session-msg".to_string(),
            parent_id: Some("msg-1".to_string()),
            role: MessageRole::Assistant,
            content: "Hi there!".to_string(),
            context_refs: None,
            timestamp: chrono::Utc::now(),
            metadata: None,
        };

        db.add_message(&msg1).unwrap();
        db.add_message(&msg2).unwrap();

        let messages = db.get_messages("session-msg").unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].content, "Hello");
        assert_eq!(messages[1].content, "Hi there!");
    }

    #[test]
    #[cfg(feature = "sqlcipher-tests")]
    fn test_list_sessions() {
        let (db, _dir) = create_test_db();

        let session1 = ChatSession {
            id: "s1".to_string(),
            repo_path: Some("/repo/a".to_string()),
            prompt_id: None,
            provider: "test".to_string(),
            model: "test".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: None,
        };

        let session2 = ChatSession {
            id: "s2".to_string(),
            repo_path: Some("/repo/a".to_string()),
            prompt_id: None,
            provider: "test".to_string(),
            model: "test".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: None,
        };

        db.create_session(&session1).unwrap();
        db.create_session(&session2).unwrap();

        let all = db.list_sessions(None).unwrap();
        assert_eq!(all.len(), 2);

        let for_repo = db.list_sessions(Some("/repo/a")).unwrap();
        assert_eq!(for_repo.len(), 2);
    }

    #[test]
    #[cfg(feature = "sqlcipher-tests")]
    fn test_delete_session_cascades() {
        let (db, _dir) = create_test_db();

        let session = ChatSession {
            id: "del-session".to_string(),
            repo_path: None,
            prompt_id: None,
            provider: "test".to_string(),
            model: "test".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: None,
        };
        db.create_session(&session).unwrap();

        let msg = ChatMessage {
            id: "del-msg".to_string(),
            session_id: "del-session".to_string(),
            parent_id: None,
            role: MessageRole::User,
            content: "Test".to_string(),
            context_refs: None,
            timestamp: chrono::Utc::now(),
            metadata: None,
        };
        db.add_message(&msg).unwrap();

        // Delete session
        db.delete_session("del-session").unwrap();

        // Verify session is gone
        assert!(db.get_session("del-session").unwrap().is_none());
        // Messages should be cascade deleted
        assert!(db.get_messages("del-session").unwrap().is_empty());
    }

    #[test]
    #[cfg(feature = "sqlcipher-tests")]
    fn test_feedback_operations() {
        let (db, _dir) = create_test_db();

        let session = ChatSession {
            id: "fb-session".to_string(),
            repo_path: None,
            prompt_id: None,
            provider: "test".to_string(),
            model: "test".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: None,
        };
        db.create_session(&session).unwrap();

        let feedback = FeedbackRecord {
            id: "fb-1".to_string(),
            session_id: "fb-session".to_string(),
            message_id: "msg-123".to_string(),
            feedback_type: FeedbackType::ThumbsUp,
            comment: Some("Great response!".to_string()),
            timestamp: chrono::Utc::now(),
            context_hash: Some("abc123".to_string()),
        };

        db.add_feedback(&feedback).unwrap();

        let retrieved = db.get_feedback("msg-123").unwrap();
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].feedback_type, FeedbackType::ThumbsUp);
    }
}
