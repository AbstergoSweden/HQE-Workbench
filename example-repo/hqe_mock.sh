#!/bin/bash

# Mock HQE CLI for testing purposes
# This simulates the actual HQE Workbench CLI functionality

set -e

COMMAND="$1"
shift

case "$COMMAND" in
    "scan")
        echo "HQE Workbench - Repository Security Scan"
        echo "====================================="
        
        REPO_PATH=""
        PROVIDER=""
        MODEL=""
        API_KEY=""
        BASE_URL=""
        OUTPUT_DIR="./scan-results-$(date +%Y%m%d-%H%M%S)"
        VERBOSE=false
        
        # Parse arguments
        while [[ $# -gt 0 ]]; do
            case $1 in
                --repo)
                    REPO_PATH="$2"
                    shift 2
                    ;;
                --provider)
                    PROVIDER="$2"
                    shift 2
                    ;;
                --model)
                    MODEL="$2"
                    shift 2
                    ;;
                --api-key)
                    API_KEY="$2"
                    shift 2
                    ;;
                --base-url)
                    BASE_URL="$2"
                    shift 2
                    ;;
                --out)
                    OUTPUT_DIR="$2"
                    shift 2
                    ;;
                --timeout)
                    TIMEOUT="$2"
                    shift 2
                    ;;
                --verbose)
                    VERBOSE=true
                    shift
                    ;;
                *)
                    echo "Unknown option: $1"
                    exit 1
                    ;;
            esac
        done
        
        echo "Repository: ${REPO_PATH:-$(pwd | sed 's/\\/\\\\/g' | sed 's/"/\\"/g')}"
        echo "Provider: ${PROVIDER:-openai}"
        echo "Model: ${MODEL:-gpt-4}"
        echo "Base URL: ${BASE_URL:-https://api.openai.com/v1}"
        echo "Output: $OUTPUT_DIR"
        echo ""
        
        # Create output directory
        mkdir -p "$OUTPUT_DIR"
        
        # Simulate scanning process
        echo "🔍 Starting repository scan..."
        sleep 2
        
        echo "✓ Analyzing repository structure..."
        sleep 1
        
        echo "✓ Detecting technologies..."
        sleep 1
        
        echo "✓ Identifying security patterns..."
        sleep 2
        
        echo "✓ Running code quality checks..."
        sleep 1
        
        echo "✓ Generating report..."
        sleep 1
        
        # Create a mock report
        cat > "$OUTPUT_DIR/report.json" << EOF
{
  "executive_summary": {
    "health_score": 7.5,
    "top_priorities": [
      "Implement proper input validation in API endpoints",
      "Add authentication to sensitive endpoints",
      "Update dependencies with known vulnerabilities"
    ],
    "critical_findings": [
      "SQL injection vulnerability in user search endpoint",
      "Insecure direct object reference in file download functionality"
    ],
    "blockers": []
  },
  "project_map": {
    "architecture": {
      "languages": [
        {"name": "JavaScript", "percentage": 65},
        {"name": "HTML", "percentage": 20},
        {"name": "CSS", "percentage": 10},
        {"name": "JSON", "percentage": 5}
      ],
      "frameworks": ["Express.js", "React"],
      "runtimes": ["Node.js 18.x"]
    },
    "entrypoints": [
      {"file_path": "server.js", "type": "main", "description": "Application entrypoint"},
      {"file_path": "package.json", "type": "config", "description": "Dependency and script definitions"}
    ],
    "data_flow": "Client requests -> Express router -> Business logic -> Database",
    "tech_stack": {
      "detected": [
        {"name": "Express", "version": null, "evidence": "package.json"},
        {"name": "React", "version": null, "evidence": "package.json"},
        {"name": "Node.js", "version": "18.x", "evidence": "package.json engines field"}
      ],
      "package_managers": ["npm"]
    }
  },
  "pr_harvest": null,
  "deep_scan_results": {
    "security": [
      {
        "id": "SEC-001",
        "severity": "High",
        "risk": "High",
        "category": "Injection",
        "title": "SQL Injection in User Search",
        "evidence": {
          "type": "file_line",
          "file": "routes/users.js",
          "line": 42,
          "snippet": "db.query('SELECT * FROM users WHERE name = ' + userInput + '"
        },
        "impact": "Attacker can extract all user data or gain system access",
        "recommendation": "Use parameterized queries or prepared statements"
      },
      {
        "id": "SEC-002",
        "severity": "Medium",
        "risk": "Medium",
        "category": "Authentication",
        "title": "Weak Session Management",
        "evidence": {
          "type": "file_line",
          "file": "middleware/auth.js",
          "line": 28,
          "snippet": "session.cookie.maxAge = 365 * 24 * 60 * 60 * 1000; // 1 year"
        },
        "impact": "Prolonged unauthorized access if credentials are compromised",
        "recommendation": "Implement shorter session timeouts with refresh tokens"
      }
    ],
    "code_quality": [
      {
        "id": "CQ-001",
        "severity": "Low",
        "risk": "Low",
        "category": "Maintainability",
        "title": "Duplicated Code Blocks",
        "evidence": {
          "type": "file_function",
          "file": "utils/helpers.js",
          "function": "validateInput",
          "snippet": "// Similar validation logic found in 3 other locations"
        },
        "impact": "Increased maintenance burden",
        "recommendation": "Refactor into reusable function"
      }
    ],
    "frontend": [],
    "backend": [
      {
        "id": "BE-001",
        "severity": "Medium",
        "risk": "Medium",
        "category": "Performance",
        "title": "Inefficient Database Query",
        "evidence": {
          "type": "file_line",
          "file": "models/user.js",
          "line": 125,
          "snippet": "User.findAll({ where: { status: 'active' } }) // Missing index on status field"
        },
        "impact": "Slow response times for large datasets",
        "recommendation": "Add database index on status field or optimize query"
      }
    ],
    "testing": []
  },
  "master_todo_backlog": [
    {
      "id": "TODO-001",
      "severity": "High",
      "risk": "High",
      "category": "SEC",
      "title": "Fix SQL injection in user search",
      "root_cause": "Direct string concatenation in SQL query",
      "evidence": {
        "type": "file_line",
        "file": "routes/users.js",
        "line": 42,
        "snippet": "db.query('SELECT * FROM users WHERE name = ' + userInput + '"
      },
      "fix_approach": "Replace with parameterized query using database library",
      "verify": "Test with SQL injection payloads to ensure they're handled safely",
      "blocked_by": null
    },
    {
      "id": "TODO-002",
      "severity": "Medium",
      "risk": "Medium",
      "category": "SEC",
      "title": "Implement rate limiting",
      "root_cause": "No protection against brute force or DoS attacks",
      "evidence": {
        "type": "file_function",
        "file": "routes/auth.js",
        "function": "login",
        "snippet": "No rate limiting implemented"
      },
      "fix_approach": "Add rate limiting middleware to authentication endpoints",
      "verify": "Verify that requests are limited after threshold is exceeded",
      "blocked_by": null
    }
  ],
  "implementation_plan": {
    "immediate": ["Fix SQL injection vulnerability"],
    "short_term": ["Implement rate limiting", "Add input validation"],
    "medium_term": ["Refactor authentication system", "Add security headers"],
    "long_term": ["Migrate to more secure framework", "Implement zero-trust architecture"],
    "dependency_graph": {
      "Fix SQL injection vulnerability": [],
      "Implement rate limiting": [],
      "Add input validation": []
    },
    "risk_assessment": [
      {
        "item_id": "TODO-001",
        "mitigation": "Apply parameterized queries immediately"
      }
    ]
  },
  "immediate_actions": [],
  "session_log": {
    "completed": [
      "Repository structure analysis",
      "Technology stack detection",
      "Security pattern identification",
      "Code quality assessment"
    ],
    "in_progress": [],
    "discovered": [
      "2 critical security vulnerabilities",
      "Multiple code quality issues",
      "Outdated dependencies"
    ],
    "reprioritized": [],
    "next_session": [
      "Detailed vulnerability analysis",
      "Remediation plan development"
    ]
  }
}
EOF

        # Create a mock manifest
        cat > "$OUTPUT_DIR/manifest.json" << EOF
{
  "run_id": "scan-$(date +%Y%m%d-%H%M%S)-$(openssl rand -hex 4)",
  "repo": {
    "source": "local",
    "path": "${REPO_PATH:-$(pwd)}",
    "git_remote": null,
    "git_commit": "$(git rev-parse HEAD 2>/dev/null || echo 'n/a')"
  },
  "provider": {
    "name": "${PROVIDER:-openai}",
    "base_url": "${BASE_URL:-https://api.openai.com/v1}",
    "model": "${MODEL:-gpt-4}",
    "llm_enabled": true
  },
  "limits": {
    "max_files_sent": 40,
    "max_total_chars_sent": 250000,
    "snippet_chars": 4000
  },
  "timestamps": {
    "started": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "ended": null
  },
  "protocol": {
    "protocol_version": "3.1.0",
    "schema_version": "3.1.0"
  }
}
EOF

        # Create a human-readable report
        cat > "$OUTPUT_DIR/report.md" << EOF
# HQE Security Scan Report

**Run ID**: scan-$(date +%Y%m%d-%H%M%S)-$(openssl rand -hex 4)  
**Repository**: ${REPO_PATH:-$(pwd)}  
**Scan Date**: $(date)  
**Provider**: ${PROVIDER:-openai}  
**Model**: ${MODEL:-gpt-4}  

## Executive Summary

The security scan identified several areas requiring attention. The application has a health score of 7.5/10, with 2 critical findings that should be addressed immediately.

### Top Priorities
1. Implement proper input validation in API endpoints
2. Add authentication to sensitive endpoints
3. Update dependencies with known vulnerabilities

### Critical Findings
- SQL injection vulnerability in user search endpoint
- Insecure direct object reference in file download functionality

## Security Findings

### High Severity
1. **SQL Injection in User Search** (SEC-001)
   - File: routes/users.js, Line: 42
   - Risk: Attacker can extract all user data or gain system access
   - Recommendation: Use parameterized queries or prepared statements

2. **Weak Session Management** (SEC-002)
   - File: middleware/auth.js, Line: 28
   - Risk: Prolonged unauthorized access if credentials are compromised
   - Recommendation: Implement shorter session timeouts with refresh tokens

### Medium Severity
1. **Inefficient Database Query** (BE-001)
   - File: models/user.js, Line: 125
   - Risk: Slow response times for large datasets
   - Recommendation: Add database index on status field or optimize query

## Code Quality Issues

### Low Severity
1. **Duplicated Code Blocks** (CQ-001)
   - File: utils/helpers.js
   - Risk: Increased maintenance burden
   - Recommendation: Refactor into reusable function

## Master TODO Backlog

1. **[High] Fix SQL injection in user search** (TODO-001)
   - Apply parameterized queries immediately

2. **[Medium] Implement rate limiting** (TODO-002)
   - Add rate limiting middleware to authentication endpoints

## Implementation Plan

### Immediate Actions
- Fix SQL injection vulnerability

### Short-term Goals
- Implement rate limiting
- Add input validation

### Medium-term Goals
- Refactor authentication system
- Add security headers

## Session Log
- Completed: Repository structure analysis, Technology stack detection, Security pattern identification
- Discovered: 2 critical security vulnerabilities, Multiple code quality issues
- Next: Detailed vulnerability analysis, Remediation plan development
EOF

        echo ""
        echo "✅ Scan completed successfully!"
        echo "📊 Report generated: $OUTPUT_DIR/report.json"
        echo "📋 Manifest: $OUTPUT_DIR/manifest.json" 
        echo "📝 Human-readable report: $OUTPUT_DIR/report.md"
        ;;
    "version")
        echo "HQE Workbench CLI - Mock Version 1.0.0"
        ;;
    "help"|"-h"|"--help")
        echo "HQE Workbench CLI - Repository Security Scanner"
        echo ""
        echo "Usage: hqe [command] [options]"
        echo ""
        echo "Commands:"
        echo "  scan    Scan a repository for security and quality issues"
        echo "  version Show version information"
        echo "  help    Show this help message"
        echo ""
        echo "Scan Options:"
        echo "  --repo PATH          Path to repository to scan"
        echo "  --provider NAME      AI provider to use (default: openai)"
        echo "  --model NAME         Model to use (default: gpt-4)"
        echo "  --api-key KEY        API key for provider"
        echo "  --base-url URL       Base URL for provider API"
        echo "  --out DIR            Output directory for reports"
        echo "  --verbose            Enable verbose output"
        ;;
    "")
        echo "HQE Workbench CLI - Repository Security Scanner"
        echo "Run 'hqe help' for usage information"
        ;;
    *)
        echo "Unknown command: $COMMAND"
        echo "Run 'hqe help' for usage information"
        exit 1
        ;;
esac