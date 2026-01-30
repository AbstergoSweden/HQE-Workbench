# Artifact Format Specification

## Output Directory Structure

```
hqe_run_2026-01-27T16-40-12Z_abcd1234/
├── run-manifest.json     # Metadata about the scan
├── report.md             # Human-readable report
├── report.json           # Structured report data
├── session-log.json      # Session tracking
└── redaction-log.json    # Secret redaction summary
```

## run-manifest.json

```json
{
  "run_id": "2026-01-27T16-40-12Z_abcd1234",
  "repo": {
    "source": "local",
    "path": "/Users/user/dev/myproject",
    "git_remote": "https://github.com/example/myproject.git",
    "git_commit": "a1b2c3d4"
  },
  "provider": {
    "name": "openai",
    "base_url": "https://api.openai.com/v1",
    "model": "gpt-4o-mini",
    "llm_enabled": true
  },
  "limits": {
    "max_files_sent": 40,
    "max_total_chars_sent": 250000,
    "snippet_chars": 4000
  },
  "timestamps": {
    "started": "2026-01-27T16:40:12Z",
    "ended": "2026-01-27T16:43:01Z"
  },
  "protocol": {
    "protocol_version": "3.1.0",
    "schema_version": "3.1.0"
  }
}
```

## report.json

```json
{
  "run_id": "2026-01-27T16-40-12Z_abcd1234",
  "executive_summary": {
    "health_score": 7,
    "top_priorities": ["Fix SEC-001", "Update DOC-001"],
    "critical_findings": [],
    "blockers": []
  },
  "project_map": {
    "architecture": {
      "languages": ["Rust"],
      "frameworks": ["Tauri"],
      "runtimes": []
    },
    "entrypoints": [
      {
        "file_path": "src/main.rs",
        "entry_type": "main",
        "description": "Application entrypoint"
      }
    ]
  },
  "master_todo_backlog": [
    {
      "id": "SEC-001",
      "severity": "High",
      "risk": "Medium",
      "category": "Sec",
      "title": "Potential security issue",
      "root_cause": ".env file not gitignored",
      "fix_approach": "Add to .gitignore",
      "verify": "git status shows .env as untracked"
    }
  ],
  "session_log": {
    "completed": ["Ingestion", "Analysis"],
    "in_progress": [],
    "discovered": ["SEC-001"],
    "reprioritized": [],
    "next_session": []
  }
}
```

## report.md

HQE v3 format with 8 sections:

1. Executive Summary
2. Project Map
3. PR Harvest (conditional)
4. Deep Scan Results
5. Master TODO Backlog
6. Implementation Plan
7. Immediate Actions
8. Session Log

## redaction-log.json

```json
{
  "total_redactions": 3,
  "by_type": {
    "AWS_ACCESS_KEY": 1,
    "GITHUB_TOKEN": 1,
    "SLACK_TOKEN": 1
  },
  "note": "Secret values removed before LLM transmission"
}
```

## session-log.json

```json
{
  "completed": ["Ingestion", "Local Analysis"],
  "in_progress": ["Waiting for LLM"],
  "discovered": ["SEC-001", "DOC-002"],
  "reprioritized": [],
  "next_session": ["Enable LLM for full analysis"]
}
```
