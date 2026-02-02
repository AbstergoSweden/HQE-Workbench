/**
 * usePrompts Hook
 * 
 * Provides access to the prompt library with rich metadata.
 * Loads prompts from the enhanced cli-prompt-library.
 */

import { useState, useEffect, useCallback, useMemo } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { PromptMetadata, PromptCategory } from '../types'

export interface Prompt {
  name: string
  description: string
  explanation?: string
  category: PromptCategory
  version: string
  input_schema: {
    properties?: Record<string, {
      type?: string | string[]
      description?: string
      enum?: Array<string | number | boolean>
      default?: unknown
    }>
    required?: string[]
  }
  template: string
}

export interface UsePromptsOptions {
  /** Include agent-specific prompts (hidden by default) */
  includeAgentPrompts?: boolean
  /** Filter by category */
  category?: string
  /** Search query */
  search?: string
}

export interface UsePromptsReturn {
  /** All available prompts */
  prompts: Prompt[]
  /** Loading state */
  loading: boolean
  /** Error if loading failed */
  error: string | null
  /** Refresh prompts from disk */
  refresh: () => Promise<void>
  /** Get a single prompt by name */
  getPrompt: (name: string) => Prompt | undefined
  /** Filtered prompts based on options */
  filteredPrompts: Prompt[]
  /** All unique categories */
  categories: string[]
  /** Count prompts by category */
  countByCategory: Record<string, number>
}

/**
 * Hook for accessing the prompt library
 * 
 * @example
 * ```tsx
 * const { prompts, loading, filteredPrompts } = usePrompts({
 *   category: 'security',
 *   search: 'audit'
 * })
 * ```
 */
export function usePrompts(options: UsePromptsOptions = {}): UsePromptsReturn {
  const { includeAgentPrompts = false, category, search } = options
  
  const [prompts, setPrompts] = useState<Prompt[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)



  const loadPrompts = useCallback(async () => {
    setLoading(true)
    setError(null)
    
    try {
      // Try enhanced prompts with metadata first
      let loaded: Prompt[] = []
      try {
        const result = await invoke<Prompt[]>('get_available_prompts_with_metadata')
        // Handle case where mock returns undefined or empty
        loaded = result || []
        // If empty, try fallback
        if (loaded.length === 0) {
          const fallback = await invoke<Prompt[]>('get_available_prompts')
          loaded = fallback || []
        }
      } catch {
        // Fallback to basic prompts
        const result = await invoke<Prompt[]>('get_available_prompts')
        loaded = result || []
      }
      
      setPrompts(loaded)
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      setError(message)
      console.error('Failed to load prompts:', err)
    } finally {
      setLoading(false)
    }
  }, [])

  // Load on mount
  useEffect(() => {
    loadPrompts()
  }, [loadPrompts])

  // Filter prompts based on options
  const filteredPrompts = useMemo(() => {
    let filtered = prompts

    // Filter out agent prompts unless included
    if (!includeAgentPrompts) {
      filtered = filtered.filter(p => 
        !p.name.startsWith('conductor_') && !p.name.startsWith('cli_security_')
      )
    }

    // Filter by category
    if (category && category !== 'all') {
      filtered = filtered.filter(p => 
        p.category?.toLowerCase() === category.toLowerCase()
      )
    }

    // Filter by search query
    if (search) {
      const query = search.toLowerCase()
      filtered = filtered.filter(p =>
        p.name.toLowerCase().includes(query) ||
        p.description.toLowerCase().includes(query) ||
        p.explanation?.toLowerCase().includes(query)
      )
    }

    return filtered
  }, [prompts, includeAgentPrompts, category, search])

  // Get unique categories
  const categories = useMemo(() => {
    const cats = new Set<string>()
    prompts.forEach(p => {
      if (p.category) {
        cats.add(p.category.toLowerCase())
      }
    })
    return Array.from(cats).sort()
  }, [prompts])

  // Count by category
  const countByCategory = useMemo(() => {
    const counts: Record<string, number> = {}
    prompts.forEach(p => {
      const cat = p.category?.toLowerCase() || 'custom'
      counts[cat] = (counts[cat] || 0) + 1
    })
    return counts
  }, [prompts])

  // Get single prompt by name
  const getPrompt = useCallback((name: string): Prompt | undefined => {
    return prompts.find(p => p.name === name || `prompts__${p.name}` === name)
  }, [prompts])

  return {
    prompts,
    loading,
    error,
    refresh: loadPrompts,
    getPrompt,
    filteredPrompts,
    categories,
    countByCategory,
  }
}

export default usePrompts
