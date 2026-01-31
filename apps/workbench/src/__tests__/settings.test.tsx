import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { invoke } from '@tauri-apps/api/core'
import { SettingsScreen } from '../screens/SettingsScreen'
import { renderWithProviders } from './test-utils'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

describe('SettingsScreen', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset()
  })

  it('discovers models and populates the dropdown', async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === 'list_provider_profiles') {
        return Promise.resolve([])
      }
      if (cmd === 'discover_models') {
        return Promise.resolve({
          models: [{ id: 'venice-text-1', name: 'Venice Text 1' }],
        })
      }
      return Promise.resolve(undefined)
    })

    renderWithProviders(<SettingsScreen />)

    const baseUrl = screen.getByPlaceholderText('https://api.openai.com/v1')
    await userEvent.type(baseUrl, 'https://api.venice.ai/api/v1')

    const discover = screen.getByRole('button', { name: /discover models/i })
    await userEvent.click(discover)

    await waitFor(() => {
      expect(screen.getByRole('option', { name: 'Venice Text 1' })).not.toBeNull()
    })
  })
})
