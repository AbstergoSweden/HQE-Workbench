import { describe, it, expect } from 'vitest'
import { screen } from '@testing-library/react'
import { ReportScreen } from '../screens/ReportScreen'
import { renderWithProviders } from './test-utils'
import { useReportStore } from '../store'

describe('ReportScreen', () => {
  it('renders local-only banner and reproduction evidence', () => {
    useReportStore.setState({
      report: {
        run_id: 'run-123',
        provider: { name: 'local', llm_enabled: false },
        executive_summary: {
          health_score: 5,
          top_priorities: [],
          critical_findings: [],
          blockers: [],
        },
        deep_scan_results: {
          security: [
            {
              id: 'BUG-001',
              severity: 'high',
              risk: 'high',
              category: 'Bug',
              title: 'Crash on launch',
              evidence: {
                type: 'reproduction',
                steps: ['Open app', 'Click button'],
                observed: 'Crash',
              },
              impact: 'App fails',
              recommendation: 'Fix null handling',
            },
          ],
          code_quality: [],
          frontend: [],
          backend: [],
          testing: [],
        },
        master_todo_backlog: [],
      },
    })

    renderWithProviders(<ReportScreen />, '/report')

    expect(screen.getByText(/local-only analysis/i)).not.toBeNull()
    expect(screen.getByText(/reproduction/i)).not.toBeNull()
  })
})
