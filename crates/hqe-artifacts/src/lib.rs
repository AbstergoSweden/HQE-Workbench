//! Artifact generation for HQE reports and manifests

#![warn(missing_docs)]

use hqe_core::models::*;
use hqe_core::scan::ScanResult;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, instrument};

/// Artifact writer handles saving reports and manifests to disk
pub struct ArtifactWriter {
    output_dir: PathBuf,
}

impl ArtifactWriter {
    /// Create a new artifact writer for the given output directory
    pub fn new(output_dir: impl AsRef<Path>) -> Self {
        Self {
            output_dir: output_dir.as_ref().to_path_buf(),
        }
    }

    /// Ensure output directory exists
    fn ensure_dir(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.output_dir)?;
        Ok(())
    }

    /// Write run manifest
    #[instrument(skip(self, manifest))]
    pub async fn write_manifest(&self, manifest: &RunManifest) -> anyhow::Result<PathBuf> {
        self.ensure_dir()?;
        let path = self.output_dir.join("run-manifest.json");
        let json = serde_json::to_string_pretty(manifest)?;
        tokio::fs::write(&path, json).await?;
        info!("Wrote manifest: {}", path.display());
        Ok(path)
    }

    /// Write report as JSON
    #[instrument(skip(self, report))]
    pub async fn write_report_json(&self, report: &HqeReport) -> anyhow::Result<PathBuf> {
        self.ensure_dir()?;
        let path = self.output_dir.join("report.json");
        let json = serde_json::to_string_pretty(report)?;
        tokio::fs::write(&path, json).await?;
        info!("Wrote report JSON: {}", path.display());
        Ok(path)
    }

    /// Write report as Markdown
    #[instrument(skip(self, report))]
    pub async fn write_report_md(&self, report: &HqeReport) -> anyhow::Result<PathBuf> {
        self.ensure_dir()?;
        let path = self.output_dir.join("report.md");
        let md = self.render_markdown(report)?;
        tokio::fs::write(&path, md).await?;
        info!("Wrote report Markdown: {}", path.display());
        Ok(path)
    }

    /// Write session log
    #[instrument(skip(self, session_log))]
    pub async fn write_session_log(&self, session_log: &SessionLog) -> anyhow::Result<PathBuf> {
        self.ensure_dir()?;
        let path = self.output_dir.join("session-log.json");
        let json = serde_json::to_string_pretty(session_log)?;
        tokio::fs::write(&path, json).await?;
        info!("Wrote session log: {}", path.display());
        Ok(path)
    }

    /// Write redaction log
    #[instrument(skip(self, summary))]
    pub async fn write_redaction_log(&self, summary: &RedactionSummary) -> anyhow::Result<PathBuf> {
        self.ensure_dir()?;
        let path = self.output_dir.join("redaction-log.json");

        #[derive(serde::Serialize)]
        struct RedactionLog {
            total_redactions: usize,
            by_type: HashMap<String, usize>,
            note: &'static str,
        }

        let log = RedactionLog {
            total_redactions: summary.total_redactions,
            by_type: summary.by_type.clone(),
            note: "Secret values removed before LLM transmission",
        };

        let json = serde_json::to_string_pretty(&log)?;
        tokio::fs::write(&path, json).await?;
        info!("Wrote redaction log: {}", path.display());
        Ok(path)
    }

    /// Write all artifacts (manifest, report JSON/MD, logs)
    pub async fn write_all(&self, result: &ScanResult) -> anyhow::Result<ArtifactPaths> {
        let manifest = self.write_manifest(&result.manifest).await?;
        let report_json = self.write_report_json(&result.report).await?;
        let report_md = self.write_report_md(&result.report).await?;
        self.write_session_log(&result.report.session_log).await?;

        Ok(ArtifactPaths {
            manifest_json: manifest,
            report_json,
            report_md,
        })
    }

    /// Render report as Markdown (HQE v3 format)
    fn render_markdown(&self, report: &HqeReport) -> anyhow::Result<String> {
        let mut md = String::new();

        // Header
        md.push_str("# HQE Engineer Report\n\n");
        md.push_str(&format!("Run ID: `{}`\n\n", report.run_id));

        // Section 1: Executive Summary
        md.push_str("## 1. Executive Summary\n\n");
        md.push_str(&format!(
            "**Health Score:** {}/10\n\n",
            report.executive_summary.health_score
        ));

        if !report.executive_summary.critical_findings.is_empty() {
            md.push_str("### Critical Findings\n\n");
            for finding in &report.executive_summary.critical_findings {
                md.push_str(&format!("- 🚨 {}\n", finding));
            }
            md.push('\n');
        }

        if !report.executive_summary.top_priorities.is_empty() {
            md.push_str("### Top Priorities\n\n");
            for priority in &report.executive_summary.top_priorities {
                md.push_str(&format!("- {}\n", priority));
            }
            md.push('\n');
        }

        if !report.executive_summary.blockers.is_empty() {
            md.push_str("### Blockers\n\n");
            for blocker in &report.executive_summary.blockers {
                md.push_str(&format!("- **{}**\n", blocker.description));
                md.push_str(&format!("  - Reason: {}\n", blocker.reason));
                md.push_str(&format!("  - How to obtain: {}\n", blocker.how_to_obtain));
            }
            md.push('\n');
        }

        // Section 2: Project Map
        md.push_str("## 2. Project Map\n\n");

        md.push_str("### Architecture\n\n");
        md.push_str(&format!(
            "**Languages:** {}\n\n",
            report.project_map.architecture.languages.join(", ")
        ));

        if !report.project_map.entrypoints.is_empty() {
            md.push_str("### Entrypoints\n\n");
            md.push_str("| File | Type | Description |\n");
            md.push_str("|------|------|-------------|\n");
            for ep in &report.project_map.entrypoints {
                md.push_str(&format!(
                    "| `{}` | {} | {} |\n",
                    ep.file_path, ep.entry_type, ep.description
                ));
            }
            md.push('\n');
        }

        if !report.project_map.tech_stack.detected.is_empty() {
            md.push_str("### Tech Stack\n\n");
            for tech in &report.project_map.tech_stack.detected {
                md.push_str(&format!(
                    "- **{}** (evidence: {})\n",
                    tech.name, tech.evidence
                ));
            }
            md.push('\n');
        }

        // Section 3: PR Harvest (if present)
        if let Some(pr_harvest) = &report.pr_harvest {
            md.push_str("## 3. PR Harvest\n\n");
            if !pr_harvest.inventory.is_empty() {
                md.push_str("| PR | Title | Status | Recommendation |\n");
                md.push_str("|----|-------|--------|----------------|\n");
                for pr in &pr_harvest.inventory {
                    md.push_str(&format!(
                        "| {} | {} | {} | {:?} |\n",
                        pr.pr_id, pr.title, pr.status, pr.recommendation
                    ));
                }
                md.push('\n');
            }
        }

        // Section 4: Deep Scan Results
        md.push_str("## 4. Deep Scan Results\n\n");

        if !report.deep_scan_results.security.is_empty() {
            md.push_str("### Security\n\n");
            self.render_findings(&mut md, &report.deep_scan_results.security);
        }

        if !report.deep_scan_results.code_quality.is_empty() {
            md.push_str("### Code Quality\n\n");
            self.render_findings(&mut md, &report.deep_scan_results.code_quality);
        }

        // Section 5: Master TODO Backlog
        md.push_str("## 5. Master TODO Backlog\n\n");
        md.push_str("| ID | Severity | Risk | Category | Title |\n");
        md.push_str("|----|----------|------|----------|-------|\n");
        for todo in &report.master_todo_backlog {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                todo.id, todo.severity, todo.risk, todo.category, todo.title
            ));
        }
        md.push('\n');

        // Section 6: Implementation Plan
        md.push_str("## 6. Implementation Plan\n\n");

        if !report.implementation_plan.immediate.is_empty() {
            md.push_str("### Immediate (Do Now)\n\n");
            for item in &report.implementation_plan.immediate {
                md.push_str(&format!("- [ ] {}\n", item));
            }
            md.push('\n');
        }

        if !report.implementation_plan.short_term.is_empty() {
            md.push_str("### Short-term (This Week)\n\n");
            for item in &report.implementation_plan.short_term {
                md.push_str(&format!("- [ ] {}\n", item));
            }
            md.push('\n');
        }

        // Section 7: Immediate Actions
        md.push_str("## 7. Immediate Actions\n\n");
        if report.immediate_actions.is_empty() {
            md.push_str("No immediate actions generated.\n\n");
        } else {
            for action in &report.immediate_actions {
                md.push_str(&format!("### {}: {}\n\n", action.todo_id, action.title));
                md.push_str(&format!("**Problem:** {}\n\n", action.problem));
                md.push_str(&format!("**Risk:** {}\n\n", action.risk));
                if action.behavior_change {
                    md.push_str("⚠️ **BEHAVIOR CHANGE**\n\n");
                }
                for diff in &action.diffs {
                    md.push_str(&format!("#### File: `{}`\n\n", diff.file_path));
                    md.push_str("```diff\n");
                    md.push_str(&diff.diff_content);
                    md.push_str("\n```\n\n");
                }
                md.push_str("**Verification:**\n");
                for step in &action.verification {
                    md.push_str(&format!("1. Run: `{}`\n", step.command));
                    md.push_str(&format!("   Expected: {}\n", step.expected_output));
                }
                md.push('\n');
            }
        }

        // Section 8: Session Log
        md.push_str("## 8. Session Log\n\n");

        if !report.session_log.completed.is_empty() {
            md.push_str("### Completed\n\n");
            for item in &report.session_log.completed {
                md.push_str(&format!("- ✅ {}\n", item));
            }
            md.push('\n');
        }

        if !report.session_log.in_progress.is_empty() {
            md.push_str("### In Progress\n\n");
            for item in &report.session_log.in_progress {
                md.push_str(&format!("- 🔄 {}\n", item));
            }
            md.push('\n');
        }

        if !report.session_log.discovered.is_empty() {
            md.push_str("### Discovered\n\n");
            for item in &report.session_log.discovered {
                md.push_str(&format!("- 🆕 {}\n", item));
            }
            md.push('\n');
        }

        Ok(md)
    }

    fn render_findings(&self, md: &mut String, findings: &[Finding]) {
        for finding in findings {
            md.push_str(&format!("#### {}: {}\n\n", finding.id, finding.title));
            md.push_str(&format!("- **Severity:** {}\n", finding.severity));
            md.push_str(&format!("- **Risk:** {}\n", finding.risk));
            md.push_str(&format!("- **Impact:** {}\n", finding.impact));
            md.push_str(&format!(
                "- **Recommendation:** {}\n",
                finding.recommendation
            ));
            md.push('\n');
        }
    }
}

/// Paths to generated artifacts
#[derive(Debug, Clone)]
pub struct ArtifactPaths {
    /// Path to manifest.json
    pub manifest_json: PathBuf,
    /// Path to report.json
    pub report_json: PathBuf,
    /// Path to report.md
    pub report_md: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_report() -> HqeReport {
        HqeReport {
            run_id: "test-123".to_string(),
            executive_summary: ExecutiveSummary {
                health_score: 7,
                top_priorities: vec!["Fix security issues".to_string()],
                critical_findings: vec![],
                blockers: vec![],
            },
            project_map: ProjectMap {
                architecture: Architecture {
                    languages: vec!["Rust".to_string()],
                    frameworks: vec![],
                    runtimes: vec![],
                    ..Default::default()
                },
                entrypoints: vec![],
                data_flow: None,
                tech_stack: TechStack::default(),
            },
            pr_harvest: None,
            deep_scan_results: DeepScanResults::default(),
            master_todo_backlog: vec![],
            implementation_plan: ImplementationPlan::default(),
            immediate_actions: vec![],
            session_log: SessionLog::default(),
        }
    }

    #[tokio::test]
    async fn test_write_manifest() -> anyhow::Result<()> {
        let temp = TempDir::new()?;
        let writer = ArtifactWriter::new(temp.path());

        let manifest = RunManifest::new("/test", "local");
        let path = writer.write_manifest(&manifest).await?;

        assert!(path.exists());
        let content = tokio::fs::read_to_string(&path).await?;
        assert!(content.contains("run_id"));
        Ok(())
    }

    #[tokio::test]
    async fn test_write_report_md() -> anyhow::Result<()> {
        let temp = TempDir::new()?;
        let writer = ArtifactWriter::new(temp.path());

        let report = create_test_report();
        let path = writer.write_report_md(&report).await?;

        assert!(path.exists());
        let content = tokio::fs::read_to_string(&path).await?;
        assert!(content.contains("HQE Engineer Report"));
        assert!(content.contains("Executive Summary"));
        assert!(content.contains("Project Map"));
        Ok(())
    }
}
