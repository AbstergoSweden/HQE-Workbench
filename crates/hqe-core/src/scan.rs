//! HQE Scan pipeline

use crate::models::*;
use crate::redaction::RedactionEngine;
use crate::repo::RepoScanner;
use std::path::Path;
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

/// Pipeline for running an HQE scan
pub struct ScanPipeline {
    config: ScanConfig,
    redaction: RedactionEngine,
    manifest: RunManifest,
    phase: ScanPhase,
}

impl ScanPipeline {
    /// Creates a new ScanPipeline for the given repository path and configuration
    pub fn new(repo_path: impl AsRef<Path>, config: ScanConfig) -> crate::Result<Self> {
        let provider_name = config
            .provider_profile
            .clone()
            .unwrap_or_else(|| "local".to_string());
        let manifest = RunManifest::new(
            repo_path.as_ref().to_string_lossy().to_string(),
            provider_name,
        );

        Ok(Self {
            config,
            redaction: RedactionEngine::new(),
            manifest,
            phase: ScanPhase::Ingestion,
        })
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
        let analysis = if self.config.local_only {
            self.run_local_analysis(&ingestion).await?
        } else {
            // This would call the LLM provider in real implementation
            // For now, fall back to local-only
            warn!("LLM mode not fully implemented, using local analysis");
            self.run_local_analysis(&ingestion).await?
        };

        // Phase C: Report Generation
        self.phase = ScanPhase::ReportGeneration;
        info!("Phase: {}", self.phase);
        let report = self.generate_report(&ingestion, &analysis).await?;

        // Phase D: Artifact Export
        self.phase = ScanPhase::ArtifactExport;
        info!("Phase: {}", self.phase);
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
            is_partial: true,
            blockers: vec![Blocker {
                description: "LLM analysis disabled - Local mode only".to_string(),
                reason: "Local-only mode provides static analysis without LLM insights".to_string(),
                how_to_obtain: "Configure LLM provider in settings for AI-powered analysis and patch generation".to_string(),
            }],
        })
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
        let executive_summary = ExecutiveSummary {
            health_score,
            top_priorities: analysis
                .findings
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
        let deep_scan_results = DeepScanResults {
            security: analysis
                .findings
                .iter()
                .filter(|f| f.category == "Security")
                .cloned()
                .collect(),
            code_quality: vec![],
            frontend: vec![],
            backend: vec![],
            testing: vec![],
        };

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
        let session_log = SessionLog {
            completed: vec!["Ingestion".to_string(), "Local Analysis".to_string()],
            in_progress: if analysis.is_partial {
                vec!["Waiting for LLM analysis".to_string()]
            } else {
                vec![]
            },
            discovered: analysis.findings.iter().map(|f| f.id.clone()).collect(),
            reprioritized: vec![],
            next_session: if analysis.is_partial {
                vec!["Enable LLM provider for full analysis".to_string()]
            } else {
                vec![]
            },
        };

        Ok(HqeReport {
            run_id: self.manifest.run_id.clone(),
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
        // Placeholder - actual export happens in hqe-artifacts crate
        Ok(ArtifactPaths {
            report_md: PathBuf::from(format!("hqe_run_{}/report.md", self.manifest.run_id)),
            report_json: PathBuf::from(format!("hqe_run_{}/report.json", self.manifest.run_id)),
            manifest_json: PathBuf::from(format!(
                "hqe_run_{}/run-manifest.json",
                self.manifest.run_id
            )),
        })
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
