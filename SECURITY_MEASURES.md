# Security Measures Documentation

## Overview
This document describes the security measures implemented in the HQE Workbench to protect against various attack vectors including prompt injection, path traversal, command injection, and other security vulnerabilities.

## Implemented Security Controls

### 1. Prompt Injection Prevention

#### Input Sanitization
- **Location**: `crates/hqe-openai/src/prompts.rs`
- **Description**: All user inputs and file content are sanitized before being included in prompts sent to LLMs
- **Mechanism**: The `sanitize_for_prompt()` function escapes special characters and removes/obfuscates typical prompt injection patterns

```rust
pub fn sanitize_for_prompt(content: &str) -> String {
    // Remove or escape prompt injection patterns
    let mut safe = content
        .replace("{{", "\\{\\{") // Escape template delimiters
        .replace("{%", "\\{%") // Escape template delimiters
        .replace("{#", "\\{#") // Escape template delimiters
        .replace("}}", "\\}\\}") // Escape template delimiters
        .replace("%}", "%\\}") // Escape template delimiters
        .replace("#}", "#\\}") // Escape template delimiters
        .replace("[INST]", "\\[INST\\]") // Escape instruction markers
        .replace("[/INST]", "\\[/INST\\]") // Escape instruction markers
        .replace("<|", "\\<|") // Escape special tokens
        .replace("|>", "|\\>") // Escape special tokens
        // ... additional sanitization patterns
}
```

#### System Prompt Protection
- **Location**: `crates/hqe-openai/src/prompts.rs`
- **Description**: Added security notice to system prompts to prevent manipulation
- **Mechanism**: Explicit instruction in system prompt to ignore attempts to modify behavior

### 2. Path Traversal Prevention

#### Canonical Path Validation
- **Location**: `crates/hqe-core/src/repo.rs` and `crates/hqe-mcp/src/loader.rs`
- **Description**: Validates that file paths are within allowed directories
- **Mechanism**: Uses `canonicalize()` to resolve symbolic links and `..` components, then verifies the resolved path is within the allowed root directory

```rust
// In repo.rs
let canonical_full_path = full_path.canonicalize().map_err(crate::HqeError::Io)?;
let canonical_root = self.root_path.canonicalize().map_err(crate::HqeError::Io)?;

if !canonical_full_path.starts_with(&canonical_root) {
    return Err(LoaderError::PathTraversal(path.to_path_buf()));
}
```

#### Prompt Loader Path Validation
- **Location**: `crates/hqe-mcp/src/loader.rs`
- **Description**: Validates file paths during prompt template loading
- **Mechanism**: Similar canonical path validation to prevent access to unauthorized files

### 3. Template Injection Prevention

#### Safe Template Substitution
- **Location**: `cli/hqe/src/main.rs`
- **Description**: Sanitizes template values during substitution
- **Mechanism**: Applies `sanitize_for_prompt()` to all template values before substitution

```rust
fn substitute_template(template: &str, args: &serde_json::Value) -> String {
    let mut result = template.to_string();

    if let Some(obj) = args.as_object() {
        for (k, v) in obj {
            let key = format!("{{{{{}}}}}", k); // {{key}}
            let val = v
                .as_str()
                .map(sanitize_for_prompt) // Validate string values
                .unwrap_or_else(|| sanitize_for_prompt(&v.to_string())); // Validate non-string values
            result = result.replace(&key, &val);
        }
    }

    result
}
```

### 4. Resource Exhaustion Prevention

#### Scan Limits Validation
- **Location**: `crates/hqe-core/src/models.rs`
- **Description**: Validates scan parameters to prevent resource exhaustion
- **Mechanism**: `ScanLimits::validate()` method enforces reasonable bounds

```rust
impl ScanLimits {
    pub fn validate(&self) -> Result<(), crate::HqeError> {
        if self.max_files_sent == 0 || self.max_files_sent > 1000 {
            return Err(crate::HqeError::Scan(format!(
                "max_files_sent must be between 1 and 1000, got {}",
                self.max_files_sent
            )));
        }

        if self.max_total_chars_sent == 0 || self.max_total_chars_sent > 50_000_000 {
            return Err(crate::HqeError::Scan(format!(
                "max_total_chars_sent must be between 1 and 50,000,000, got {}",
                self.max_total_chars_sent
            )));
        }

        if self.snippet_chars == 0 || self.snippet_chars > 1_000_000 {
            return Err(crate::HqeError::Scan(format!(
                "snippet_chars must be between 1 and 1,000,000, got {}",
                self.snippet_chars
            )));
        }

        Ok(())
    }
}
```

### 5. Deserialization Attack Prevention

#### TOML/YAML Content Validation
- **Location**: `crates/hqe-mcp/src/loader.rs`
- **Description**: Validates content before deserialization to prevent unsafe type indicators
- **Mechanism**: Checks for dangerous patterns in TOML/YAML content

```rust
fn validate_toml_content(content: &str) -> Result<(), LoaderError> {
    if content.contains("!!") || content.contains("!<!") || content.contains("!") {
        return Err(LoaderError::ParseToml {
            path: std::path::PathBuf::from("validation"),
            source: toml::de::Error::custom("TOML contains potentially unsafe type indicators"),
        });
    }

    Ok(())
}

fn validate_yaml_content(content: &str) -> Result<(), LoaderError> {
    if content.contains("!!") || content.contains("!") || content.contains("&") || content.contains("*") {
        return Err(LoaderError::ParseYaml {
            path: std::path::PathBuf::from("validation"),
            source: serde_yaml::Error::custom(
                "YAML contains potentially unsafe type indicators or anchors",
            ),
        });
    }

    Ok(())
}
```

### 6. ReDoS Prevention in Redaction Engine

#### Optimized Regex Patterns
- **Location**: `crates/hqe-core/src/redaction.rs`
- **Description**: Improved regex patterns to prevent Regular Expression Denial of Service
- **Mechanism**: Added word boundaries and length limits to prevent catastrophic backtracking

```rust
// Before: r"[0-9a-zA-Z/+]{40}" (vulnerable to ReDoS on long lines)
// After: r"\b[0-9a-zA-Z/+]{40}\b" (with word boundaries)

// Added length limits to prevent overly greedy matches
if let Ok(re) = Regex::new(r#"(?i)(secret|api[_-]?key|token)\s*=\s*["']?[a-zA-Z0-9_-]{16,64}["']?"#) {
    patterns.push((SecretType::GenericSecret, re));
}
```

### 7. SQL Injection False Positive Reduction

#### Context-Aware Detection
- **Location**: `crates/hqe-core/src/repo.rs`
- **Description**: Improved SQL injection detection to reduce false positives
- **Mechanism**: Only flags code that combines SQL keywords with dynamic string construction

```rust
// Check for SQL keywords that are actually SQL (not substrings)
let sql_keywords = [
    "select ", "insert ", "update ", "delete ", "drop ", "from ", "where ",
];
let has_sql_keyword = sql_keywords.iter().any(|kw| line_lower.contains(kw));

// Check for string interpolation patterns that could inject user input
let has_formatting = line_lower.contains("format!(")
    || line_lower.contains("format(")
    || (line.contains("$") && line.contains("{"));

// Only flag if we have SQL keywords AND dynamic string construction
if has_sql_keyword && (has_formatting || has_concat) {
    // Additional check: exclude common false positives
    let is_false_positive = line_lower.contains("selected_")
        && !line_lower.contains("select ")
        || line_lower.contains("updated_") && !line_lower.contains("update ")
        // ... additional false positive checks
}
```

## Security Testing Procedures

### Automated Testing
- Unit tests cover all security validation functions
- Property-based tests for input sanitization
- Fuzzing tests for parser validation functions
- Integration tests for end-to-end security workflows

### Manual Security Testing
- Penetration testing of all user input paths
- Path traversal testing with various encoding techniques
- Prompt injection testing with known bypass techniques
- Resource exhaustion testing with large inputs

## Maintenance Guidelines

### Adding New Input Sources
When adding new input sources that eventually reach LLMs or file operations:
1. Apply input sanitization using existing functions
2. Validate file paths against allowed directories
3. Add appropriate rate limiting if applicable
4. Update security tests to cover new paths

### Review Checklist
- [ ] All user inputs are sanitized before use
- [ ] File paths are validated against allowed directories
- [ ] Resource limits are enforced
- [ ] Deserialization is protected against attacks
- [ ] Regex patterns are ReDoS-resistant
- [ ] New code paths are covered by security tests

## Incident Response
If a security vulnerability is discovered:
1. Report privately via security@hqeworkbench.com
2. Include reproduction steps and impact assessment
3. Do not open public GitHub issues for vulnerabilities
4. Coordinate with maintainers on disclosure timeline