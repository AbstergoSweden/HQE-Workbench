#!/usr/bin/env python3
"""
Venice AI API Test Script for HQE Workbench

This script runs the HQE Workbench on the example repository using the Venice AI API.
"""

import os
import sys
import subprocess
import argparse
import json
from pathlib import Path
from datetime import datetime

def load_venice_config(config_path):
    """Load Venice API configuration from file."""
    config = {}
    if os.path.exists(config_path):
        with open(config_path, 'r') as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith('#'):
                    if '=' in line:
                        key, value = line.split('=', 1)
                        config[key.strip()] = value.strip()
    return config

def validate_environment():
    """Validate that required environment and tools are available."""
    # Check for required environment variables
    if not os.environ.get('VENICE_API_KEY'):
        print("Error: VENICE_API_KEY environment variable is not set")
        print("Please set your Venice API key before running this script")
        return False

    # Check for hqe_mock_refactored.sh script
    if not os.path.isfile('./hqe_mock_refactored.sh'):
        print("Error: hqe_mock_refactored.sh script not found")
        print("Please ensure the refactored mock HQE CLI script is available")
        return False

    return True

def run_hqe_scan(repo_path, output_dir, config):
    """Run HQE Workbench scan with Venice API."""
    cmd = [
        './hqe_mock_refactored.sh', 'scan',
        '--repo', repo_path,
        '--provider', 'venice',
        '--model', config.get('VENICE_MODEL_NAME', 'venice-medium'),
        '--base-url', config.get('VENICE_API_BASE_URL', 'https://api.venice.ai/v1'),
        '--api-key', os.environ['VENICE_API_KEY'],
        '--timeout', config.get('VENICE_REQUEST_TIMEOUT', '60'),
        '--out', str(output_dir),
        '--verbose'
    ]

    print(f"Running command: {' '.join(cmd)}")

    try:
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        print("Scan completed successfully!")
        print("STDOUT:", result.stdout)
        if result.stderr:
            print("STDERR:", result.stderr)
        return True
    except subprocess.CalledProcessError as e:
        print(f"Scan failed with return code {e.returncode}")
        print("STDOUT:", e.stdout)
        print("STDERR:", e.stderr)
        return False

def main():
    parser = argparse.ArgumentParser(description='Run HQE Workbench with Venice AI API')
    parser.add_argument('--repo', default='./example-repo', help='Path to repository to scan (default: ./example-repo)')
    parser.add_argument('--config', default='./venice.config', help='Path to Venice config file (default: ./venice.config)')
    parser.add_argument('--output', help='Output directory for scan results')

    args = parser.parse_args()

    # Validate environment
    if not validate_environment():
        sys.exit(1)

    # Load configuration
    config = load_venice_config(args.config)

    # Set up output directory
    if args.output:
        output_dir = Path(args.output)
    else:
        timestamp = datetime.now().strftime('%Y%m%d-%H%M%S')
        output_dir = Path(f'./scan-results-{timestamp}')

    output_dir.mkdir(parents=True, exist_ok=True)

    print("="*60)
    print("HQE Workbench - Venice AI API Test")
    print("="*60)
    print(f"Repository: {args.repo}")
    print(f"Output directory: {output_dir.absolute()}")
    print(f"Venice API endpoint: {config.get('VENICE_API_BASE_URL', 'https://api.venice.ai/v1')}")
    print(f"Venice model: {config.get('VENICE_MODEL_NAME', 'venice-medium')}")
    print("-"*60)

    # Run the scan
    success = run_hqe_scan(args.repo, str(output_dir), config)

    if success:
        print("\nScan completed successfully!")
        print(f"Results are available in: {output_dir.absolute()}")

        # Show summary if report is available
        report_path = output_dir / 'report.json'
        if report_path.exists():
            print("\nReport Summary:")
            try:
                with open(report_path, 'r') as f:
                    report = json.load(f)

                # Print basic statistics
                if 'executive_summary' in report:
                    summary = report['executive_summary']
                    print(f"  Health Score: {summary.get('health_score', 'N/A')}")
                    print(f"  Top Priorities: {len(summary.get('top_priorities', []))} items")
                    print(f"  Critical Findings: {len(summary.get('critical_findings', []))} items")

                if 'project_map' in report:
                    project_map = report['project_map']
                    if 'architecture' in project_map:
                        arch = project_map['architecture']
                        print(f"  Languages Detected: {len(arch.get('languages', []))}")

                if 'deep_scan_results' in report:
                    scan_results = report['deep_scan_results']
                    print(f"  Security Issues: {len(scan_results.get('security', []))}")
                    print(f"  Code Quality Issues: {len(scan_results.get('code_quality', []))}")

                if 'master_todo_backlog' in report:
                    todos = report['master_todo_backlog']
                    print(f"  TODO Items: {len(todos)}")

            except Exception as e:
                print(f"  Could not parse report.json: {e}")
        else:
            print(f"\nNo report.json found in output directory")
    else:
        print("\nScan failed!")
        sys.exit(1)

    print("\n" + "="*60)
    print("HQE Workbench scan completed")
    print("="*60)

if __name__ == '__main__':
    main()