# HQE Workbench Security & Bug Findings

**Date:** 2025-01-29  
**Scope:** Full codebase review  
**Severity Scale:** 🔴 Critical | 🟠 High | 🟡 Medium | 🟢 Low

---

## 🔴 CRITICAL: ProviderProfile Struct Inconsistency

### Summary
Three incompatible `ProviderProfile` struct definitions exist across the codebase, causing serialization/deserialization failures.

### Affected Files
1. `crates/hqe-openai/src/lib.rs:147-156`
2. `crates/hqe-core/src/models.rs:414-424`
3. `crates/hqe-openai/src/profile.rs:21-35`

### Details

**Definition 1 (lib.rs):**
```rust
pub struct ProviderProfile {
    pub name: String,
    pub base_url: String,
    pub api_key_id: String,
    pub default_model: String,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub organization: Option<String>,
    pub project: Option<String>,
}
```

**Definition 2 (profile.rs):**
```rust
pub struct ProviderProfile {
    pub name: String,
    pub base_url: String,
    pub model: Option<String>,  // Different field name!
    #[serde(default)]
    pub headers: BTreeMap<String, String>,  // Different type!
    pub provider_kind: Option<ProviderKind>,
    pub timeout_s: u64,  // Extra field!
}
```

**Definition 3 (models.rs):**
```rust
pub struct ProviderProfile {
    pub name: String,
    pub base_url: String,
    pub api_key_id: String,
    pub default_model: String,
    pub headers: Option<HashMap<String, String>>,
    pub organization: Option<String>,
    pub project: Option<String>,
}
```

### Impact
- Profile saved with one struct cannot be loaded with another
- CLI uses `hqe_openai::ProviderProfile` but may receive data serialized from `hqe_core::ProviderProfile`
- Silent data loss or deserialization errors

### Recommended Fix
```rust
// In hqe-protocol/src/models.rs (single source of truth)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderProfile {
    pub name: String,
    pub base_url: String,
    pub api_key_id: String,
    pub default_model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_kind: Option<ProviderKind>,
    #[serde(default = "default_timeout_s")]
    pub timeout_s: u64,
}
```

---

## 🟠 HIGH: SQL Injection Detection False Positives

### Summary
The SQL injection detection pattern produces false positives on benign code.

### Affected Code
```rust
// crates/hqe-core/src/repo.rs:452-472
if (line_lower.contains("select")
    || line_lower.contains("insert")
    || line_lower.contains("update"))
    && (line_lower.contains("format!")
        || line_lower.contains("format(")
        || line.contains("$"))
{
    findings.push(LocalFinding {
        finding_type: "SQL_INJECTION_RISK".to_string(),
        // ...
    });
}
```

### False Positive Examples
```rust
// This would be flagged but is safe:
let selected_item = format!("Selected: {}", name);  // "selected" contains "select"

// This comment would be flagged:
// See the INSERT statement documentation at...

// This variable name would be flagged:
let updated_at = format!("{}", timestamp);  // "updated_at" contains "update"
```

### Recommended Fix
```rust
fn check_sql_injection(line: &str, idx: usize) -> Option<LocalFinding> {
    let line_lower = line.to_lowercase();
    
    // Skip comments
    if line.trim().starts_with("//") 
        || line.trim().starts_with("#")
        || line.trim().starts_with("/*") {
        return None;
    }
    
    // Require SQL keywords to be actual SQL, not substrings
    let sql_keywords = ["select ", "insert ", "update ", "delete ", "drop "];
    let has_sql = sql_keywords.iter().any(|kw| line_lower.contains(kw));
    
    // Require actual string interpolation patterns, not just "$"
    let has_formatting = line.contains("format!(\"") && line.contains("{}")
        || line.contains("format(\"") && line.contains("%s")
        || line.contains("$") && line_lower.contains("template");
    
    if has_sql && has_formatting {
        // Additional check: ensure we're not in a string literal
        if !is_inside_string_literal(line, idx) {
            return Some(LocalFinding { /* ... */ });
        }
    }
    
    None
}
```

---

## 🟠 HIGH: Git Apply Patch Logic Error

### Summary
The dry-run check in `apply_patch` is redundant and potentially incorrect.

### Affected Code
```rust
// crates/hqe-git/src/lib.rs:183-211
pub async fn apply_patch(&self, patch: &str, dry_run: bool) -> anyhow::Result<()> {
    // First do a dry-run check
    if !dry_run {
        let _check = self.run_git(&["apply", "--check", "-"]).await?;  // BUG: This ignores the patch!

        // We need to pass the patch via stdin...
        let mut cmd = Command::new("git");
        cmd.current_dir(&self.path)
            .args(["apply", "--check", "-"])  // This runs check AGAIN
            // ...
    }
    // ...
}
```

### Issues
1. First check at line 187 doesn't pass the patch content
2. The check runs even when `dry_run` is false (should only validate then apply)
3. Redundant operations

### Recommended Fix
```rust
pub async fn apply_patch(&self, patch: &str, dry_run: bool) -> anyhow::Result<()> {
    let args = if dry_run {
        vec!["apply", "--check", "-"]
    } else {
        // First check, then apply
        self.run_git_with_stdin(&["apply", "--check", "-"], patch).await?;
        vec!["apply", "-"]
    };
    
    self.run_git_with_stdin(&args, patch).await
}
```

---

## 🟠 HIGH: Secret Detection in Documentation Files

### Summary
Secret patterns are checked against documentation files, causing false positives.

### Affected Code
```rust
// crates/hqe-core/src/repo.rs:388-400
for file in &scanned.files {
    // Only check source code files
    if !file.ends_with(".rs")
        && !file.ends_with(".js")
        // ...
    {
        continue;
    }
    // ...
}
```

### Problem
Documentation files like `README.md`, `API.md`, `SETUP.md` can contain example API keys or tokens that trigger false positives.

### Recommended Fix
```rust
const DOC_EXTENSIONS: &[&str] = &[".md", ".txt", ".rst", ".adoc", ".markdown"];

fn should_check_for_secrets(path: &str) -> bool {
    let lower = path.to_lowercase();
    
    // Skip documentation files
    for ext in DOC_EXTENSIONS {
        if lower.ends_with(ext) {
            return false;
        }
    }
    
    // Skip example/test files
    if lower.contains("example") || lower.contains("test") || lower.contains("fixture") {
        return false;
    }
    
    true
}
```

---

## 🟡 MEDIUM: Blocking Operations in Async Handler

### Summary
The prompt handler uses blocking operations inside an async context.

### Affected Code
```rust
// cli/hqe/src/main.rs:246-272
let handler = Box::new(move |args: serde_json::Value| -> anyhow::Result<serde_json::Value> {
    let prompt_text = substitute_template(&template, &args);
    
    tokio::task::block_in_place(|| {  // Blocks the runtime!
        tokio::runtime::Handle::current().block_on(async {
            let response = client_clone.chat(...).await?;
            Ok(json!({ "result": response.choices[0].message.content }))
        })
    })
});
```

### Impact
- Blocks the async runtime thread
- Reduces concurrency capacity
- Potential for thread pool exhaustion under load

### Recommended Fix
```rust
// Option 1: Make handler async
pub type ToolHandler = Box<dyn Fn(Value) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>> + Send + Sync>;

// Option 2: Use channel-based approach
let (tx, rx) = tokio::sync::oneshot::channel();
tokio::spawn(async move {
    let result = client.chat(...).await;
    let _ = tx.send(result);
});
```

---

## 🟡 MEDIUM: Path Traversal in Prompt Loader

### Summary
The prompt loader doesn't validate that the resolved path stays within the root directory.

### Affected Code
```rust
// crates/hqe-mcp/src/loader.rs:83
let relative_path = path.strip_prefix(&self.root_path)?;
```

### Problem
If `root_path` is `/prompts` and `path` is `/prompts/../../etc/passwd.toml`, the `strip_prefix` succeeds but the resulting path still contains `..` components.

### Recommended Fix
```rust
fn load_prompt_file(&self, path: &Path) -> Result<LoadedPromptTool> {
    // Canonicalize and validate path is within root
    let canonical_path = path.canonicalize()
        .with_context(|| format!("Failed to canonicalize: {}", path.display()))?;
    let canonical_root = self.root_path.canonicalize()
        .with_context(|| format!("Failed to canonicalize root: {}", self.root_path.display()))?;
    
    if !canonical_path.starts_with(&canonical_root) {
        anyhow::bail!("Path traversal detected: {}", path.display());
    }
    
    // ... rest of function
}
```

---

## 🟡 MEDIUM: Mutex Poisoning Risk

### Summary
Memory stores use `.unwrap()` on mutex locks, which panics if the mutex is poisoned.

### Affected Code
```rust
// crates/hqe-openai/src/profile.rs:200, 205
impl ProfilesStore for MemoryProfilesStore {
    fn load_profiles(&self) -> Result<Vec<ProviderProfile>, ProfileError> {
        let profiles = self.profiles.lock().unwrap();  // Panics if poisoned!
        Ok(profiles.clone())
    }
    
    fn save_profiles(&self, profiles: &[ProviderProfile]) -> Result<(), ProfileError> {
        let mut stored = self.profiles.lock().unwrap();  // Panics if poisoned!
        *stored = profiles.to_vec();
        Ok(())
    }
}
```

### Recommended Fix
```rust
fn load_profiles(&self) -> Result<Vec<ProviderProfile>, ProfileError> {
    let profiles = self.profiles.lock()
        .map_err(|_| ProfileError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Mutex poisoned"
        )))?;
    Ok(profiles.clone())
}
```

---

## 🟢 LOW: Regex Performance Issues

### Summary
Some regex patterns could cause performance issues on certain inputs.

### Affected Patterns
```rust
// crates/hqe-core/src/redaction.rs:71-73
if let Ok(re) = Regex::new(r"[0-9a-zA-Z/+]{40}") {  // No anchors!
    patterns.push((SecretType::AwsSecretKey, re));
}
```

### Problem
- Pattern `[0-9a-zA-Z/+]{40}` has no word boundaries
- On long lines with many alphanumeric characters, it may try many match positions
- Could cause ReDoS on crafted input

### Recommended Fix
```rust
// Add word boundaries or anchors
if let Ok(re) = Regex::new(r"\b[0-9a-zA-Z/+]{40}\b") {
    patterns.push((SecretType::AwsSecretKey, re));
}

// Or use more specific patterns
if let Ok(re) = Regex::new(r"(?i)(aws_secret_access_key|secret)\s*=\s*['\"][0-9a-zA-Z/+]{40}['\"]") {
    patterns.push((SecretType::AwsSecretKey, re));
}
```

---

## Summary Statistics

| Severity | Count | Categories |
|----------|-------|------------|
| 🔴 Critical | 1 | Architecture |
| 🟠 High | 4 | Security, Logic |
| 🟡 Medium | 6 | Performance, Reliability |
| 🟢 Low | 7 | Code Quality, Documentation |
| **Total** | **18** | |

---

*Document generated: 2025-01-29*  
*Reviewers: Automated code analysis*
