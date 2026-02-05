import { FC, useCallback, useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useLocation } from 'react-router-dom'
import { useToast } from '../context/ToastContext'
import { UnifiedOutputPanel } from '../components/UnifiedOutputPanel'
import { usePrompts, usePromptExecution } from '../hooks'
import { ChatMessage, ProviderProfile } from '../types'
import { PlayIcon, ChatBubbleLeftRightIcon } from '@heroicons/react/24/solid'

// Extended Prompt interface with metadata
interface PromptTool {
  name: string
  description: string
  explanation?: string
  category?: string
  version?: string
  input_schema?: { properties?: Record<string, JSONSchemaProperty> }
  template: string
}

interface JSONSchemaProperty {
  type?: string | string[]
  description?: string
  enum?: Array<string | number | boolean>
  default?: unknown
  [key: string]: unknown
}

const getSchemaType = (schema: JSONSchemaProperty): string => {
  if (Array.isArray(schema.type)) {
    return schema.type.find((t) => t !== 'null') || schema.type[0] || 'string'
  }
  if (typeof schema.type === 'string') {
    return schema.type
  }
  if (schema.enum && schema.enum.length > 0) {
    const first = schema.enum[0]
    return typeof first
  }
  return 'string'
}

const buildTypedArgs = (
  properties: Record<string, JSONSchemaProperty>,
  rawArgs: Record<string, unknown>
) => {
  const typed: Record<string, unknown> = {}
  Object.entries(properties).forEach(([key, schema]) => {
    const type = getSchemaType(schema)
    const raw = rawArgs[key]

    if (raw === undefined || raw === null) {
      return
    }

    if (type === 'boolean') {
      if (typeof raw === 'boolean') {
        typed[key] = raw
      } else if (typeof raw === 'string') {
        typed[key] = raw.toLowerCase() === 'true'
      } else {
        typed[key] = Boolean(raw)
      }
      return
    }

    if (type === 'number' || type === 'integer') {
      if (typeof raw === 'number') {
        typed[key] = type === 'integer' ? Math.trunc(raw) : raw
        return
      }
      if (typeof raw === 'string') {
        if (raw.trim() === '') {
          return
        }
        const parsed = type === 'integer' ? parseInt(raw, 10) : parseFloat(raw)
        if (Number.isNaN(parsed)) {
          throw new Error(`Invalid number for "${key}"`)
        }
        typed[key] = parsed
        return
      }
    }

    if (type === 'object' || type === 'array') {
      if (typeof raw === 'string') {
        if (raw.trim() === '') {
          return
        }
        try {
          typed[key] = JSON.parse(raw)
        } catch {
          throw new Error(`Invalid JSON for "${key}"`)
        }
        return
      }
      typed[key] = raw
      return
    }

    typed[key] = raw
  })
  return typed
}

// Category colors for the UI
const categoryColors: Record<string, { bg: string; border: string; text: string }> = {
  security: { bg: 'var(--dracula-red)10', border: 'var(--dracula-red)40', text: 'var(--dracula-red)' },
  quality: { bg: 'var(--dracula-green)10', border: 'var(--dracula-green)40', text: 'var(--dracula-green)' },
  refactor: { bg: 'var(--dracula-orange)10', border: 'var(--dracula-orange)40', text: 'var(--dracula-orange)' },
  explain: { bg: 'var(--dracula-cyan)10', border: 'var(--dracula-cyan)40', text: 'var(--dracula-cyan)' },
  test: { bg: 'var(--dracula-purple)10', border: 'var(--dracula-purple)40', text: 'var(--dracula-purple)' },
  document: { bg: 'var(--dracula-yellow)10', border: 'var(--dracula-yellow)40', text: 'var(--dracula-yellow)' },
  architecture: { bg: 'var(--dracula-pink)10', border: 'var(--dracula-pink)40', text: 'var(--dracula-pink)' },
  performance: { bg: 'var(--dracula-comment)10', border: 'var(--dracula-comment)40', text: 'var(--dracula-comment)' },
  dependencies: { bg: 'var(--dracula-cyan)10', border: 'var(--dracula-cyan)40', text: 'var(--dracula-cyan)' },
  custom: { bg: 'var(--dracula-green)10', border: 'var(--dracula-green)40', text: 'var(--dracula-green)' },
  agent: { bg: 'var(--dracula-orange)10', border: 'var(--dracula-orange)40', text: 'var(--dracula-orange)' },
}

const getCategoryStyle = (category?: string) => {
  const key = category?.toLowerCase() || 'custom'
  return categoryColors[key] || categoryColors.custom
}

const isAgentPrompt = (name: string): boolean => {
  return name.startsWith('conductor_') || name.startsWith('cli_security_') || name.startsWith('agent_')
}

export const ThinktankScreen: FC = () => {
  const location = useLocation()
  const toast = useToast()

  // Local UI state
  const [filters, setFilters] = useState({
    searchQuery: '',
    showAgentPrompts: false,
    selectedCategory: 'all',
  })
  const { searchQuery, showAgentPrompts, selectedCategory } = filters
  const [selectedPrompt, setSelectedPrompt] = useState<PromptTool | null>(null)
  const [args, setArgs] = useState<Record<string, unknown>>({})
  const [chatMode, setChatMode] = useState(false)
  const [initialMessages, setInitialMessages] = useState<ChatMessage[]>([])
  const [profiles, setProfiles] = useState<ProviderProfile[]>([])
  const [selectedProfile, setSelectedProfile] = useState<string>('')
  const [selectedModel, setSelectedModel] = useState<string>('default')

  // Use the enhanced prompts hook
  const {
    prompts,
    filteredPrompts,
    loading,
    error: promptsError,
    refresh,
    categories,
    countByCategory,
  } = usePrompts({
    includeAgentPrompts: showAgentPrompts,
    category: selectedCategory,
    search: searchQuery,
  })

  // Use the prompt execution hook
  const {
    execute,
    executing,
    error: executionError,
    result,
    reset,
  } = usePromptExecution()

  useEffect(() => {
    const loadProfiles = async () => {
      try {
        const result = await invoke<ProviderProfile[]>('list_provider_profiles')
        setProfiles(result ?? [])
        if (result && result.length > 0) {
          setSelectedProfile(result[0].name)
          setSelectedModel(result[0].default_model || 'default')
        }
      } catch (error) {
        console.error('Failed to load provider profiles:', error)
        toast.error('Failed to load provider profiles')
      }
    }
    loadProfiles()
  }, [toast])

  const handleSelectPrompt = useCallback((prompt: PromptTool, initialArgs?: Record<string, unknown>) => {
    if (isAgentPrompt(prompt.name)) {
      setFilters(prev => ({ ...prev, showAgentPrompts: true }))
    }
    setSelectedPrompt(prompt)
    reset()
    setChatMode(false)
    setInitialMessages([])

    // Initialize args from schema
    const newArgs: Record<string, unknown> = {}
    const properties = prompt.input_schema?.properties || {}
    Object.entries(properties).forEach(([key, schema]) => {
      const initial = initialArgs?.[key]
      if (initial !== undefined) {
        newArgs[key] = initial
        return
      }
      if (schema.default !== undefined) {
        newArgs[key] = schema.default
        return
      }
      const type = getSchemaType(schema)
      if (type === 'boolean') {
        newArgs[key] = false
      } else {
        newArgs[key] = ''
      }
    })
    setArgs(newArgs)
  }, [reset])

  // Handle incoming navigation state
  useEffect(() => {
    const state = location.state as { promptName?: string; args?: Record<string, unknown> } | null
    if (state?.promptName && prompts.length > 0) {
      const target = prompts.find(p =>
        p.name === state.promptName || `prompts__${p.name}` === state.promptName
      )
      if (target) {
        // Wrap in a microtask to avoid synchronous setState during render
        Promise.resolve().then(() => {
          handleSelectPrompt(target, state.args)
        })
      }
    }
  }, [location.state, prompts, handleSelectPrompt])

  const handleExecute = async () => {
    if (!selectedPrompt) return

    const properties = selectedPrompt.input_schema?.properties || {}
    const typedArgs =
      Object.keys(properties).length === 0 ? args : buildTypedArgs(properties, args)

    try {
      if (!selectedProfile) {
        toast.error('Select a provider profile in Settings')
        return
      }
      const response = await execute({
        tool_name: selectedPrompt.name,
        args: typedArgs,
        profile_name: selectedProfile || undefined,
        model: selectedModel || undefined,
      })

      // Create initial message for potential chat transition
      const assistantMessage: ChatMessage = {
        id: `report-stable`,
        session_id: '',
        role: 'assistant',
        content: response,
        timestamp: new Date().toISOString(),
      }
      setInitialMessages([assistantMessage])
    } catch {
      toast.error('Prompt execution failed')
    }
  }

  const handleStartChat = () => {
    if (result) {
      setChatMode(true)
    }
  }

  const displayedError = promptsError || executionError
  const hiddenAgentCount = prompts.filter(p => isAgentPrompt(p.name)).length

  const selectedProfileInfo = profiles.find((p) => p.name === selectedProfile)

  // Chat mode view
  if (chatMode && result) {
    return (
      <div className="flex h-full gap-4">
        {/* Left sidebar with prompt info */}
        <div className="w-64 flex flex-col card p-4" style={{ borderColor: 'var(--dracula-comment)' }}>
          <div className="flex items-center gap-2 mb-4">
            <button
              onClick={() => setChatMode(false)}
              className="text-xs hover:text-terminal-cyan"
              style={{ color: 'var(--dracula-comment)' }}
            >
              ← Back to prompt
            </button>
          </div>

          <h3 className="font-mono text-sm mb-2" style={{ color: 'var(--dracula-fg)' }}>
            {selectedPrompt?.name.replace(/_/g, ' ')}
          </h3>

          {selectedPrompt?.category && (
            <span
              className="text-xs px-2 py-0.5 rounded w-fit mb-3"
              style={{
                backgroundColor: getCategoryStyle(selectedPrompt.category).bg,
                color: getCategoryStyle(selectedPrompt.category).text,
                border: `1px solid ${getCategoryStyle(selectedPrompt.category).border}`,
              }}
            >
              {selectedPrompt.category}
            </span>
          )}

          <p className="text-xs mb-4" style={{ color: 'var(--dracula-comment)' }}>
            {selectedPrompt?.description}
          </p>

          <div className="mt-auto pt-4 border-t" style={{ borderColor: 'var(--dracula-comment)' }}>
            <p className="text-xs" style={{ color: 'var(--dracula-comment)' }}>
              Ask follow-up questions about the analysis results.
            </p>
          </div>
        </div>

        {/* Main chat area */}
        <div className="flex-1 card overflow-hidden" style={{ borderColor: 'var(--dracula-comment)' }}>
          <UnifiedOutputPanel
            initialMessages={initialMessages}
            contextRef={{
              prompt_id: selectedPrompt?.name,
              provider: selectedProfile || 'default',
              model: selectedModel || 'default',
            }}
            showInput={true}
          />
        </div>
      </div>
    )
  }

  return (
    <div className="flex h-full gap-4">
      {/* Sidebar: Prompt List */}
      <div
        className="w-80 flex flex-col card p-0 overflow-hidden"
        style={{ borderColor: 'var(--dracula-comment)' }}
      >
        <div
          className="p-3 border-b flex flex-col gap-3"
          style={{ borderColor: 'var(--dracula-comment)' }}
        >
          <div className="flex justify-between items-center">
            <span className="text-xs uppercase tracking-wider" style={{ color: 'var(--dracula-comment)' }}>
              Prompt Library
            </span>
            <button
              onClick={refresh}
              className="text-xs p-1 rounded transition-colors hover:text-terminal-cyan"
              disabled={loading}
              style={{ color: 'var(--dracula-comment)' }}
              title="Refresh Library"
            >
              {loading ? <span className="animate-spin">⟳</span> : '↻'}
            </button>
          </div>

          {/* Category filter */}
          <select
            value={selectedCategory}
            onChange={(e) =>
              setFilters(prev => ({ ...prev, selectedCategory: e.target.value }))
            }
            className="input text-sm"
          >
            <option value="all">All Categories ({prompts.length})</option>
            {categories.map(cat => (
              <option key={cat} value={cat}>
                {cat.charAt(0).toUpperCase() + cat.slice(1)} ({countByCategory[cat] || 0})
              </option>
            ))}
          </select>

          {/* Search */}
          <input
            type="text"
            placeholder="Search prompts..."
            value={searchQuery}
            onChange={(e) =>
              setFilters(prev => ({ ...prev, searchQuery: e.target.value }))
            }
            className="input text-sm"
          />

          {/* Agent prompt toggle */}
          <label className="flex items-center gap-2 text-xs" style={{ color: 'var(--dracula-comment)' }}>
            <input
              type="checkbox"
              checked={showAgentPrompts}
              onChange={(e) =>
                setFilters(prev => ({ ...prev, showAgentPrompts: e.target.checked }))
              }
            />
            Show agent prompts {hiddenAgentCount > 0 && `(${hiddenAgentCount} hidden)`}
          </label>
        </div>

        <div className="flex-1 overflow-auto">
          {loading ? (
            <div className="p-4 space-y-2">
              {[...Array(6)].map((_, i) => (
                <div
                  key={i}
                  className="h-10 w-full animate-pulse"
                  style={{ background: 'var(--dracula-current-line)' }}
                />
              ))}
            </div>
          ) : filteredPrompts.length === 0 ? (
            <div className="p-6 text-center flex flex-col items-center gap-3">
              <div className="text-3xl opacity-50">📭</div>
              <p className="text-sm font-medium" style={{ color: 'var(--dracula-fg)' }}>
                No prompts found
              </p>
              <p className="text-xs" style={{ color: 'var(--dracula-comment)' }}>
                Add <code className="px-1 py-0.5 rounded" style={{ background: 'var(--dracula-current-line)' }}>.toml</code> files to your <code className="px-1 py-0.5 rounded" style={{ background: 'var(--dracula-current-line)' }}>mcp-server/</code> folder or set <code className="px-1 py-0.5 rounded" style={{ background: 'var(--dracula-current-line)' }}>HQE_PROMPTS_DIR</code>
              </p>
              <button
                onClick={refresh}
                className="btn text-xs mt-2"
                style={{ borderColor: 'var(--dracula-cyan)' }}
              >
                ↻ refresh
              </button>
            </div>
          ) : (
            <div className="flex flex-col">
              {filteredPrompts.map(p => (
                <button
                  key={p.name}
                  onClick={() => handleSelectPrompt(p as PromptTool)}
                  className="p-3 text-left text-sm border-b transition-all duration-150"
                  style={{
                    borderColor: 'var(--dracula-comment)',
                    backgroundColor: selectedPrompt?.name === p.name
                      ? 'var(--dracula-bg)'
                      : 'transparent',
                    borderLeft: selectedPrompt?.name === p.name
                      ? '2px solid var(--dracula-green)'
                      : '2px solid transparent',
                    color: selectedPrompt?.name === p.name
                      ? 'var(--dracula-green)'
                      : 'var(--dracula-fg)',
                  }}
                  onMouseEnter={(e) => {
                    if (selectedPrompt?.name !== p.name) {
                      e.currentTarget.style.backgroundColor = 'var(--dracula-current-line)'
                    }
                  }}
                  onMouseLeave={(e) => {
                    if (selectedPrompt?.name !== p.name) {
                      e.currentTarget.style.backgroundColor = 'transparent'
                    }
                  }}
                >
                  <div className="font-mono text-sm truncate flex items-center gap-2">
                    {p.name.replace(/_/g, ' ')}
                    {isAgentPrompt(p.name) && (
                      <span
                        className="text-[10px] px-1.5 py-0.5 rounded"
                        style={{
                          background: 'var(--dracula-orange)20',
                          color: 'var(--dracula-orange)',
                          border: '1px solid var(--dracula-orange)'
                        }}
                      >
                        AGENT
                      </span>
                    )}
                    {p.category && !isAgentPrompt(p.name) && (
                      <span
                        className="text-[10px] px-1.5 py-0.5 rounded"
                        style={{
                          background: getCategoryStyle(p.category).bg,
                          color: getCategoryStyle(p.category).text,
                          border: `1px solid ${getCategoryStyle(p.category).border}`,
                        }}
                      >
                        {p.category}
                      </span>
                    )}
                  </div>
                  <div className="text-xs truncate mt-1" style={{ color: 'var(--dracula-comment)' }}>
                    {p.description}
                  </div>
                </button>
              ))}
            </div>
          )}
        </div>

        <div
          className="p-2 border-t text-center text-xs"
          style={{ borderColor: 'var(--dracula-comment)', color: 'var(--dracula-comment)' }}
        >
          {filteredPrompts.length} prompts displayed
          {hiddenAgentCount > 0 && !showAgentPrompts && ` • ${hiddenAgentCount} agent prompts hidden`}
        </div>
      </div>

      {/* Main Content: Input & Execution */}
      <div className="flex-1 flex flex-col gap-4 overflow-hidden">
        {displayedError && (
          <div
            className="p-3 rounded text-sm"
            style={{
              background: 'var(--dracula-red)20',
              border: '1px solid var(--dracula-red)',
              color: 'var(--dracula-red)'
            }}
          >
            ⚠️ {displayedError}
          </div>
        )}

        {!selectedPrompt ? (
          <div
            className="flex-1 flex items-center justify-center card"
            style={{ borderColor: 'var(--dracula-comment)', borderStyle: 'dashed' }}
          >
            <div className="text-center" style={{ color: 'var(--dracula-comment)' }}>
              <div className="text-4xl mb-4">🧠</div>
              <p>Select a prompt from the library to begin</p>
              <p className="text-sm mt-2 opacity-70">
                {prompts.length} prompts available
              </p>
            </div>
          </div>
        ) : (
          <div className="flex-1 flex flex-col gap-4 overflow-hidden">
            {/* Input Section */}
            <div
              className="card p-4"
              style={{ borderColor: 'var(--dracula-comment)' }}
            >
              <div className="flex items-center gap-2 mb-4">
                <span className="text-terminal-green">❯</span>
                <h2 className="font-mono text-lg" style={{ color: 'var(--dracula-fg)' }}>
                  {selectedPrompt.name.replace(/_/g, ' ')}
                </h2>
                <span className="font-mono text-xs" style={{ color: 'var(--dracula-comment)' }}>
                  ({selectedPrompt.name})
                </span>
                {selectedPrompt.version && (
                  <span className="font-mono text-xs" style={{ color: 'var(--dracula-comment)' }}>
                    v{selectedPrompt.version}
                  </span>
                )}
              </div>

              <div className="grid grid-cols-2 gap-3 mb-4">
                <div>
                  <label className="text-terminal-cyan font-mono text-sm block mb-1">
                    --profile
                  </label>
                  <select
                    value={selectedProfile}
                    onChange={(e) => {
                      const name = e.target.value
                      setSelectedProfile(name)
                      const profile = profiles.find((p) => p.name === name)
                      setSelectedModel(profile?.default_model || 'default')
                    }}
                    className="input text-sm"
                  >
                    {profiles.length === 0 && (
                      <option value="">No profiles configured</option>
                    )}
                    {profiles.map((profile) => (
                      <option key={profile.name} value={profile.name}>
                        {profile.name}
                      </option>
                    ))}
                  </select>
                </div>
                <div>
                  <label className="text-terminal-cyan font-mono text-sm block mb-1">
                    --model
                  </label>
                  <input
                    type="text"
                    value={selectedModel}
                    onChange={(e) => setSelectedModel(e.target.value)}
                    className="input text-sm"
                    placeholder={selectedProfileInfo?.default_model || 'default'}
                  />
                </div>
              </div>

              {isAgentPrompt(selectedPrompt.name) && (
                <div
                  className="mb-4 text-sm p-3 rounded"
                  style={{
                    background: 'var(--dracula-orange)10',
                    border: '1px solid var(--dracula-orange)30',
                    color: 'var(--dracula-orange)'
                  }}
                >
                  ⚠ This prompt is designed for an agent runtime with tool/file access.
                  Thinktank will only send text to the model.
                </div>
              )}

              {/* Description */}
              <p className="text-sm mb-2" style={{ color: 'var(--dracula-comment)' }}>
                {selectedPrompt.description}
              </p>

              {/* Explanation (if available) */}
              {selectedPrompt.explanation && (
                <div
                  className="mb-4 p-3 rounded text-sm"
                  style={{
                    background: 'var(--dracula-current-line)',
                    border: '1px solid var(--dracula-comment)30',
                  }}
                >
                  <div className="text-xs uppercase tracking-wider mb-2" style={{ color: 'var(--dracula-cyan)' }}>
                    About this prompt
                  </div>
                  <div style={{ color: 'var(--dracula-fg)' }}>
                    {selectedPrompt.explanation}
                  </div>
                </div>
              )}

              <div className="flex flex-col gap-3">
                {Object.entries(selectedPrompt.input_schema?.properties || {}).map(([key, schema]) => {
                  const type = getSchemaType(schema)
                  const enumValues = Array.isArray(schema.enum) ? schema.enum : null
                  const value = args[key]
                  const inputId = `prompt-field-${key}`
                  return (
                    <div key={key} className="flex flex-col gap-1">
                      <label
                        htmlFor={inputId}
                        className="text-xs font-mono"
                        style={{ color: 'var(--dracula-cyan)' }}
                      >
                        --{key}
                        {schema.description && (
                          <span className="ml-2 font-normal" style={{ color: 'var(--dracula-comment)' }}>
                            ({schema.description})
                          </span>
                        )}
                      </label>
                      {enumValues ? (
                        <select
                          id={inputId}
                          value={String(value ?? enumValues[0] ?? '')}
                          onChange={(e) => {
                            const selected = enumValues.find((opt) => String(opt) === e.target.value)
                            setArgs((prev) => ({ ...prev, [key]: selected ?? e.target.value }))
                          }}
                          className="input"
                        >
                          {enumValues.map((opt) => (
                            <option key={String(opt)} value={String(opt)}>
                              {String(opt)}
                            </option>
                          ))}
                        </select>
                      ) : type === 'boolean' ? (
                        <label className="flex items-center gap-2 text-sm" style={{ color: 'var(--dracula-fg)' }}>
                          <input
                            id={inputId}
                            type="checkbox"
                            checked={Boolean(value)}
                            onChange={(e) => setArgs((prev) => ({ ...prev, [key]: e.target.checked }))}
                          />
                          {schema.description || `Enable ${key}`}
                        </label>
                      ) : type === 'number' || type === 'integer' ? (
                        <input
                          id={inputId}
                          type="number"
                          step={type === 'integer' ? '1' : 'any'}
                          value={value === undefined || value === null ? '' : String(value)}
                          onChange={(e) => setArgs((prev) => ({ ...prev, [key]: e.target.value }))}
                          className="input"
                          placeholder={schema.description || `Enter ${key}...`}
                        />
                      ) : type === 'object' || type === 'array' || key === 'args' || (type === 'string' && !schema.description?.includes('short')) ? (
                        <textarea
                          id={inputId}
                          value={value === undefined ? '' : String(value)}
                          onChange={(e) => setArgs((prev) => ({ ...prev, [key]: e.target.value }))}
                          className="input min-h-[100px]"
                          placeholder={schema.description || `Enter ${key}...`}
                        />
                      ) : (
                        <input
                          id={inputId}
                          type="text"
                          value={value === undefined ? '' : String(value)}
                          onChange={(e) => setArgs((prev) => ({ ...prev, [key]: e.target.value }))}
                          className="input"
                          placeholder={schema.description || `Enter ${key}...`}
                        />
                      )}
                    </div>
                  )
                })}

                {/* Execute Button & Chat Access */}
                <div className="flex gap-2 mt-4">
                  <button
                    className="btn btn-primary flex-1 h-12 text-lg font-bold"
                    onClick={handleExecute}
                    disabled={executing || !selectedPrompt || !selectedProfile}
                  >
                    {executing ? (
                      <>
                        <span className="loading loading-spinner"></span>
                        Executing Prompt...
                      </>
                    ) : (
                      <>
                        <PlayIcon className="w-5 h-5 mr-2" />
                        Run Prompt
                      </>
                    )}
                  </button>

                  {selectedProfile && (
                    <button
                      className="btn btn-secondary h-12 px-6"
                      onClick={() => {
                        setInitialMessages([]);
                        setChatMode(true);
                      }}
                      title="Start a direct chat session without running a prompt"
                    >
                      <ChatBubbleLeftRightIcon className="w-5 h-5" />
                      <span className="ml-2">Chat</span>
                    </button>
                  )}
                </div>
              </div>
            </div>

            {/* Output Section */}
            {(result || executing) && (
              <div
                className="flex-1 card flex flex-col p-0 overflow-hidden"
                style={{ borderColor: 'var(--dracula-comment)' }}
              >
                <div
                  className="p-3 border-b flex justify-between items-center"
                  style={{ borderColor: 'var(--dracula-comment)' }}
                >
                  <div className="flex items-center gap-3">
                    <span className="text-xs uppercase tracking-wider" style={{ color: 'var(--dracula-comment)' }}>
                      Output
                    </span>
                    {result && (
                      <button
                        onClick={handleStartChat}
                        className="text-xs px-2 py-1 rounded flex items-center gap-1"
                        style={{
                          background: 'var(--dracula-green)20',
                          color: 'var(--dracula-green)',
                          border: '1px solid var(--dracula-green)50',
                        }}
                      >
                        💬 Start Chat
                      </button>
                    )}
                  </div>
                  {result && (
                    <button
                      onClick={() => navigator.clipboard.writeText(result)}
                      className="text-xs hover:text-terminal-cyan transition-colors"
                      style={{ color: 'var(--dracula-comment)' }}
                    >
                      Copy
                    </button>
                  )}
                </div>

                <div className="flex-1 overflow-hidden" style={{ background: 'var(--dracula-bg)' }}>
                  <UnifiedOutputPanel
                    initialMessages={result ? [{
                      id: `report-stable`,
                      session_id: '',
                      role: 'assistant',
                      content: result,
                      timestamp: new Date().toISOString(),
                    }] : []}
                    contextRef={{
                      prompt_id: selectedPrompt.name,
                      provider: selectedProfile || 'default',
                      model: selectedModel || 'default',
                    }}
                    showInput={false}
                    isLoading={executing}
                    className="h-full"
                  />
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  )
}

export default ThinktankScreen
