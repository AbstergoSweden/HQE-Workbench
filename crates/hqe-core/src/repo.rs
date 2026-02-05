//! Repository ingestion and analysis

use crate::models::{DetectedTechnology, Entrypoint, LocalFinding, Severity, TechStack};
use crate::redaction::should_exclude_file;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};
use walkdir::WalkDir;

/// Mask secret values in a line, keeping only the key name.
/// Example: "API_KEY=sk-abc123" -> "API_KEY=***REDACTED***"
fn mask_secret_line(line: &str) -> String {
    if let Some((k, _)) = line.split_once('=') {
        format!("{}=***REDACTED***", k.trim())
    } else {
        "***REDACTED***".to_string()
    }
}

/// Repository scanner
#[derive(Debug, Clone)]
pub struct RepoScanner {
    /// Root path of the repository to scan
    pub root_path: PathBuf,
    /// Maximum file size to process (in bytes)
    pub max_file_size: usize,
    /// Maximum directory depth to traverse
    pub max_depth: usize,
}

impl RepoScanner {
    /// Creates a new RepoScanner for the given root path
    pub fn new(root_path: impl AsRef<Path>) -> Self {
        Self {
            root_path: root_path.as_ref().to_path_buf(),
            max_file_size: 1_000_000, // 1MB default
            max_depth: 10,            // Default max depth
        }
    }

    /// Set the maximum depth for directory traversal
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Set the maximum file size for reading
    pub fn with_max_file_size(mut self, size: usize) -> Self {
        self.max_file_size = size;
        self
    }

    /// Scan repository and build directory tree summary
    pub fn scan(&self) -> crate::Result<ScannedRepo> {
        let mut files = Vec::new();
        let mut directories = Vec::new();
        let mut total_size: u64 = 0;

        for entry in WalkDir::new(&self.root_path)
            .follow_links(false)
            .max_depth(self.max_depth)
        {
            let entry = entry.map_err(|e| crate::HqeError::Scan(e.to_string()))?;
            let path = entry.path();
            let relative_path = path
                .strip_prefix(&self.root_path)
                .map_err(|e| crate::HqeError::Scan(format!("Failed to strip prefix: {}", e)))?;
            let path_str = relative_path.to_string_lossy().to_string();

            if path.is_dir() {
                if !should_exclude_dir(&path_str) {
                    directories.push(path_str);
                }
                continue;
            }

            if should_exclude_file(&path_str) {
                debug!("Excluding file: {}", path_str);
                continue;
            }

            if let Ok(metadata) = entry.metadata() {
                let size = metadata.len();
                if size > self.max_file_size as u64 {
                    warn!("Skipping large file ({} bytes): {}", size, path_str);
                    continue;
                }
                total_size += size;
            }

            files.push(path_str);
        }

        Ok(ScannedRepo {
            root_path: self.root_path.clone(),
            files,
            directories,
            total_size,
        })
    }

    /// Detect entrypoints in the repository
    pub fn detect_entrypoints(&self) -> crate::Result<Vec<Entrypoint>> {
        let mut entrypoints = Vec::new();

        // Common entrypoint patterns
        let patterns = vec![
            (
                "main",
                vec![
                    "main.rs", "main.go", "main.py", "main.js", "main.ts", "index.js", "index.ts",
                    "app.py", "app.go", "lib.rs", "mod.rs",
                ],
            ),
            (
                "config",
                vec![
                    "package.json",
                    "Cargo.toml",
                    "pyproject.toml",
                    "setup.py",
                    "go.mod",
                    "requirements.txt",
                    "Pipfile",
                    "poetry.lock",
                    "Gemfile",
                    "composer.json",
                ],
            ),
            (
                "docker",
                vec![
                    "Dockerfile",
                    "docker-compose.yml",
                    "docker-compose.yaml",
                    ".dockerignore",
                ],
            ),
            (
                "ci",
                vec![
                    ".github/workflows/ci.yml",
                    ".github/workflows/build.yml",
                    ".github/workflows/test.yml",
                    ".gitlab-ci.yml",
                    "Jenkinsfile",
                ],
            ),
            (
                "docs",
                vec![
                    "README.md",
                    "README.rst",
                    "CONTRIBUTING.md",
                    "CHANGELOG.md",
                    "LICENSE",
                ],
            ),
        ];

        for (entry_type, filenames) in patterns {
            for filename in filenames {
                let full_path = self.root_path.join(filename);
                if full_path.exists() {
                    entrypoints.push(Entrypoint {
                        file_path: filename.to_string(),
                        entry_type: entry_type.to_string(),
                        description: format!("Detected {} entrypoint", entry_type),
                    });
                }
            }
        }

        Ok(entrypoints)
    }

    /// Detect tech stack from package manifests
    pub fn detect_tech_stack(&self) -> crate::Result<TechStack> {
        let mut detected = Vec::new();
        let mut package_managers = Vec::new();

        // Node.js / JavaScript
        if self.root_path.join("package.json").exists() {
            package_managers.push("npm/pnpm/yarn".to_string());

            // Try to read dependencies
            if let Ok(content) = std::fs::read_to_string(self.root_path.join("package.json")) {
                if content.contains("react") {
                    detected.push(DetectedTechnology {
                        name: "React".to_string(),
                        version: None,
                        evidence: "package.json".to_string(),
                    });
                }
                if content.contains("vue") {
                    detected.push(DetectedTechnology {
                        name: "Vue.js".to_string(),
                        version: None,
                        evidence: "package.json".to_string(),
                    });
                }
                if content.contains("express") {
                    detected.push(DetectedTechnology {
                        name: "Express".to_string(),
                        version: None,
                        evidence: "package.json".to_string(),
                    });
                }
                if content.contains("next") {
                    detected.push(DetectedTechnology {
                        name: "Next.js".to_string(),
                        version: None,
                        evidence: "package.json".to_string(),
                    });
                }
                if content.contains("@tauri-apps") {
                    detected.push(DetectedTechnology {
                        name: "Tauri".to_string(),
                        version: None,
                        evidence: "package.json".to_string(),
                    });
                }
            }
        }

        // Rust
        if self.root_path.join("Cargo.toml").exists() {
            package_managers.push("cargo".to_string());
            detected.push(DetectedTechnology {
                name: "Rust".to_string(),
                version: None,
                evidence: "Cargo.toml".to_string(),
            });

            // Check for tokio
            if let Ok(content) = std::fs::read_to_string(self.root_path.join("Cargo.toml")) {
                if content.contains("tokio") {
                    detected.push(DetectedTechnology {
                        name: "Tokio Async Runtime".to_string(),
                        version: None,
                        evidence: "Cargo.toml".to_string(),
                    });
                }
            }
        }

        // Python
        if self.root_path.join("requirements.txt").exists()
            || self.root_path.join("pyproject.toml").exists()
        {
            package_managers.push("pip/poetry".to_string());
            detected.push(DetectedTechnology {
                name: "Python".to_string(),
                version: None,
                evidence: "requirements.txt or pyproject.toml".to_string(),
            });
        }

        // Go
        if self.root_path.join("go.mod").exists() {
            package_managers.push("go modules".to_string());
            detected.push(DetectedTechnology {
                name: "Go".to_string(),
                version: None,
                evidence: "go.mod".to_string(),
            });
        }

        // Docker
        if self.root_path.join("Dockerfile").exists() {
            detected.push(DetectedTechnology {
                name: "Docker".to_string(),
                version: None,
                evidence: "Dockerfile".to_string(),
            });
        }

        Ok(TechStack {
            detected,
            package_managers,
        })
    }

    /// Run comprehensive local risk checks with snippets
    pub async fn local_risk_checks(&self) -> crate::Result<Vec<LocalFinding>> {
        let mut findings = Vec::new();

        // Check for .env files
        findings.extend(self.check_env_files().await?);

        // Check for secrets in code
        findings.extend(self.check_code_secrets().await?);

        // Check for security anti-patterns
        findings.extend(self.check_security_patterns().await?);

        // Check for code quality issues
        findings.extend(self.check_code_quality().await?);

        // Check for configuration issues
        findings.extend(self.check_config_issues()?);

        // Check for suspicious file patterns
        findings.extend(self.check_suspicious_files()?);

        Ok(findings)
    }

    async fn check_env_files(&self) -> crate::Result<Vec<LocalFinding>> {
        let mut findings = Vec::new();
        let env_files = vec![
            ".env",
            ".env.local",
            ".env.production",
            ".env.development",
            ".env.staging",
        ];

        for env_file in env_files {
            let path = self.root_path.join(env_file);
            if path.exists() {
                let gitignore_path = self.root_path.join(".gitignore");
                let mut gitignored = false;

                if let Ok(gitignore) = tokio::fs::read_to_string(&gitignore_path).await {
                    gitignored = gitignore.contains(env_file) || gitignore.contains(".env");
                }

                if !gitignored {
                    // Read first few lines to show content (masked for security)
                    let snippet = if let Ok(content) = tokio::fs::read_to_string(&path).await {
                        let preview: Vec<String> =
                            content.lines().take(3).map(mask_secret_line).collect();
                        if preview.iter().any(|l| l.contains('=')) {
                            Some(preview.join("\n"))
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    findings.push(LocalFinding {
                        finding_type: "UNGITIGNORED_ENV".to_string(),
                        description: format!(
                            "{} exists but is not gitignored - potential secret exposure",
                            env_file
                        ),
                        file_path: env_file.to_string(),
                        severity: Severity::High,
                        line_number: Some(1),
                        snippet: snippet.or_else(|| {
                            Some("Environment file with potential secrets".to_string())
                        }),
                        recommendation: Some(format!("Add '{}' to .gitignore", env_file)),
                    });
                }

                // Check for actual secrets in .env files
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    for (line_num, line) in content.lines().enumerate() {
                        if (line.to_lowercase().contains("password")
                            || line.to_lowercase().contains("secret")
                            || line.to_lowercase().contains("api_key")
                            || line.to_lowercase().contains("token"))
                            && line.contains('=')
                            && !line.trim().ends_with('=')
                        {
                            findings.push(LocalFinding {
                                finding_type: "HARDCODED_SECRET".to_string(),
                                description: format!("Potential hardcoded secret in {}", env_file),
                                file_path: env_file.to_string(),
                                severity: Severity::Critical,
                                line_number: Some(line_num + 1),
                                snippet: Some(
                                    line.split('=').next().unwrap_or(line).to_string()
                                        + "=***REDACTED***",
                                ),
                                recommendation: Some(
                                    "Move to secure vault or use environment variable injection"
                                        .to_string(),
                                ),
                            });
                        }
                    }
                }
            }
        }

        Ok(findings)
    }

    async fn check_code_secrets(&self) -> crate::Result<Vec<LocalFinding>> {
        let mut findings = Vec::new();
        let scanned = self.scan()?;
        // Patterns to check in source code
        let secret_patterns: Vec<(&str, &str)> = vec![
            (
                "API_KEY",
                "(?i)(api[_-]?key|apikey)\\s*[=:]\\s*['\"][a-zA-Z0-9_-]{16,}['\"]",
            ),
            (
                "PASSWORD",
                "(?i)(password|passwd|pwd)\\s*[=:]\\s*['\"][^'\"]{4,}['\"]",
            ),
            (
                "SECRET",
                "(?i)(secret|private[_-]?key)\\s*[=:]\\s*['\"][a-zA-Z0-9_-]{8,}['\"]",
            ),
            (
                "TOKEN",
                "(?i)(token|auth[_-]?token)\\s*[=:]\\s*['\"][a-zA-Z0-9_-]{10,}['\"]",
            ),
            ("AWS_KEY", "AKIA[0-9A-Z]{16}"),
            ("GITHUB_TOKEN", "gh[pousr]_[A-Za-z0-9_]{36,}"),
            ("SLACK_TOKEN", "xox[baprs]-[0-9]{10,13}-[0-9]{10,13}"),
        ];

        // PERF-001: Compile regexes once before iterating files
        let compiled_patterns: Vec<(&str, regex::Regex)> = secret_patterns
            .iter()
            .filter_map(|(name, pattern)| regex::Regex::new(pattern).ok().map(|re| (*name, re)))
            .collect();

        for file in &scanned.files {
            // Only check source code files
            if !file.ends_with(".rs")
                && !file.ends_with(".js")
                && !file.ends_with(".ts")
                && !file.ends_with(".py")
                && !file.ends_with(".go")
                && !file.ends_with(".java")
                && !file.ends_with(".rb")
                && !file.ends_with(".php")
            {
                continue;
            }

            // Skip documentation files
            let doc_extensions = [".md", ".txt", ".rst", ".adoc", ".markdown"];
            let file_lower = file.to_lowercase();
            if doc_extensions.iter().any(|ext| file_lower.ends_with(ext)) {
                continue;
            }

            // Skip test/example files
            let test_patterns = ["test", "spec", "fixture", "example", "mock"];
            let file_name = std::path::Path::new(file)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if test_patterns
                .iter()
                .any(|p| file_name.to_lowercase().contains(p))
            {
                continue;
            }

            if let Ok(Some(content)) = self.read_file(file).await {
                for (pattern_name, re) in &compiled_patterns {
                    for (idx, line) in content.lines().enumerate() {
                        if re.is_match(line) {
                            // Skip comments
                            if line.trim().starts_with("//")
                                || line.trim().starts_with("#")
                                || line.trim().starts_with("(*")
                                || line.trim().starts_with("/*")
                            {
                                continue;
                            }

                            findings.push(LocalFinding {
                                finding_type: format!("POTENTIAL_{}", pattern_name),
                                description: format!(
                                    "Potential {} detected in source code",
                                    pattern_name.to_lowercase().replace("_", " ")
                                ),
                                file_path: file.clone(),
                                severity: Severity::Critical,
                                line_number: Some(idx + 1),
                                snippet: Some(mask_secret_line(line)),
                                recommendation: Some(
                                    "Use environment variables or a secrets manager".to_string(),
                                ),
                            });
                            break; // Only report first occurrence per pattern per file
                        }
                    }
                }
            }
        }

        Ok(findings)
    }

    async fn check_security_patterns(&self) -> crate::Result<Vec<LocalFinding>> {
        let mut findings = Vec::new();
        let scanned = self.scan()?;

        for file in &scanned.files {
            if let Ok(Some(content)) = self.read_file(file).await {
                // Check for SQL injection patterns
                for (idx, line) in content.lines().enumerate() {
                    let trimmed = line.trim();
                    let line_lower = trimmed.to_lowercase();

                    // Skip comment lines
                    if trimmed.starts_with("//")
                        || trimmed.starts_with("#")
                        || trimmed.starts_with("(*")
                        || trimmed.starts_with("/*")
                        || trimmed.starts_with("--")
                        || trimmed.starts_with("*")
                    {
                        continue;
                    }

                    // SQL injection risk detection
                    // Check for SQL keywords that are actually SQL (not substrings)
                    let sql_keywords = [
                        "select ", "insert ", "update ", "delete ", "drop ", "from ", "where ",
                    ];
                    let has_sql_keyword = sql_keywords.iter().any(|kw| line_lower.contains(kw));

                    // Check for string interpolation patterns that could inject user input
                    let has_formatting = line_lower.contains("format!(")
                        || line_lower.contains("format(")
                        || (line.contains("$") && line.contains("{"));

                    // Check for string concatenation patterns
                    let has_concat = line.contains("+ ") || line.contains(" +");

                    // Only flag if we have SQL keywords AND dynamic string construction
                    if has_sql_keyword && (has_formatting || has_concat) {
                        // Additional check: exclude common false positives
                        // - Variable names like "selected_item" or "updated_at"
                        // - Comments that weren't caught by the simple check above
                        let is_false_positive = line_lower.contains("selected_")
                            && !line_lower.contains("select ")
                            || line_lower.contains("updated_") && !line_lower.contains("update ")
                            || line_lower.contains("inserted_") && !line_lower.contains("insert ")
                            || line_lower.contains("from_") && !line_lower.contains(" from ")
                            || line_lower.contains("where_") && !line_lower.contains(" where ");

                        if !is_false_positive {
                            findings.push(LocalFinding {
                                finding_type: "SQL_INJECTION_RISK".to_string(),
                                description: "Potential SQL injection - string formatting with SQL"
                                    .to_string(),
                                file_path: file.clone(),
                                severity: Severity::High,
                                line_number: Some(idx + 1),
                                snippet: Some(trimmed.to_string()),
                                recommendation: Some(
                                    "Use parameterized queries or prepared statements".to_string(),
                                ),
                            });
                        }
                    }

                    // Insecure HTTP
                    if line_lower.contains("http://")
                        && !line_lower.contains("localhost")
                        && !line_lower.contains("127.0.0.1")
                    {
                        findings.push(LocalFinding {
                            finding_type: "INSECURE_HTTP".to_string(),
                            description: "Insecure HTTP URL detected".to_string(),
                            file_path: file.clone(),
                            severity: Severity::Medium,
                            line_number: Some(idx + 1),
                            snippet: Some(line.trim().to_string()),
                            recommendation: Some("Use HTTPS instead of HTTP".to_string()),
                        });
                    }

                    // eval() usage
                    if line_lower.contains("eval(") {
                        findings.push(LocalFinding {
                            finding_type: "DANGEROUS_EVAL".to_string(),
                            description: "Dangerous eval() usage detected".to_string(),
                            file_path: file.clone(),
                            severity: Severity::High,
                            line_number: Some(idx + 1),
                            snippet: Some(line.trim().to_string()),
                            recommendation: Some(
                                "Avoid eval() - use safer alternatives".to_string(),
                            ),
                        });
                    }
                }
            }
        }

        // Check package.json for suspicious postinstall
        if let Ok(content) = tokio::fs::read_to_string(self.root_path.join("package.json")).await {
            if content.contains("postinstall")
                && (content.contains("curl")
                    || content.contains("wget")
                    || content.contains("http"))
            {
                findings.push(LocalFinding {
                    finding_type: "SUSPICIOUS_POSTINSTALL".to_string(),
                    description: "package.json contains postinstall script with network activity - potential supply chain risk".to_string(),
                    file_path: "package.json".to_string(),
                    severity: Severity::High,
                    line_number: None,
                    snippet: Some("\"postinstall\": \"...\"".to_string()),
                    recommendation: Some("Review postinstall scripts for security".to_string()),
                });
            }
        }

        Ok(findings)
    }

    async fn check_code_quality(&self) -> crate::Result<Vec<LocalFinding>> {
        let mut findings = Vec::new();
        let scanned = self.scan()?;

        for file in &scanned.files {
            // Check for TODO/FIXME comments that might indicate issues
            if let Ok(Some(content)) = self.read_file(file).await {
                for (idx, line) in content.lines().enumerate() {
                    let trimmed = line.trim().to_lowercase();

                    if trimmed.contains("todo:")
                        || trimmed.contains("fixme:")
                        || trimmed.contains("hack:")
                    {
                        let severity = if trimmed.contains("security") || trimmed.contains("vuln") {
                            Severity::High
                        } else {
                            Severity::Low
                        };

                        findings.push(LocalFinding {
                            finding_type: "TODO_MARKER".to_string(),
                            description: "Code marker found".to_string(),
                            file_path: file.clone(),
                            severity,
                            line_number: Some(idx + 1),
                            snippet: Some(line.trim().to_string()),
                            recommendation: Some("Address or remove the TODO".to_string()),
                        });
                    }

                    // Check for console.log/debug in production code
                    if (file.ends_with(".js") || file.ends_with(".ts") || file.ends_with(".tsx"))
                        && (trimmed.contains("console.log(") || trimmed.contains("console.debug("))
                    {
                        findings.push(LocalFinding {
                            finding_type: "DEBUG_CODE".to_string(),
                            description: "Debug console statement in production code".to_string(),
                            file_path: file.clone(),
                            severity: Severity::Low,
                            line_number: Some(idx + 1),
                            snippet: Some(line.trim().to_string()),
                            recommendation: Some(
                                "Remove debug statements before production".to_string(),
                            ),
                        });
                    }
                }
            }
        }

        Ok(findings)
    }

    fn check_config_issues(&self) -> crate::Result<Vec<LocalFinding>> {
        let mut findings = Vec::new();

        // Check for missing README
        let has_readme = self.root_path.join("README.md").exists()
            || self.root_path.join("README.rst").exists()
            || self.root_path.join("README.txt").exists();

        if !has_readme {
            findings.push(LocalFinding {
                finding_type: "MISSING_README".to_string(),
                description: "No README file found in repository root".to_string(),
                file_path: ".".to_string(),
                severity: Severity::Low,
                line_number: None,
                snippet: None,
                recommendation: Some("Add a README.md with project description".to_string()),
            });
        }

        // Check for missing LICENSE
        let has_license = self.root_path.join("LICENSE").exists()
            || self.root_path.join("LICENSE.md").exists()
            || self.root_path.join("LICENSE.txt").exists();

        if !has_license {
            findings.push(LocalFinding {
                finding_type: "MISSING_LICENSE".to_string(),
                description: "No LICENSE file found".to_string(),
                file_path: ".".to_string(),
                severity: Severity::Info,
                line_number: None,
                snippet: None,
                recommendation: Some("Add a LICENSE file".to_string()),
            });
        }

        // Check for .gitignore
        if !self.root_path.join(".gitignore").exists() {
            findings.push(LocalFinding {
                finding_type: "MISSING_GITIGNORE".to_string(),
                description: "No .gitignore file found".to_string(),
                file_path: ".".to_string(),
                severity: Severity::Medium,
                line_number: None,
                snippet: None,
                recommendation: Some("Create .gitignore for your tech stack".to_string()),
            });
        }

        Ok(findings)
    }

    fn check_suspicious_files(&self) -> crate::Result<Vec<LocalFinding>> {
        let mut findings = Vec::new();
        let scanned = self.scan()?;

        for file in &scanned.files {
            // Check for sensitive file patterns
            let sensitive_patterns = vec![
                ("id_rsa", "SSH private key"),
                ("id_dsa", "SSH private key"),
                (".pem", "PEM certificate/key"),
                (".p12", "PKCS12 certificate"),
                (".pfx", "PFX certificate"),
                ("credentials", "Credentials file"),
                ("secret", "Secret file"),
                ("backup", "Backup file"),
                (".bak", "Backup file"),
            ];

            let file_lower = file.to_lowercase();
            for (pattern, description) in sensitive_patterns {
                if file_lower.contains(pattern) {
                    findings.push(LocalFinding {
                        finding_type: "SENSITIVE_FILE".to_string(),
                        description: format!("{} detected: {}", description, file),
                        file_path: file.clone(),
                        severity: Severity::High,
                        line_number: None,
                        snippet: None,
                        recommendation: Some(
                            "Ensure this file is gitignored and not committed".to_string(),
                        ),
                    });
                    break;
                }
            }

            // Check file permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let full_path = self.root_path.join(file);
                if let Ok(metadata) = std::fs::metadata(&full_path) {
                    let mode = metadata.permissions().mode();
                    // Check for world-writable files
                    if mode & 0o002 != 0 {
                        findings.push(LocalFinding {
                            finding_type: "WORLD_WRITABLE".to_string(),
                            description: format!("World-writable file: {}", file),
                            file_path: file.clone(),
                            severity: Severity::Medium,
                            line_number: None,
                            snippet: None,
                            recommendation: Some(
                                "Remove world-write permissions: chmod o-w".to_string(),
                            ),
                        });
                    }
                }
            }
        }

        Ok(findings)
    }

    /// Read file content with size limit.
    ///
    /// This method ensures the path is within the repository root and
    /// handles canonicalization to prevent path traversal.
    pub async fn read_file(&self, relative_path: &str) -> crate::Result<Option<String>> {
        // Prevent path traversal by ensuring the resolved path is within the root directory
        // First, validate the relative path doesn't contain dangerous patterns
        if relative_path.contains("..") || relative_path.contains("./") || relative_path.starts_with("/") {
            warn!("Suspicious path pattern detected: {}", relative_path);
            return Err(crate::HqeError::Scan(format!(
                "Invalid path pattern detected: {}",
                relative_path
            )));
        }

        let full_path = self.root_path.join(relative_path);

        if !full_path.exists() {
            return Ok(None);
        }

        // Canonicalize both paths to resolve any '..' components and verify the file is within the allowed directory
        let canonical_full_path = full_path.canonicalize().map_err(crate::HqeError::Io)?;
        let canonical_root = self.root_path.canonicalize().map_err(crate::HqeError::Io)?;

        if !canonical_full_path.starts_with(&canonical_root) {
            warn!("Path traversal attempt detected: {}", relative_path);
            return Err(crate::HqeError::Scan(format!(
                "Path traversal detected: file '{}' is outside the allowed directory",
                relative_path
            )));
        }

        let metadata = tokio::fs::metadata(&canonical_full_path)
            .await
            .map_err(crate::HqeError::Io)?;
        if metadata.len() > self.max_file_size as u64 {
            warn!("File too large to read: {}", relative_path);
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(&canonical_full_path)
            .await
            .map_err(crate::HqeError::Io)?;
        Ok(Some(content))
    }
}

/// Scanned repository summary
#[derive(Debug, Clone)]
pub struct ScannedRepo {
    /// Root path of the scanned repository
    pub root_path: PathBuf,
    /// List of file paths in the repository
    pub files: Vec<String>,
    /// List of directory paths in the repository
    pub directories: Vec<String>,
    /// Total size of the repository in bytes
    pub total_size: u64,
}

impl ScannedRepo {
    /// Build directory tree string
    pub fn tree_summary(&self, max_depth: usize) -> String {
        let mut lines = vec![".".to_string()];

        let mut sorted_dirs = self.directories.clone();
        sorted_dirs.sort();

        for dir in sorted_dirs.iter().take(50) {
            let depth = dir.split('/').count();
            if depth > max_depth {
                continue;
            }
            let indent = "  ".repeat(depth);
            lines.push(format!(
                "{}{}/",
                indent,
                dir.split('/').next_back().unwrap_or(dir)
            ));
        }

        if self.directories.len() > 50 {
            lines.push("  ...".to_string());
        }

        lines.join("\n")
    }

    /// Get key files for LLM analysis (bounded)
    pub fn key_files(&self, max_files: usize) -> Vec<String> {
        let priority_patterns = vec![
            "README",
            "CHANGELOG",
            "LICENSE",
            "package.json",
            "Cargo.toml",
            "pyproject.toml",
            "go.mod",
            "Dockerfile",
            "docker-compose",
            ".github/workflows",
            "src/main",
            "src/lib",
            "src/index",
            "app",
            "main",
            "index",
        ];

        let mut key_files: Vec<String> = self
            .files
            .iter()
            .filter(|f| {
                priority_patterns
                    .iter()
                    .any(|p| f.to_lowercase().contains(&p.to_lowercase()))
            })
            .cloned()
            .collect();

        // Add some source files if we have room
        let source_extensions = [".rs", ".ts", ".js", ".py", ".go"];
        for ext in &source_extensions {
            if key_files.len() >= max_files {
                break;
            }
            for file in &self.files {
                if file.ends_with(ext) && !key_files.contains(file) {
                    key_files.push(file.clone());
                    if key_files.len() >= max_files {
                        break;
                    }
                }
            }
        }

        key_files.truncate(max_files);
        key_files
    }
}

fn should_exclude_dir(path: &str) -> bool {
    let excluded = [
        ".git",
        ".svn",
        ".hg",
        "node_modules",
        "target",
        "dist",
        "build",
        ".next",
        ".nuxt",
        ".vuepress",
        "__pycache__",
        ".pytest_cache",
        ".idea",
        ".vscode",
    ];

    excluded.iter().any(|e| path.contains(e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_scan_finds_files() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("test.txt"), "hello").unwrap();
        std::fs::create_dir(temp.path().join("src")).unwrap();
        std::fs::write(temp.path().join("src/main.rs"), "fn main() {}").unwrap();

        let scanner = RepoScanner::new(temp.path());
        let repo = scanner.scan().unwrap();

        assert!(repo.files.contains(&"test.txt".to_string()));
        assert!(repo.files.contains(&"src/main.rs".to_string()));
    }

    #[test]
    fn test_detect_entrypoints() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("package.json"), r#"{"name":"test"}"#).unwrap();
        std::fs::create_dir(temp.path().join("src")).unwrap();
        std::fs::write(temp.path().join("src/main.rs"), "fn main() {}").unwrap();

        let scanner = RepoScanner::new(temp.path());
        let entrypoints = scanner.detect_entrypoints().unwrap();

        assert!(entrypoints.iter().any(|e| e.file_path == "package.json"));
    }

    #[test]
    fn test_detect_tech_stack() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("Cargo.toml"), "[package]").unwrap();

        let scanner = RepoScanner::new(temp.path());
        let stack = scanner.detect_tech_stack().unwrap();

        assert!(stack.detected.iter().any(|t| t.name == "Rust"));
        assert!(stack.package_managers.contains(&"cargo".to_string()));
    }

    #[tokio::test]
    async fn test_local_risk_checks_env() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join(".env"), "SECRET=123").unwrap();
        // No .gitignore

        let scanner = RepoScanner::new(temp.path());
        let findings = scanner.local_risk_checks().await.unwrap();

        assert!(findings
            .iter()
            .any(|f| f.finding_type == "UNGITIGNORED_ENV"));
    }

    #[tokio::test]
    async fn test_path_traversal_protection() {
        let temp_parent = TempDir::new().unwrap();
        let temp_child = temp_parent.path().join("child_dir");
        std::fs::create_dir(&temp_child).unwrap();
        std::fs::write(temp_child.join("allowed_file.txt"), "content").unwrap();

        // Create a "sensitive" file in the parent directory
        std::fs::write(temp_parent.path().join("sensitive.txt"), "secret").unwrap();

        let scanner = RepoScanner::new(&temp_child);

        // This should work - reading a file within the allowed directory
        let result = scanner.read_file("allowed_file.txt").await.unwrap();
        assert!(result.is_some());

        // This should fail - path traversal attempt to access parent directory
        let result = scanner.read_file("../sensitive.txt").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Path traversal detected"));
    }

    #[test]
    fn test_key_files_priority() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("README.md"), "# Test").unwrap();
        std::fs::create_dir(temp.path().join("src")).unwrap();
        std::fs::write(temp.path().join("src/main.rs"), "fn main() {}").unwrap();
        std::fs::write(temp.path().join("ignored.txt"), "ignore me").unwrap();

        let scanner = RepoScanner::new(temp.path());
        let repo = scanner.scan().unwrap();
        let key = repo.key_files(10);

        assert!(key.contains(&"README.md".to_string()));
        assert!(key.contains(&"src/main.rs".to_string()));
    }

    #[tokio::test]
    async fn test_sql_injection_detection_logic() {
        let temp = TempDir::new().unwrap();

        // Create a file with SQL keywords but no format patterns - should NOT trigger detection
        std::fs::write(
            temp.path().join("test1.rs"),
            r#"
            // This should NOT be flagged as SQL injection because there's no formatting
            let query = "SELECT * FROM users";
            println!("{}", query);
        "#,
        )
        .unwrap();

        // Create a file with format patterns but no SQL keywords - should NOT trigger detection
        std::fs::write(
            temp.path().join("test2.rs"),
            r#"
            // This should NOT be flagged as SQL injection because there are no SQL keywords
            let msg = format!("Hello {}", name);
            println!("{}", msg);
        "#,
        )
        .unwrap();

        // Create a file with both SQL keywords AND format patterns - SHOULD trigger detection
        std::fs::write(temp.path().join("test3.rs"), r#"
            // This SHOULD be flagged as SQL injection because it has both SQL keywords and formatting
            let query = format!("SELECT * FROM users WHERE id = {}", user_id);
        "#).unwrap();

        let scanner = RepoScanner::new(temp.path());
        let findings = scanner.local_risk_checks().await.unwrap();

        // Count SQL injection findings
        let sql_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.finding_type == "SQL_INJECTION_RISK")
            .collect();

        // Should only have 1 finding (the one with both SQL keywords and formatting)
        assert_eq!(sql_findings.len(), 1);

        // The finding should be in test3.rs
        assert!(sql_findings
            .iter()
            .any(|f| f.file_path.contains("test3.rs")));
    }
}
