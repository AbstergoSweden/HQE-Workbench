//! HQE Scan pipeline

use crate::models::*;
use crate::redaction::RedactionEngine;
use crate::repo::RepoScanner;
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, instrument, warn};

/// Scan pipeline phases
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScanPhase {
    /// Phase 1: Repository ingestion and content analysis
    Ingestion,
    /// Phase 2: Analysis (local + optional LLM)
    Analysis,
    /// Phase 3: Report generation
    ReportGeneration,
    /// Phase 4: Artifact export
    ArtifactExport,
}

impl std::fmt::Display for ScanPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanPhase::Ingestion => write!(f, "Ingestion"),
            ScanPhase::Analysis => write!(f, "Analysis"),
            ScanPhase::ReportGeneration => write!(f, "Report Generation"),
            ScanPhase::ArtifactExport => write!(f, "Artifact Export"),
        }
    }
}

/// Trait for LLM-backed analysis implementations.
#[async_trait]
pub trait LlmAnalyzer: Send + Sync {
    /// Analyze the evidence bundle and return structured findings/todos.
    async fn analyze(&self, bundle: EvidenceBundle) -> crate::Result<AnalysisResult>;
}

/// Pipeline for running an HQE scan
pub struct ScanPipeline {
    config: ScanConfig,
    redaction: RedactionEngine,
    manifest: RunManifest,
    phase: ScanPhase,
    llm_analyzer: Option<Arc<dyn LlmAnalyzer>>,
}

impl ScanPipeline {
    /// Creates a new ScanPipeline for the given repository path and configuration
    pub fn new(repo_path: impl AsRef<Path>, config: ScanConfig) -> crate::Result<Self> {
        let provider_name = config
            .provider_profile
            .clone()
            .unwrap_or_else(|| "local".to_string());
        let mut manifest = RunManifest::new(
            repo_path.as_ref().to_string_lossy().to_string(),
            provider_name,
        );
        manifest.provider.llm_enabled = config.llm_enabled && !config.local_only;

        Ok(Self {
            config,
            redaction: RedactionEngine::new(),
            manifest,
            phase: ScanPhase::Ingestion,
            llm_analyzer: None,
        })
    }

    /// Attach an LLM analyzer implementation.
    pub fn with_llm_analyzer(mut self, analyzer: Arc<dyn LlmAnalyzer>) -> Self {
        self.llm_analyzer = Some(analyzer);
        self
    }

    /// Update provider metadata in the run manifest.
    pub fn set_provider_info(&mut self, provider: ProviderInfo) {
        self.manifest.provider = provider;
    }

    /// Get current phase
    pub fn current_phase(&self) -> ScanPhase {
        self.phase
    }

    /// Run the complete scan pipeline
    #[instrument(skip(self))]
    pub async fn run(&mut self) -> crate::Result<ScanResult> {
        info!("Starting HQE scan pipeline");

        // Phase A: Ingestion
        self.phase = ScanPhase::Ingestion;
        info!("Phase: {}", self.phase);
        let ingestion = self.run_ingestion().await?;

        // Phase B: Analysis (local + optional LLM)
        self.phase = ScanPhase::Analysis;
        info!("Phase: {}", self.phase);
        let analysis = if self.config.local_only || !self.config.llm_enabled {
            self.run_local_analysis(
                &ingestion,
                Some(Blocker {
                    description: "LLM analysis disabled - Local mode only".to_string(),
                    reason: "Local-only mode provides static analysis without LLM insights"
                        .to_string(),
                    how_to_obtain:
                        "Configure LLM provider in settings for AI-powered analysis and patch generation"
                            .to_string(),
                }),
            )
            .await?
        } else {
            match &self.llm_analyzer {
                Some(analyzer) => match analyzer
                    .analyze(self.build_evidence_bundle(&ingestion))
                    .await
                {
                    Ok(result) => result,
                    Err(err) => {
                        warn!(
                            "LLM analysis failed, falling back to local analysis: {}",
                            err
                        );
                        self.run_local_analysis(
                            &ingestion,
                            Some(Blocker {
                                description: "LLM analysis failed".to_string(),
                                reason: err.to_string(),
                                how_to_obtain: "Verify provider configuration and retry"
                                    .to_string(),
                            }),
                        )
                        .await?
                    }
                },
                None => {
                    warn!("LLM analyzer not configured, using local analysis");
                    self.run_local_analysis(
                        &ingestion,
                        Some(Blocker {
                            description: "LLM analyzer not configured".to_string(),
                            reason: "No LLM provider configured for this scan".to_string(),
                            how_to_obtain: "Configure a provider profile and retry".to_string(),
                        }),
                    )
                    .await?
                }
            }
        };

        // Phase C: Report Generation
        self.phase = ScanPhase::ReportGeneration;
        info!("Phase: {}", self.phase);
        let report = self.generate_report(&ingestion, &analysis).await?;

        // Phase D: Artifact Export (delegated to caller)
        self.phase = ScanPhase::ArtifactExport;
        info!("Phase: {} (delegated)", self.phase);
        let artifacts = self.export_artifacts(&report).await?;

        info!("Scan pipeline complete");

        Ok(ScanResult {
            manifest: self.manifest.clone(),
            report,
            artifacts,
        })
    }

    /// Phase A: Local repo ingestion
    async fn run_ingestion(&mut self) -> crate::Result<IngestionResult> {
        let scanner = RepoScanner::new(&self.manifest.repo.path);

        // Scan repository structure
        let repo = scanner.scan()?;

        // Detect entrypoints
        let entrypoints = scanner.detect_entrypoints()?;

        // Detect tech stack
        let tech_stack = scanner.detect_tech_stack()?;

        // Run local risk checks
        let local_findings = scanner.local_risk_checks().await?;

        // Get key files content
        let key_files = repo.key_files(self.config.limits.max_files_sent);
        let mut file_contents = Vec::new();

        for file_path in key_files {
            if let Ok(Some(content)) = scanner.read_file(&file_path).await {
                // Redact secrets before storing
                let redacted = self.redaction.redact(&content);
                file_contents.push(IngestedFile {
                    path: file_path.clone(),
                    content: redacted,
                    size_bytes: content.len(),
                    language: detect_language(&file_path),
                    is_entrypoint: entrypoints.iter().any(|e| e.file_path == file_path),
                });
            }
        }

        // Build repo summary
        let repo_summary = RepoSummary {
            name: Path::new(&self.manifest.repo.path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            commit_hash: self.manifest.repo.git_commit.clone(),
            directory_tree: repo.tree_summary(3),
            tech_stack: tech_stack.clone(),
            entrypoints: entrypoints.clone(),
        };

        let redaction_summary = self.redaction.summary();

        Ok(IngestionResult {
            repo_summary,
            files: file_contents,
            local_findings,
            redaction_summary,
        })
    }

    /// Phase B: Local analysis (LLM disabled)
    async fn run_local_analysis(
        &self,
        ingestion: &IngestionResult,
        blocker: Option<Blocker>,
    ) -> crate::Result<AnalysisResult> {
        // Build partial report from local findings
        let mut findings = Vec::new();

        // Convert local findings to formal findings with detailed snippets
        for (idx, local) in ingestion.local_findings.iter().enumerate() {
            let severity = local.severity.clone();
            let id = format!("LOCAL-{:03}", idx + 1);

            let evidence = match (&local.line_number, &local.snippet) {
                (Some(line), Some(snippet)) => Evidence::FileLine {
                    file: local.file_path.clone(),
                    line: *line,
                    snippet: snippet.clone(),
                },
                _ => Evidence::FileLine {
                    file: local.file_path.clone(),
                    line: local.line_number.unwrap_or(1),
                    snippet: local
                        .snippet
                        .clone()
                        .unwrap_or_else(|| "Detected via local heuristics".to_string()),
                },
            };

            findings.push(Finding {
                id,
                severity,
                risk: RiskLevel::Medium,
                category: "Security".to_string(),
                title: local.description.clone(),
                evidence,
                impact: "Potential security risk".to_string(),
                recommendation: local
                    .recommendation
                    .clone()
                    .unwrap_or_else(|| "Review and remediate".to_string()),
            });
        }

        // Generate TODO items from findings
        let todos: Vec<TodoItem> = findings
            .iter()
            .map(|f| TodoItem {
                id: f.id.clone(),
                severity: f.severity.clone(),
                risk: f.risk.clone(),
                category: TodoCategory::Sec,
                title: f.title.clone(),
                root_cause: "Detected by local scan".to_string(),
                evidence: f.evidence.clone(),
                fix_approach: f.recommendation.clone(),
                verify: "Run hqe scan again".to_string(),
                blocked_by: None,
            })
            .collect();

        Ok(AnalysisResult {
            findings,
            todos,
            is_partial: blocker.is_some(),
            blockers: blocker.into_iter().collect(),
        })
    }

    fn build_evidence_bundle(&self, ingestion: &IngestionResult) -> EvidenceBundle {
        let snippet_chars = self.config.limits.snippet_chars;
        let files = ingestion
            .files
            .iter()
            .map(|file| {
                let content = if file.content.len() > snippet_chars {
                    file.content[..snippet_chars].to_string()
                } else {
                    file.content.clone()
                };
                FileSnippet {
                    path: file.path.clone(),
                    content,
                    start_line: None,
                    end_line: None,
                }
            })
            .collect();

        EvidenceBundle {
            repo_summary: ingestion.repo_summary.clone(),
            files,
            local_findings: ingestion.local_findings.clone(),
        }
    }

    /// Phase C: Report generation
    async fn generate_report(
        &self,
        ingestion: &IngestionResult,
        analysis: &AnalysisResult,
    ) -> crate::Result<HqeReport> {
        // Calculate weighted health score based on findings
        let critical_count = analysis
            .findings
            .iter()
            .filter(|f| matches!(f.severity, Severity::Critical))
            .count();
        let high_count = analysis
            .findings
            .iter()
            .filter(|f| matches!(f.severity, Severity::High))
            .count();
        let medium_count = analysis
            .findings
            .iter()
            .filter(|f| matches!(f.severity, Severity::Medium))
            .count();
        let low_count = analysis
            .findings
            .iter()
            .filter(|f| matches!(f.severity, Severity::Low))
            .count();

        // Weighted scoring: critical (10 pts), high (5 pts), medium (2 pts), low (0.5 pts)
        let weighted_penalty = (critical_count * 10) as f32
            + (high_count * 5) as f32
            + (medium_count * 2) as f32
            + (low_count as f32 * 0.5);

        // Scale penalty to 0-10 range (capping at 100 weighted points for 0 score)
        let penalty_scaled = (weighted_penalty / 10.0).min(10.0);
        let health_score = (10.0 - penalty_scaled).max(0.0) as u8;

        // Build executive summary
        let mut priority_findings: Vec<&Finding> = analysis.findings.iter().collect();
        priority_findings.sort_by_key(|f| (severity_rank(&f.severity), risk_rank(&f.risk)));
        priority_findings.reverse();

        let executive_summary = ExecutiveSummary {
            health_score,
            top_priorities: priority_findings
                .iter()
                .take(3)
                .map(|f| format!("{}: {}", f.id, f.title))
                .collect(),
            critical_findings: analysis
                .findings
                .iter()
                .filter(|f| matches!(f.severity, Severity::Critical))
                .map(|f| f.title.clone())
                .collect(),
            blockers: if analysis.is_partial {
                analysis.blockers.clone()
            } else {
                vec![]
            },
        };

        // Build project map
        let project_map = ProjectMap {
            architecture: Architecture {
                languages: ingestion
                    .repo_summary
                    .tech_stack
                    .detected
                    .iter()
                    .map(|t| t.name.clone())
                    .collect(),
                frameworks: vec![],
                runtimes: vec![],
                frontend_backend_separation: None,
                databases: vec![],
                message_queues: vec![],
                third_party_services: vec![],
                build_system: None,
            },
            entrypoints: ingestion.repo_summary.entrypoints.clone(),
            data_flow: None,
            tech_stack: ingestion.repo_summary.tech_stack.clone(),
        };

        // Build deep scan results (categorized)
        let normalized_findings = normalize_findings(&analysis.findings);

        let mut deep_scan_results = DeepScanResults::default();
        for finding in &normalized_findings {
            match categorize_finding(finding) {
                DeepScanBucket::Security => deep_scan_results.security.push(finding.clone()),
                DeepScanBucket::CodeQuality => deep_scan_results.code_quality.push(finding.clone()),
                DeepScanBucket::Frontend => deep_scan_results.frontend.push(finding.clone()),
                DeepScanBucket::Backend => deep_scan_results.backend.push(finding.clone()),
                DeepScanBucket::Testing => deep_scan_results.testing.push(finding.clone()),
            }
        }

        // Build implementation plan
        let implementation_plan = ImplementationPlan {
            immediate: analysis
                .todos
                .iter()
                .filter(|t| matches!(t.severity, Severity::Critical | Severity::High))
                .map(|t| format!("{}: {}", t.id, t.title))
                .collect(),
            short_term: analysis
                .todos
                .iter()
                .filter(|t| matches!(t.severity, Severity::Medium))
                .take(5)
                .map(|t| format!("{}: {}", t.id, t.title))
                .collect(),
            medium_term: vec![],
            long_term: vec![],
            dependency_graph: HashMap::new(),
            risk_assessment: vec![],
        };

        // Build session log
        let mut completed = vec!["Ingestion".to_string()];
        if self.manifest.provider.llm_enabled {
            if analysis.is_partial {
                completed.push("Local Analysis (fallback)".to_string());
            } else {
                completed.push("LLM Analysis".to_string());
            }
        } else {
            completed.push("Local Analysis".to_string());
        }

        let mut in_progress = Vec::new();
        let mut next_session = Vec::new();
        if analysis.is_partial {
            if self.manifest.provider.llm_enabled {
                in_progress.push("Waiting for LLM analysis".to_string());
                next_session.push("Retry LLM analysis".to_string());
            } else {
                next_session.push("Enable LLM provider for full analysis".to_string());
            }
        }

        let session_log = SessionLog {
            completed,
            in_progress,
            discovered: analysis.findings.iter().map(|f| f.id.clone()).collect(),
            reprioritized: vec![],
            next_session,
        };

        Ok(HqeReport {
            run_id: self.manifest.run_id.clone(),
            provider: Some(self.manifest.provider.clone()),
            executive_summary,
            project_map,
            pr_harvest: None,
            deep_scan_results,
            master_todo_backlog: analysis.todos.clone(),
            implementation_plan,
            immediate_actions: vec![],
            session_log,
        })
    }

    /// Phase D: Artifact export
    async fn export_artifacts(&self, _report: &HqeReport) -> crate::Result<ArtifactPaths> {
        // Artifact writing is handled by callers (CLI/UI) via hqe-artifacts.
        Ok(ArtifactPaths::empty())
    }
}

#[derive(Debug, Clone, Copy)]
enum DeepScanBucket {
    Security,
    CodeQuality,
    Frontend,
    Backend,
    Testing,
}

fn categorize_finding(finding: &Finding) -> DeepScanBucket {
    let category = finding.category.to_lowercase();
    let file_hint = match &finding.evidence {
        Evidence::FileLine { file, .. } => file.as_str(),
        Evidence::FileFunction { file, .. } => file.as_str(),
        Evidence::Reproduction { .. } => "",
    };
    let combined = format!("{} {}", category, file_hint.to_lowercase());

    if combined.contains("sec") || combined.contains("security") {
        return DeepScanBucket::Security;
    }

    if combined.contains("test")
        || combined.contains("__tests__")
        || combined.contains("spec")
        || combined.contains("specs")
    {
        return DeepScanBucket::Testing;
    }

    if combined.contains("frontend")
        || combined.contains("ui")
        || combined.contains("ux")
        || combined.contains("client")
        || combined.contains("web")
        || combined.contains("apps/workbench")
    {
        return DeepScanBucket::Frontend;
    }

    if combined.contains("backend")
        || combined.contains("server")
        || combined.contains("api")
        || combined.contains("cli")
        || combined.contains("crates/")
    {
        return DeepScanBucket::Backend;
    }

    DeepScanBucket::CodeQuality
}

fn normalize_findings(findings: &[Finding]) -> Vec<Finding> {
    findings
        .iter()
        .map(|f| {
            let mut normalized = f.clone();
            normalized.category = normalize_finding_category(&normalized.category);
            normalized
        })
        .collect()
}

fn normalize_finding_category(raw: &str) -> String {
    let s = raw.trim().to_lowercase();
    if s.is_empty() {
        return "Code Quality".to_string();
    }

    let mapped = if s.starts_with("sec") || s == "security" {
        "Security"
    } else if s.starts_with("bug") {
        "Bug"
    } else if s.starts_with("perf") || s == "performance" {
        "Performance"
    } else if s == "dx" || s.contains("developer") {
        "DX"
    } else if s == "ux" || s.contains("user") || s.contains("ui") {
        "UX"
    } else if s == "docs" || s == "documentation" || s == "doc" {
        "Docs"
    } else if s == "debt" || s.contains("technical debt") {
        "Debt"
    } else if s == "deps" || s.contains("dependency") {
        "Deps"
    } else {
        return raw.trim().to_string();
    };

    mapped.to_string()
}

fn severity_rank(severity: &Severity) -> u8 {
    match severity {
        Severity::Critical => 4,
        Severity::High => 3,
        Severity::Medium => 2,
        Severity::Low => 1,
        Severity::Info => 0,
    }
}

fn risk_rank(risk: &RiskLevel) -> u8 {
    match risk {
        RiskLevel::High => 3,
        RiskLevel::Medium => 2,
        RiskLevel::Low => 1,
    }
}

/// Results from Phase A (Ingestion)
#[derive(Debug, Clone)]
pub struct IngestionResult {
    /// Repository summary information
    pub repo_summary: RepoSummary,
    /// List of ingested files with content
    pub files: Vec<IngestedFile>,
    /// Local findings from initial scan
    pub local_findings: Vec<LocalFinding>,
    /// Summary of redactions performed
    pub redaction_summary: crate::models::RedactionSummary,
}

/// Results from Phase B (Analysis)
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// Findings from the analysis phase
    pub findings: Vec<Finding>,
    /// TODO items generated from analysis
    pub todos: Vec<TodoItem>,
    /// Whether the analysis is partial (e.g., due to LLM unavailability)
    pub is_partial: bool,
    /// Blockers that prevent complete analysis
    pub blockers: Vec<Blocker>,
}

/// Complete scan result
#[derive(Debug, Clone)]
pub struct ScanResult {
    /// Run manifest with metadata about the scan
    pub manifest: RunManifest,
    /// Generated HQE report with findings and recommendations
    pub report: HqeReport,
    /// Paths to generated artifacts
    pub artifacts: ArtifactPaths,
}

/// Paths to exported artifacts
#[derive(Debug, Clone)]
pub struct ArtifactPaths {
    /// Path to the markdown report
    pub report_md: std::path::PathBuf,
    /// Path to the JSON report
    pub report_json: std::path::PathBuf,
    /// Path to the manifest JSON file
    pub manifest_json: std::path::PathBuf,
}

impl ArtifactPaths {
    /// Empty artifact paths when export is handled externally.
    pub fn empty() -> Self {
        Self {
            report_md: PathBuf::new(),
            report_json: PathBuf::new(),
            manifest_json: PathBuf::new(),
        }
    }
}

/// Detect language from file extension
fn detect_language(path: &str) -> Option<String> {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())?;

    match ext {
        "rs" => Some("rust".to_string()),
        "ts" => Some("typescript".to_string()),
        "tsx" => Some("typescript".to_string()),
        "js" => Some("javascript".to_string()),
        "jsx" => Some("javascript".to_string()),
        "py" => Some("python".to_string()),
        "go" => Some("go".to_string()),
        "java" => Some("java".to_string()),
        "kt" => Some("kotlin".to_string()),
        "swift" => Some("swift".to_string()),
        "rb" => Some("ruby".to_string()),
        "php" => Some("php".to_string()),
        "c" => Some("c".to_string()),
        "cpp" | "cc" => Some("cpp".to_string()),
        "h" | "hpp" => Some("header".to_string()),
        "md" => Some("markdown".to_string()),
        "json" => Some("json".to_string()),
        "yaml" | "yml" => Some("yaml".to_string()),
        "toml" => Some("toml".to_string()),
        _ => Some(ext.to_string()),
    }
}

use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_scan_pipeline_local_only() -> anyhow::Result<()> {
        let temp = TempDir::new()?;
        tokio::fs::write(temp.path().join("package.json"), r#"{"name":"test"}"#).await?;
        tokio::fs::write(temp.path().join(".env"), "SECRET=123").await?;

        let config = ScanConfig {
            llm_enabled: false,
            provider_profile: None,
            limits: ScanLimits::default(),
            local_only: true,
            timeout_seconds: 30,
        };

        let mut pipeline = ScanPipeline::new(temp.path(), config)?;
        let result = pipeline.run().await?;

        assert!(!result.report.master_todo_backlog.is_empty());
        assert!(!result.report.executive_summary.blockers.is_empty());
        assert_eq!(result.manifest.protocol.protocol_version, "3.1.0");
        Ok(())
    }

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language("main.rs"), Some("rust".to_string()));
        assert_eq!(detect_language("app.ts"), Some("typescript".to_string()));
        assert_eq!(detect_language("script.py"), Some("python".to_string()));
        assert_eq!(detect_language("README.md"), Some("markdown".to_string()));
    }
}
