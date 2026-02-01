#!/bin/bash

# Venice AI API Test Script for HQE Workbench
# This script runs the HQE Workbench on the example repository using the Venice AI API

set -e  # Exit immediately if a command exits with a non-zero status

echo "==========================================="
echo "HQE Workbench - Venice AI API Test Script"
echo "==========================================="

# Check if required environment variables are set
if [ -z "$VENICE_API_KEY" ]; then
    echo "Error: VENICE_API_KEY environment variable is not set"
    echo "Please set your Venice API key before running this script"
    exit 1
fi

# Set default values if not provided
VENICE_API_BASE_URL="${VENICE_API_BASE_URL:-https://api.venice.ai/v1}"
VENICE_MODEL_NAME="${VENICE_MODEL_NAME:-venice-medium}"
VENICE_REQUEST_TIMEOUT="${VENICE_REQUEST_TIMEOUT:-60}"

echo "Using API endpoint: $VENICE_API_BASE_URL"
echo "Using model: $VENICE_MODEL_NAME"
echo "Repository to scan: $(pwd)"

# Create a temporary directory for the scan results
OUTPUT_DIR="./scan-results-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$OUTPUT_DIR"

echo "Output directory: $OUTPUT_DIR"

# Run HQE Workbench scan with Venice AI
echo ""
echo "Starting HQE Workbench scan with Venice AI..."
echo ""

# Use the hqe CLI with Venice API configuration
if [ -f "./hqe_mock_refactored.sh" ]; then
    ./hqe_mock_refactored.sh scan \
        --repo "$(pwd)" \
        --provider venice \
        --model "$VENICE_MODEL_NAME" \
        --base-url "$VENICE_API_BASE_URL" \
        --api-key "$VENICE_API_KEY" \
        --timeout "$VENICE_REQUEST_TIMEOUT" \
        --out "$OUTPUT_DIR" \
        --verbose
else
    echo "Error: hqe_mock_refactored.sh not found"
    echo "Please ensure the refactored mock HQE CLI script is available"
    exit 1
fi

echo ""
echo "Scan completed successfully!"
echo "Results are available in: $OUTPUT_DIR"

# Show summary of findings
echo ""
echo "Scan Summary:"
if [ -f "$OUTPUT_DIR/report.json" ]; then
    echo "Report: $OUTPUT_DIR/report.json"
    echo "Manifest: $OUTPUT_DIR/manifest.json"
    
    # Display basic statistics from the report
    if command -v jq &> /dev/null; then
        echo ""
        echo "Report Statistics:"
        jq -r '
        "  Executive Summary:" +
        "\n    Health Score: \( .executive_summary.health_score )" +
        "\n    Top Priorities: \( .executive_summary.top_priorities | length ) items" +
        "\n    Critical Findings: \( .executive_summary.critical_findings | length ) items" +
        "\n  Project Map:" +
        "\n    Languages: \( .project_map.architecture.languages | length ) languages detected" +
        "\n    Entrypoints: \( .project_map.entrypoints | length ) entrypoints" +
        "\n  Deep Scan Results:" +
        "\n    Security Issues: \( .deep_scan_results.security | length )" +
        "\n    Code Quality Issues: \( .deep_scan_results.code_quality | length )" +
        "\n  TODO Backlog:" +
        "\n    Items: \( .master_todo_backlog | length )"
        ' "$OUTPUT_DIR/report.json" 2>/dev/null || echo "  Could not parse report.json"
    fi
else
    echo "No report.json found in output directory"
fi

echo ""
echo "==========================================="
echo "HQE Workbench scan completed"
echo "==========================================="