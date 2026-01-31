//! Prompt templates for HQE Engineer Protocol

use hqe_core::models::EvidenceBundle;

/// System prompt for HQE Engineer v3
pub const HQE_SYSTEM_PROMPT: &str = r#"You are an HQE Engineer following the HQE Engineer v3 protocol.

Your responsibilities:
- Analyze codebases for health, quality, and security issues
- Produce reports in the STRICT output order specified
- NEVER fabricate findings - only cite evidence you can see
- Use evidence anchors: "path:line" when available, or "path + function + snippet"
- If information is missing, produce partial output + BLOCKERS + instrumentation steps

Output order (STRICT):
1. Executive Summary - health score (1-10), top 3 priorities, critical findings
2. Project Map - architecture, entrypoints, data flow, tech stack
3. PR Harvest - (if PRs provided)
4. Deep Scan Results - by category with severity (Critical/High/Medium/Low/Info) and risk (Low/Medium/High)
5. Master TODO Backlog - comprehensive, prioritized, actionable
6. Implementation Plan - phased roadmap
7. Immediate Actions - patch-packaged diffs (if applicable)
8. Session Log - completed/in-progress/discovered/reprioritized

Hard constraints:
- NO fabrication - use only provided context
- NO stalling - deliver partial value if context missing
- NO false verification claims
- Evidence required for every finding

ID prefixes (enforced):
- BOOT-### - Boot/startup reliability
- SEC-### - Security
- BUG-### - Functional bugs
- PERF-### - Performance
- UX-### - User experience
- DX-### - Developer experience
- DOC-### - Documentation
- DEBT-### - Technical debt
- DEPS-### - Dependencies
"#;

/// Build a JSON-only analysis prompt for structured findings/todos.
pub fn build_analysis_json_prompt(bundle: &EvidenceBundle) -> String {
    let mut prompt = String::new();

    prompt.push_str("# HQE JSON Analysis Request\n\n");
    prompt.push_str("Return ONLY a JSON object with this shape:\n");
    prompt.push_str(
        r#"{
  "findings": [
    {
      "id": "SEC-001",
      "severity": "high|medium|low|critical|info",
      "risk": "high|medium|low",
      "category": "Security|Bug|Perf|DX|UX|Docs|Debt|Deps",
      "title": "Short title",
      "evidence": {"type":"file_line","file":"path","line":1,"snippet":"..."},
      "impact": "Why it matters",
      "recommendation": "What to do"
    }
  ],
  "todos": [
    {
      "id": "SEC-001",
      "severity": "high|medium|low|critical|info",
      "risk": "high|medium|low",
      "category": "BOOT|SEC|BUG|PERF|DX|UX|DOC|DEBT|DEPS",
      "title": "Actionable fix",
      "root_cause": "Root cause",
      "evidence": {"type":"file_line","file":"path","line":1,"snippet":"..."},
      "fix_approach": "How to fix",
      "verify": "How to verify",
      "blocked_by": null
    }
  ],
  "blockers": [
    {"description":"...","reason":"...","how_to_obtain":"..."}
  ],
  "is_partial": false
}"#,
    );
    prompt.push_str("\n\nRules:\n");
    prompt.push_str("- Output JSON only. No markdown or extra text.\n");
    prompt.push_str("- If evidence is missing, add a blocker and set is_partial=true.\n");
    prompt
        .push_str("- Prefer FileLine evidence; use FileFunction or Reproduction only if needed.\n");

    prompt.push_str("\n## Repository Summary\n");
    prompt.push_str(&format!("Name: {}\n", bundle.repo_summary.name));
    if let Some(commit) = &bundle.repo_summary.commit_hash {
        prompt.push_str(&format!("Commit: {}\n", commit));
    }
    prompt.push_str("\n## Directory Tree\n");
    prompt.push_str(&bundle.repo_summary.directory_tree);
    prompt.push('\n');

    if !bundle.repo_summary.tech_stack.detected.is_empty() {
        prompt.push_str("\n## Detected Technologies\n");
        for tech in &bundle.repo_summary.tech_stack.detected {
            prompt.push_str(&format!("- {} (evidence: {})\n", tech.name, tech.evidence));
        }
    }

    if !bundle.repo_summary.entrypoints.is_empty() {
        prompt.push_str("\n## Entrypoints\n");
        for ep in &bundle.repo_summary.entrypoints {
            prompt.push_str(&format!(
                "- {} ({}): {}\n",
                ep.file_path, ep.entry_type, ep.description
            ));
        }
    }

    if !bundle.files.is_empty() {
        prompt.push_str("\n## File Snippets\n");
        for file in &bundle.files {
            prompt.push_str(&format!("--- file: {}\n", file.path));
            prompt.push_str(&format!("```\n{}\n```\n\n", file.content));
        }
    }

    if !bundle.local_findings.is_empty() {
        prompt.push_str("\n## Local Findings\n");
        for finding in &bundle.local_findings {
            prompt.push_str(&format!(
                "- [{}] {}: {}\n",
                finding.severity, finding.finding_type, finding.description
            ));
        }
    }

    prompt
}

/// Build the user prompt for a scan
pub fn build_scan_prompt(bundle: &EvidenceBundle) -> String {
    let mut prompt = String::new();

    prompt.push_str("# HQE Scan Request\n\n");
    prompt.push_str(&format!("Repository: {}\n", bundle.repo_summary.name));

    if let Some(commit) = &bundle.repo_summary.commit_hash {
        prompt.push_str(&format!("Commit: {}\n", commit));
    }

    prompt.push_str("\n## Constraints\n\n");
    prompt.push_str("Cite evidence as \"path:line\" when available; ");
    prompt.push_str("otherwise \"path + function + snippet\".\n");
    prompt
        .push_str("If info missing, produce partial output + BLOCKERS + instrumentation steps.\n");

    prompt.push_str("\n## Repository Tree\n\n");
    prompt.push_str(&bundle.repo_summary.directory_tree);
    prompt.push('\n');

    if !bundle.repo_summary.tech_stack.detected.is_empty() {
        prompt.push_str("\n## Detected Technologies\n\n");
        for tech in &bundle.repo_summary.tech_stack.detected {
            prompt.push_str(&format!("- {} (evidence: {})\n", tech.name, tech.evidence));
        }
    }

    if !bundle.repo_summary.entrypoints.is_empty() {
        prompt.push_str("\n## Entrypoints Detected\n\n");
        for ep in &bundle.repo_summary.entrypoints {
            prompt.push_str(&format!(
                "- {} ({}): {}\n",
                ep.file_path, ep.entry_type, ep.description
            ));
        }
    }

    if !bundle.files.is_empty() {
        prompt.push_str("\n## Key File Snippets\n\n");
        for file in &bundle.files {
            prompt.push_str(&format!("--- file: {}\n", file.path));
            prompt.push_str(&format!("```\n{}\n```\n\n", file.content));
        }
    }

    if !bundle.local_findings.is_empty() {
        prompt.push_str("\n## Local Findings (from heuristics)\n\n");
        for finding in &bundle.local_findings {
            prompt.push_str(&format!(
                "- [{}] {}: {}\n",
                finding.severity, finding.finding_type, finding.description
            ));
        }
    }

    prompt.push_str("\n---\n\n");
    prompt.push_str("Produce the HQE Engineer v3 report in the STRICT section order. ");
    prompt.push_str("Every finding MUST include evidence. ");
    prompt.push_str("If uncertain, flag with confidence level and verification steps.\n");

    prompt
}

/// Build prompt for patch generation
pub fn build_patch_prompt(
    todo_id: &str,
    title: &str,
    root_cause: &str,
    evidence: &str,
    file_context: &str,
) -> String {
    format!(
        r#"Generate a safe, minimal patch for the following TODO item.

TODO-ID: {todo_id}
Title: {title}
Root Cause: {root_cause}
Evidence: {evidence}

File Context:
{file_context}

Change Budget Rules:
- Limit to <= 5 files changed per TODO-ID
- No formatting-only changes
- Never fix by deleting features
- Flag behavior changes with warning

Generate a unified diff that:
1. Fixes the root cause
2. Includes verification steps
3. Has rollback instructions

Output format:
```diff
--- a/path/to/file
+++ b/path/to/file
@@ -line,lines +line,lines @@
- old line
+ new line
```

Verification:
1. Run: <command>
   Expected: <output>

Rollback: <command or git revert>
"#
    )
}

/// Build prompt for test generation
pub fn build_test_prompt(
    function_name: &str,
    file_path: &str,
    function_code: &str,
    test_framework: &str,
) -> String {
    format!(
        r#"Generate comprehensive tests for the following function.

Function: {function_name}
File: {file_path}
Test Framework: {test_framework}

Code:
```
{function_code}
```

Requirements:
1. Test happy path
2. Test error cases
3. Test edge cases
4. Include descriptive test names
5. Follow {test_framework} conventions

Output only the test code, no explanations.
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use hqe_core::models::*;

    fn create_test_bundle() -> EvidenceBundle {
        EvidenceBundle {
            repo_summary: RepoSummary {
                name: "test-repo".to_string(),
                commit_hash: Some("abc123".to_string()),
                directory_tree: "src/\n  main.rs\n".to_string(),
                tech_stack: TechStack {
                    detected: vec![DetectedTechnology {
                        name: "Rust".to_string(),
                        version: None,
                        evidence: "Cargo.toml".to_string(),
                    }],
                    package_managers: vec!["cargo".to_string()],
                },
                entrypoints: vec![Entrypoint {
                    file_path: "src/main.rs".to_string(),
                    entry_type: "main".to_string(),
                    description: "Application entrypoint".to_string(),
                }],
            },
            files: vec![FileSnippet {
                path: "src/main.rs".to_string(),
                content: "fn main() { println!(\"Hello\"); }".to_string(),
                start_line: Some(1),
                end_line: Some(1),
            }],
            local_findings: vec![],
        }
    }

    #[test]
    fn test_build_scan_prompt() {
        let bundle = create_test_bundle();
        let prompt = build_scan_prompt(&bundle);

        assert!(prompt.contains("HQE Scan Request"));
        assert!(prompt.contains("test-repo"));
        assert!(prompt.contains("abc123"));
        assert!(prompt.contains("src/main.rs"));
    }

    #[test]
    fn test_build_patch_prompt() {
        let prompt = build_patch_prompt(
            "BUG-001",
            "Fix null pointer",
            "Missing null check",
            "src/main.rs:42",
            "fn risky() { let x = null; }",
        );

        assert!(prompt.contains("BUG-001"));
        assert!(prompt.contains("Fix null pointer"));
        assert!(prompt.contains("unified diff"));
    }

    #[test]
    fn test_system_prompt_content() {
        assert!(HQE_SYSTEM_PROMPT.contains("STRICT output order"));
        assert!(HQE_SYSTEM_PROMPT.contains("NO fabrication"));
        assert!(HQE_SYSTEM_PROMPT.contains("BOOT-###"));
    }
}
