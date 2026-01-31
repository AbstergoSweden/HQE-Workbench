import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { invoke } from '@tauri-apps/api/core'
import { ThinktankScreen } from '../screens/ThinktankScreen'
import { renderWithProviders } from './test-utils'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

describe('ThinktankScreen', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset()
  })

  it('casts args based on schema before execution', async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === 'get_available_prompts') {
        return Promise.resolve([
          {
            name: 'demo_prompt',
            description: 'Demo prompt',
            input_schema: {
              properties: {
                count: { type: 'integer' },
                enabled: { type: 'boolean' },
              },
            },
            template: 'Hello',
          },
        ])
      }
      if (cmd === 'execute_prompt') {
        return Promise.resolve({ result: 'ok' })
      }
      return Promise.resolve(undefined)
    })

    renderWithProviders(<ThinktankScreen />, '/thinktank')

    const promptButton = await screen.findByRole('button', { name: /demo prompt/i })
    await userEvent.click(promptButton)

    const countInput = screen.getByLabelText('count')
    await userEvent.clear(countInput)
    await userEvent.type(countInput, '3')

    const enabledCheckbox = screen.getByLabelText('enabled')
    await userEvent.click(enabledCheckbox)

    const runButton = screen.getByRole('button', { name: /run prompt/i })
    await userEvent.click(runButton)

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('execute_prompt', {
        request: {
          tool_name: 'demo_prompt',
          args: { count: 3, enabled: true },
          profile_name: null,
        },
      })
    })
  })

  it('hides agent/tool prompts by default and allows showing them', async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === 'get_available_prompts') {
        return Promise.resolve([
          { name: 'demo_prompt', description: 'Demo prompt', input_schema: { properties: { args: { type: 'string' } } }, template: 'Hello' },
          { name: 'conductor_setup', description: 'Agent prompt', input_schema: { properties: {} }, template: 'Agent-only' },
        ])
      }
      return Promise.resolve(undefined)
    })

    renderWithProviders(<ThinktankScreen />, '/thinktank')

    // LLM prompt visible
    expect(await screen.findByRole('button', { name: /demo prompt/i })).not.toBeNull()

    // Agent prompt hidden
    expect(screen.queryByRole('button', { name: /conductor setup/i })).toBeNull()

    // Toggle shows agent prompts
    const showToggle = screen.getByRole('checkbox', { name: /show agent\/tool prompts/i })
    await userEvent.click(showToggle)

    expect(await screen.findByRole('button', { name: /conductor setup/i })).not.toBeNull()
  })
})
