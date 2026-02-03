/**
 * usePromptExecution Hook
 * 
 * Handles prompt execution with the AI and manages the execution lifecycle.
 * Integrates with the chat system for follow-up conversations.
 */

import { useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { ChatMessage } from '../types'

export interface PromptExecutionRequest {
  /** Name of the prompt tool to execute */
  tool_name: string
  /** Arguments for the prompt template */
  args: Record<string, unknown>
  /** Provider profile to use (optional, uses default if not specified) */
  profile_name?: string
  /** Model override (optional) */
  model?: string
}

export interface PromptExecutionResponse {
  /** The generated result from the AI */
  result: string
  /** System prompt version used */
  system_prompt_version?: string
}

export interface UsePromptExecutionReturn {
  /** Execute a prompt */
  execute: (request: PromptExecutionRequest) => Promise<string>
  /** Current execution state */
  executing: boolean
  /** Error from last execution */
  error: string | null
  /** Result from last execution */
  result: string | null
  /** Clear result and error */
  reset: () => void
  /** Convert result to chat messages for UnifiedOutputPanel */
  toChatMessages: () => ChatMessage[]
}

/**
 * Hook for executing prompts against the AI
 * 
 * @example
 * ```tsx
 * const { execute, executing, result } = usePromptExecution()
 * 
 * const handleRun = async () => {
 *   const output = await execute({
 *     tool_name: 'security_audit',
 *     args: { code: '...' }
 *   })
 *   console.log(output)
 * }
 * ```
 */
export function usePromptExecution(): UsePromptExecutionReturn {
  const [executing, setExecuting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [result, setResult] = useState<string | null>(null)

  const execute = useCallback(async (
    request: PromptExecutionRequest
  ): Promise<string> => {
    setExecuting(true)
    setError(null)
    setResult(null)

    try {
      const response = await invoke<PromptExecutionResponse>('execute_prompt', {
        request: {
          tool_name: request.tool_name,
          args: request.args,
          profile_name: request.profile_name ?? null,
          model: request.model ?? null,
        }
      })

      setResult(response.result)
      return response.result
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      setError(message)
      throw err
    } finally {
      setExecuting(false)
    }
  }, [])

  const reset = useCallback(() => {
    setResult(null)
    setError(null)
  }, [])

  const toChatMessages = useCallback((): ChatMessage[] => {
    if (!result) return []

    return [{
      id: `prompt-result-${Date.now()}`,
      session_id: '',
      role: 'assistant',
      content: result,
      timestamp: new Date().toISOString(),
    }]
  }, [result])

  return {
    execute,
    executing,
    error,
    result,
    reset,
    toChatMessages,
  }
}

export default usePromptExecution
