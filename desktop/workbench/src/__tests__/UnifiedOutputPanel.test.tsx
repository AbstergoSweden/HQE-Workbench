import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import * as matchers from '@testing-library/jest-dom/matchers'
import { UnifiedOutputPanel } from '../components/UnifiedOutputPanel'
import { renderWithProviders } from './test-utils'
import { invoke } from '@tauri-apps/api/core'

// Extend Vitest expect with jest-dom matchers
expect.extend(matchers)

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

// Mock scrollIntoView for JSDOM
window.HTMLElement.prototype.scrollIntoView = vi.fn()

describe('UnifiedOutputPanel Accessibility', () => {
  const mockContextRef = {
    repo_path: '/tmp/test',
    prompt_id: 'test-prompt',
    provider: 'test-provider',
    model: 'test-model',
  }

  const mockSession = {
    id: 'session-123',
    provider: 'test-provider',
    model: 'test-model',
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    message_count: 0,
  }

  beforeEach(() => {
    vi.clearAllMocks()
    // Mock window.__TAURI_INTERNALS__ to enable Tauri logic
    Object.defineProperty(window, '__TAURI_INTERNALS__', {
      value: {},
      writable: true,
      configurable: true, // Allow deletion
    })
    
    // Setup default invoke implementation
    vi.mocked(invoke).mockImplementation((cmd) => {
      if (cmd === 'create_chat_session') {
        return Promise.resolve(mockSession)
      }
      return Promise.resolve(undefined)
    })
  })
  
  afterEach(() => {
    // Clean up window mock
    // @ts-expect-error - deleting property
    delete window.__TAURI_INTERNALS__
  })

  it('has an accessible send button', async () => {
    renderWithProviders(
      <UnifiedOutputPanel
        contextRef={mockContextRef}
        showInput={true}
        initialMessages={[]}
      />
    )

    // Wait for session to be created (invoke called)
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('create_chat_session', expect.anything())
    })

    // Find input and type to enable send button
    const input = screen.getByPlaceholderText(/ask a follow-up question/i)
    await userEvent.type(input, 'Hello world')

    // Attempt to find the send button by its accessible name
    const sendButton = screen.getByRole('button', { name: /send message/i })
    expect(sendButton).toBeInTheDocument()
    
    // Wait for button to be enabled (state update might be async)
    await waitFor(() => {
        expect(sendButton).toBeEnabled()
    })
  })
})
