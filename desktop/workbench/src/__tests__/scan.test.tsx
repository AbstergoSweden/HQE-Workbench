import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { invoke } from '@tauri-apps/api/core'
import { ScanScreen } from '../screens/ScanScreen'
import { renderWithProviders } from './test-utils'
import { useRepoStore, useReportStore, useScanStore } from '../store'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const sampleReport = {
  run_id: 'run-123',
  provider: { name: 'local', llm_enabled: false },
  executive_summary: {
    health_score: 7,
    top_priorities: [],
    critical_findings: [],
    blockers: [],
  },
  deep_scan_results: {
    security: [],
    code_quality: [],
    frontend: [],
    backend: [],
    testing: [],
  },
  master_todo_backlog: [],
}

describe('ScanScreen', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset()
    useRepoStore.setState({ path: '/tmp/repo', name: 'repo', isGit: false })
    useReportStore.setState({ report: null })
    useScanStore.setState({ isScanning: false, phase: null, progress: 0, error: null })
  })

  it('invokes scan_repo when Start Scan is clicked', async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === 'scan_repo') {
        return Promise.resolve(sampleReport)
      }
      if (cmd === 'list_provider_profiles') {
        return Promise.resolve([])
      }
      return Promise.resolve(undefined)
    })

    renderWithProviders(<ScanScreen />, '/scan')

    const startButton = screen.getByRole('button', { name: /execute_scan/i })
    await userEvent.click(startButton)

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('scan_repo', expect.any(Object))
    })
  })
})
