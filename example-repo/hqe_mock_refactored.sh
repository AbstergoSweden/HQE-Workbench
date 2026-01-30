#!/bin/bash

# Mock HQE CLI for testing purposes
# This simulates the actual HQE Workbench CLI functionality

set -e

# Function to display help information
show_help() {
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
    echo "  --timeout SECONDS    Request timeout (default: 60)"
    echo "  --verbose            Enable verbose output"
}

# Function to validate repository path
validate_repo_path() {
    local path="$1"
    
    # Check for path traversal attempts
    if [[ "$path" == *"../*" || "$path" == *"../"* || "$path" == *"../" ]]; then
        echo "Error: Path contains parent directory references ('..')" >&2
        return 1
    fi
    
    # Resolve the path to ensure it's absolute and within allowed boundaries
    local canonical_path
    canonical_path=$(realpath "$path" 2>/dev/null) || {
        echo "Error: Failed to resolve path: $path" >&2
        return 1
    }
    
    # Get the home directory to establish a reasonable boundary
    local home_dir
    home_dir=$(eval echo ~$USER)
    local canonical_home
    canonical_home=$(realpath "$home_dir" 2>/dev/null) || {
        echo "Error: Failed to resolve home directory" >&2
        return 1
    }
    
    if [[ ! "$canonical_path" =~ ^"$canonical_home"(/|$) ]] && [[ ! "$canonical_path" =~ ^/tmp(/|$) ]] && [[ ! "$canonical_path" =~ ^/Users(/|$) ]]; then
        echo "Error: Repository path must be within home directory, user directory, or temporary directory" >&2
        return 1
    fi
    
    return 0
}

# Function to parse command line arguments for the scan command
parse_scan_arguments() {
    local repo_path=""
    local provider="openai"
    local model="gpt-4"
    local api_key=""
    local base_url="https://api.openai.com/v1"
    local output_dir="./scan-results-$(date +%Y%m%d-%H%M%S)"
    local timeout="60"
    local verbose=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --repo)
                repo_path="$2"
                shift 2
                ;;
            --provider)
                provider="$2"
                shift 2
                ;;
            --model)
                model="$2"
                shift 2
                ;;
            --api-key)
                api_key="$2"
                shift 2
                ;;
            --base-url)
                base_url="$2"
                shift 2
                ;;
            --out)
                output_dir="$2"
                shift 2
                ;;
            --timeout)
                timeout="$2"
                shift 2
                ;;
            --verbose)
                verbose=true
                shift
                ;;
            *)
                echo "Unknown option: $1"
                echo "Run 'hqe help' for usage information"
                exit 1
                ;;
        esac
    done
    
    # Validate repository path
    if [[ -n "$repo_path" ]]; then
        if ! validate_repo_path "$repo_path"; then
            exit 1
        fi
    fi
    
    # Call the run_scan function with all parameters
    run_scan "$repo_path" "$provider" "$model" "$api_key" "$base_url" "$output_dir" "$timeout" "$verbose"
}

# Function to run the actual scan simulation
run_scan() {
    local repo_path="$1"
    local provider="$2"
    local model="$3"
    local api_key="$4"
    local base_url="$5"
    local output_dir="$6"
    local timeout="$7"
    local verbose="$8"
    
    echo "HQE Workbench - Repository Security Scan"
    echo "====================================="
    
    echo "Repository: ${repo_path:-$(pwd)}"
    echo "Provider: $provider"
    echo "Model: $model"
    echo "Base URL: $base_url"
    echo "Output: $output_dir"
    echo ""
    
    # Create output directory
    mkdir -p "$output_dir"
    
    # Simulate scanning process
    echo "🔍 Starting repository scan..."
    sleep 1
    
    echo "✓ Analyzing repository structure..."
    sleep 1
    
    echo "✓ Detecting technologies..."
    sleep 1
    
    echo "✓ Identifying security patterns..."
    sleep 1
    
    echo "✓ Running code quality checks..."
    sleep 1
    
    echo "✓ Generating report..."
    sleep 1
    
    # Generate reports
    generate_json_report "$output_dir" "$repo_path" "$provider" "$model" "$base_url"
    generate_manifest "$output_dir" "$repo_path" "$provider" "$model" "$base_url"
    generate_human_readable_report "$output_dir" "$repo_path" "$provider" "$model"
    
    echo ""
    echo "✅ Scan completed successfully!"
    echo "📊 Report generated: $output_dir/report.json"
    echo "📋 Manifest: $output_dir/manifest.json" 
    echo "📝 Human-readable report: $output_dir/report.md"
}

# Function to generate the JSON report
generate_json_report() {
    local output_dir="$1"
    local repo_path="$2"
    local provider="$3"
    local model="$4"
    local base_url="$5"
    
    cat > "$output_dir/report.json" << EOF
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
}

# Function to generate the manifest file
generate_manifest() {
    local output_dir="$1"
    local repo_path="$2"
    local provider="$3"
    local model="$4"
    local base_url="$5"
    
    cat > "$output_dir/manifest.json" << EOF
{
  "run_id": "scan-$(date +%Y%m%d-%H%M%S)-$(openssl rand -hex 4)",
  "repo": {
    "source": "local",
    "path": "${repo_path:-$(pwd)}",
    "git_remote": null,
    "git_commit": "$(git rev-parse HEAD 2>/dev/null || echo 'n/a')"
  },
  "provider": {
    "name": "$provider",
    "base_url": "$base_url",
    "model": "$model",
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
}

# Function to generate the human-readable report
generate_human_readable_report() {
    local output_dir="$1"
    local repo_path="$2"
    local provider="$3"
    local model="$4"
    
    cat > "$output_dir/report.md" << EOF
# HQE Security Scan Report

**Run ID**: scan-$(date +%Y%m%d-%H%M%S)-$(openssl rand -hex 4)  
**Repository**: ${repo_path:-$(pwd)}  
**Scan Date**: $(date)  
**Provider**: $provider  
**Model**: $model  

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
}

# Main function to parse the command
main() {
    local command="$1"
    shift
    
    case "$command" in
        "scan")
            parse_scan_arguments "$@"
            ;;
        "version")
            echo "HQE Workbench CLI - Mock Version 1.0.0"
            ;;
        "help"|"-h"|"--help")
            show_help
            ;;
        "")
            show_help
            ;;
        *)
            echo "Unknown command: $command"
            echo "Run 'hqe help' for usage information"
            exit 1
            ;;
    esac
}

# Call the main function with all arguments
main "$@"