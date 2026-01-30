//! Data models for HQE protocol

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Current HQE protocol version
pub const HQE_PROTOCOL_VERSION: &str = "3.1.0";
/// Current HQE schema version
pub const HQE_SCHEMA_VERSION: &str = "3.1.0";

/// Run manifest - top-level metadata for a scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunManifest {
    /// Unique identifier for this scan run
    pub run_id: String,
    /// Information about the repository being scanned
    pub repo: RepoInfo,
    /// Information about the provider used for the scan
    pub provider: ProviderInfo,
    /// Limits applied during the scan
    pub limits: ScanLimits,
    /// Timestamps for the scan execution
    pub timestamps: Timestamps,
    /// Protocol and schema versions used
    pub protocol: ProtocolVersions,
}

impl RunManifest {
    /// Creates a new RunManifest with the given repository path and provider name
    pub fn new(repo_path: impl Into<String>, provider_name: impl Into<String>) -> Self {
        let now = Utc::now();
        let uuid_short = &Uuid::new_v4().to_string()[..8];
        let run_id = format!("{}_{}", now.format("%Y-%m-%dT%H-%M-%SZ"), uuid_short);

        Self {
            run_id,
            repo: RepoInfo {
                source: RepoSource::Local,
                path: repo_path.into(),
                git_remote: None,
                git_commit: None,
            },
            provider: ProviderInfo {
                name: provider_name.into(),
                base_url: None,
                model: None,
                llm_enabled: false,
            },
            limits: ScanLimits::default(),
            timestamps: Timestamps {
                started: now,
                ended: None,
            },
            protocol: ProtocolVersions {
                protocol_version: HQE_PROTOCOL_VERSION.to_string(),
                schema_version: HQE_SCHEMA_VERSION.to_string(),
            },
        }
    }
}

/// Source type for a repository
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoSource {
    /// Local filesystem repository
    Local,
    /// Git repository
    Git,
}

/// Repository metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    /// Source type (local or git)
    pub source: RepoSource,
    /// Path to the repository
    pub path: String,
    /// Git remote URL if available
    pub git_remote: Option<String>,
    /// Current git commit hash if available
    pub git_commit: Option<String>,
}

/// LLM provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// Provider name
    pub name: String,
    /// Provider API base URL
    pub base_url: Option<String>,
    /// Model being used
    pub model: Option<String>,
    /// Whether LLM features are enabled
    pub llm_enabled: bool,
}

/// Limits for scan operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanLimits {
    /// Maximum number of files to send to LLM
    pub max_files_sent: usize,
    /// Maximum total characters to send
    pub max_total_chars_sent: usize,
    /// Character limit per code snippet
    pub snippet_chars: usize,
}

impl Default for ScanLimits {
    fn default() -> Self {
        Self {
            max_files_sent: 40,
            max_total_chars_sent: 250_000,
            snippet_chars: 4_000,
        }
    }
}

/// Scan timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timestamps {
    /// When the scan started
    pub started: DateTime<Utc>,
    /// When the scan ended (None if still running)
    pub ended: Option<DateTime<Utc>>,
}

/// Protocol and schema version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolVersions {
    /// HQE protocol version
    pub protocol_version: String,
    /// HQE schema version
    pub schema_version: String,
}

/// Complete HQE Report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HqeReport {
    /// Unique identifier for this scan run
    pub run_id: String,
    /// Executive summary with health score and top priorities
    pub executive_summary: ExecutiveSummary,
    /// Project map with architecture and tech stack information
    pub project_map: ProjectMap,
    /// PR harvest information if pull requests were analyzed
    pub pr_harvest: Option<PrHarvest>,
    /// Deep scan results organized by category
    pub deep_scan_results: DeepScanResults,
    /// Master TODO backlog with actionable items
    pub master_todo_backlog: Vec<TodoItem>,
    /// Implementation plan with short, medium, and long term items
    pub implementation_plan: ImplementationPlan,
    /// Immediate actions with patch information
    pub immediate_actions: Vec<PatchAction>,
    /// Session log with completed, in-progress, and discovered items
    pub session_log: SessionLog,
}

/// Section 1: Executive Summary
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutiveSummary {
    /// Health score from 1-10 indicating overall codebase health
    pub health_score: u8, // 1-10
    /// Top priority items that need attention
    pub top_priorities: Vec<String>,
    /// Critical findings that require immediate attention
    pub critical_findings: Vec<String>,
    /// Blockers that prevent progress
    pub blockers: Vec<Blocker>,
}

/// A blocking issue that prevents progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blocker {
    /// Description of the blocker
    pub description: String,
    /// Why this is blocking
    pub reason: String,
    /// How to resolve the blocker
    pub how_to_obtain: String,
}

/// Section 2: Project Map
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectMap {
    /// Architecture information including languages, frameworks, etc.
    pub architecture: Architecture,
    /// List of application entry points
    pub entrypoints: Vec<Entrypoint>,
    /// Data flow description if available
    pub data_flow: Option<String>,
    /// Technology stack information
    pub tech_stack: TechStack,
}

/// Project architecture information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Architecture {
    /// Programming languages used
    pub languages: Vec<String>,
    /// Frameworks used
    pub frameworks: Vec<String>,
    /// Runtime environments
    pub runtimes: Vec<String>,
    /// Description of frontend/backend architecture
    pub frontend_backend_separation: Option<String>,
    /// Databases used
    pub databases: Vec<String>,
    /// Message queues used
    pub message_queues: Vec<String>,
    /// Third-party services integrated
    pub third_party_services: Vec<String>,
    /// Build system used
    pub build_system: Option<String>,
}

/// Application entry point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entrypoint {
    /// Path to the entry point file
    pub file_path: String,
    /// Type of entry point (e.g., "main", "server", "cli")
    pub entry_type: String,
    /// Description of what this entry point does
    pub description: String,
}

/// Detected technology stack
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TechStack {
    /// Technologies detected in the project
    pub detected: Vec<DetectedTechnology>,
    /// Package managers used
    pub package_managers: Vec<String>,
}

/// A detected technology in the project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedTechnology {
    /// Technology name
    pub name: String,
    /// Detected version if available
    pub version: Option<String>,
    /// Evidence of this technology (file path, config content, etc.)
    pub evidence: String,
}

/// Section 3: PR Harvest
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrHarvest {
    /// Inventory of pull requests analyzed
    pub inventory: Vec<PrInfo>,
    /// Conflicts identified between pull requests
    pub conflicts: Vec<PrConflict>,
}

/// Pull request information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrInfo {
    /// PR identifier
    pub pr_id: String,
    /// PR title
    pub title: String,
    /// Current status
    pub status: String,
    /// Intent/purpose of the PR
    pub intent: String,
    /// Files modified by this PR
    pub files_touched: Vec<String>,
    /// Risk level
    pub risk: RiskLevel,
    /// Recommendation for this PR
    pub recommendation: PrRecommendation,
}

/// Recommendation for a pull request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrRecommendation {
    /// Accept the PR as-is
    Accept,
    /// Accept with modifications
    Modify,
    /// Reject the PR
    Reject,
}

/// Conflict between two pull requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrConflict {
    /// Description of the conflict
    pub description: String,
    /// First PR ID
    pub pr_x: String,
    /// Approach taken by first PR
    pub approach_x: String,
    /// Second PR ID
    pub pr_y: String,
    /// Approach taken by second PR
    pub approach_y: String,
    /// Suggested resolution
    pub resolution: String,
    /// Files affected by the conflict
    pub affected_files: Vec<String>,
}

/// Section 4: Deep Scan Results
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeepScanResults {
    /// Security-related findings
    pub security: Vec<Finding>,
    /// Code quality findings
    pub code_quality: Vec<Finding>,
    /// Frontend-specific findings
    pub frontend: Vec<Finding>,
    /// Backend-specific findings
    pub backend: Vec<Finding>,
    /// Testing-related findings
    pub testing: Vec<Finding>,
}

/// A finding from deep scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Unique identifier for this finding
    pub id: String,
    /// Severity level of the finding
    pub severity: Severity,
    /// Risk level assessment
    pub risk: RiskLevel,
    /// Category of the finding
    pub category: String,
    /// Title of the finding
    pub title: String,
    /// Evidence supporting this finding
    pub evidence: Evidence,
    /// Impact of the issue
    pub impact: String,
    /// Recommendation for addressing the issue
    pub recommendation: String,
}

/// Severity level of a finding
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Critical severity - requires immediate attention
    Critical,
    /// High severity - should be addressed soon
    High,
    /// Medium severity - should be addressed
    Medium,
    /// Low severity - minor issue
    Low,
    /// Informational - no action required
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "Critical"),
            Severity::High => write!(f, "High"),
            Severity::Medium => write!(f, "Medium"),
            Severity::Low => write!(f, "Low"),
            Severity::Info => write!(f, "Info"),
        }
    }
}

/// Risk level assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    /// Low risk
    Low,
    /// Medium risk
    Medium,
    /// High risk
    High,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "Low"),
            RiskLevel::Medium => write!(f, "Medium"),
            RiskLevel::High => write!(f, "High"),
        }
    }
}

/// Evidence for a finding
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Evidence {
    /// Evidence at a specific line in a file
    FileLine {
        /// File path
        file: String,
        /// Line number
        line: usize,
        /// Code snippet
        snippet: String,
    },
    /// Evidence in a specific function
    FileFunction {
        /// File path
        file: String,
        /// Function name
        function: String,
        /// Code snippet
        snippet: String,
    },
    /// Reproduction steps
    Reproduction {
        /// Steps to reproduce
        steps: Vec<String>,
        /// Observed behavior
        observed: String,
    },
}

/// Section 5: Master TODO Backlog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    /// Unique identifier for this TODO item
    pub id: String,
    /// Severity level of the issue
    pub severity: Severity,
    /// Risk level assessment
    pub risk: RiskLevel,
    /// Category of the TODO item
    pub category: TodoCategory,
    /// Title of the TODO item
    pub title: String,
    /// Root cause of the issue
    pub root_cause: String,
    /// Evidence supporting this finding
    pub evidence: Evidence,
    /// Approach to fix the issue
    pub fix_approach: String,
    /// Verification steps to confirm the fix
    pub verify: String,
    /// Optional ID of another TODO item that blocks this one
    pub blocked_by: Option<String>,
}

/// Category for TODO items
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TodoCategory {
    /// Bootstrap/setup issues
    Boot,
    /// Security issues
    Sec,
    /// Bug fixes
    Bug,
    /// Performance improvements
    Perf,
    /// User experience
    Ux,
    /// Developer experience
    Dx,
    /// Documentation
    Doc,
    /// Technical debt
    Debt,
    /// Dependency updates
    Deps,
}

impl std::fmt::Display for TodoCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TodoCategory::Boot => write!(f, "BOOT"),
            TodoCategory::Sec => write!(f, "SEC"),
            TodoCategory::Bug => write!(f, "BUG"),
            TodoCategory::Perf => write!(f, "PERF"),
            TodoCategory::Ux => write!(f, "UX"),
            TodoCategory::Dx => write!(f, "DX"),
            TodoCategory::Doc => write!(f, "DOC"),
            TodoCategory::Debt => write!(f, "DEBT"),
            TodoCategory::Deps => write!(f, "DEPS"),
        }
    }
}

/// Section 6: Implementation Plan
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImplementationPlan {
    /// Immediate actions to take (next 1-2 weeks)
    pub immediate: Vec<String>,
    /// Short-term items (next 1-2 months)
    pub short_term: Vec<String>,
    /// Medium-term items (next 3-6 months)
    pub medium_term: Vec<String>,
    /// Long-term items (next 6+ months)
    pub long_term: Vec<String>,
    /// Dependency relationships between items
    pub dependency_graph: HashMap<String, Vec<String>>,
    /// Risk assessments for implementation items
    pub risk_assessment: Vec<RiskAssessment>,
}

/// Risk assessment for an implementation item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Item ID being assessed
    pub item_id: String,
    /// Risk mitigation strategy
    pub mitigation: String,
}

/// Section 7: Immediate Actions (Patches)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchAction {
    /// ID of the TODO item this patch addresses
    pub todo_id: String,
    /// Title of the patch
    pub title: String,
    /// Description of the problem being solved
    pub problem: String,
    /// Root cause of the issue
    pub root_cause: String,
    /// Risk level of applying this patch
    pub risk: RiskLevel,
    /// Whether this patch introduces a behavior change
    pub behavior_change: bool,
    /// List of file diffs to apply
    pub diffs: Vec<FileDiff>,
    /// Verification steps to confirm the patch works
    pub verification: Vec<VerificationStep>,
    /// Instructions for rolling back the patch if needed
    pub rollback: String,
}

/// A file diff for a patch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    /// Path to the file
    pub file_path: String,
    /// Diff content (unified diff format)
    pub diff_content: String,
}

/// A verification step for testing a patch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationStep {
    /// Command to run
    pub command: String,
    /// Expected output
    pub expected_output: String,
}

/// Section 8: Session Log
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionLog {
    /// Items completed in this session
    pub completed: Vec<String>,
    /// Items currently in progress
    pub in_progress: Vec<String>,
    /// New items discovered during the session
    pub discovered: Vec<String>,
    /// Items that were reprioritized during the session
    pub reprioritized: Vec<String>,
    /// Items planned for the next session
    pub next_session: Vec<String>,
}

/// Session log entry for individual items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLogEntry {
    /// Unique identifier for the item
    pub item_id: String,
    /// Title of the item
    pub title: String,
    /// Current status of the item
    pub status: SessionItemStatus,
}

/// Status of a session log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionItemStatus {
    /// Item was completed
    Completed,
    /// Item is in progress
    InProgress,
    /// New item discovered
    Discovered,
    /// Item was reprioritized
    Reprioritized,
}

// Re-export ProviderProfile from hqe-protocol for consistency
pub use hqe_protocol::models::ProviderProfile;

/// Scan configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    /// Whether LLM features are enabled for this scan
    pub llm_enabled: bool,
    /// Name of the provider profile to use for LLM operations
    pub provider_profile: Option<String>,
    /// Limits applied during the scan
    pub limits: ScanLimits,
    /// Whether to run in local-only mode without LLM
    pub local_only: bool,
    /// Timeout in seconds for LLM operations (0 means no timeout)
    pub timeout_seconds: u64,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            llm_enabled: false,
            provider_profile: None,
            limits: ScanLimits::default(),
            local_only: true,
            timeout_seconds: 120, // 2 minute default for LLM operations
        }
    }
}

/// Ingested file with content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestedFile {
    /// File path
    pub path: String,
    /// File content
    pub content: String,
    /// Size in bytes
    pub size_bytes: usize,
    /// Detected programming language
    pub language: Option<String>,
    /// Whether this is an entry point
    pub is_entrypoint: bool,
}

/// Summary of redactions performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionSummary {
    /// Total number of redactions
    pub total_redactions: usize,
    /// Redactions grouped by type
    pub by_type: HashMap<String, usize>,
}

/// Evidence bundle sent to LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceBundle {
    /// Repository summary information
    pub repo_summary: RepoSummary,
    /// File snippets for analysis
    pub files: Vec<FileSnippet>,
    /// Local findings from initial scan
    pub local_findings: Vec<LocalFinding>,
}

/// Repository summary for evidence bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoSummary {
    /// Repository name
    pub name: String,
    /// Current commit hash
    pub commit_hash: Option<String>,
    /// Directory tree representation
    pub directory_tree: String,
    /// Detected tech stack
    pub tech_stack: TechStack,
    /// Application entry points
    pub entrypoints: Vec<Entrypoint>,
}

/// A code snippet from a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSnippet {
    /// File path
    pub path: String,
    /// Snippet content
    pub content: String,
    /// Starting line number (1-indexed)
    pub start_line: Option<usize>,
    /// Ending line number (1-indexed)
    pub end_line: Option<usize>,
}

/// A finding from local (non-LLM) analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalFinding {
    /// Type of finding (e.g., "secret", "sql_injection")
    pub finding_type: String,
    /// Description of the issue
    pub description: String,
    /// File path where found
    pub file_path: String,
    /// Severity level
    pub severity: Severity,
    /// Line number if applicable
    pub line_number: Option<usize>,
    /// Code snippet showing the issue
    pub snippet: Option<String>,
    /// Recommendation for fixing
    pub recommendation: Option<String>,
}
