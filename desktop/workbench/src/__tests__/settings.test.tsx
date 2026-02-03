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
      if (cmd === 'get_provider_specs') {
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

    const nameInput = screen.getByPlaceholderText('my-provider')
    await userEvent.type(nameInput, 'Venice')

    const baseUrl = screen.getByPlaceholderText('https://api.openai.com/v1')
    await userEvent.type(baseUrl, 'https://api.venice.ai/api/v1')

    // API key is also required for discovery
    const apiKey = screen.getByPlaceholderText('sk-...')
    await userEvent.type(apiKey, 'test-api-key')

    const discover = screen.getByRole('button', { name: /^discover$/i })
    await userEvent.click(discover)

    await waitFor(() => {
      expect(screen.getByRole('option', { name: 'Venice Text 1' })).not.toBeNull()
    })
  })

  it('uses stored API key and headers for model discovery when available', async () => {
    const profile = {
      name: 'Venice',
      base_url: 'https://api.venice.ai/api/v1',
      api_key_id: 'api_key:Venice',
      default_model: 'venice-text-1',
      headers: { 'X-Test': '1' },
      organization: null,
      project: null,
      provider_kind: 'venice',
      timeout_s: 60,
    }

    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === 'list_provider_profiles') {
        return Promise.resolve([profile])
      }
      if (cmd === 'get_provider_specs') {
        return Promise.resolve([])
      }
      if (cmd === 'get_api_key') {
        return Promise.resolve('stored-secret')
      }
      if (cmd === 'discover_models') {
        return Promise.resolve({ models: [] })
      }
      return Promise.resolve(undefined)
    })

    renderWithProviders(<SettingsScreen />)

    // Wait for profile to appear as a button in the profiles list
    const profileButton = await screen.findByRole('button', { name: /Venice/i })
    await userEvent.click(profileButton)

    // Verify that get_api_key was called to retrieve the stored API key
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('get_api_key', { apiKeyId: 'api_key:Venice' })
    })

    const discover = screen.getByRole('button', { name: /^discover$/i })
    await userEvent.click(discover)

    // Discovery now uses profile_name
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('discover_models', {
        profile_name: 'Venice',
      })
    })
  })
})
