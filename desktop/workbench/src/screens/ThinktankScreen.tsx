import { FC, useCallback, useEffect, useState } from 'react'
import { useLocation } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter'
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism'
import { useToast } from '../context/ToastContext'

interface PromptTool {
  name: string
  description: string
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

export const ThinktankScreen: FC = () => {
  const location = useLocation()
  const [prompts, setPrompts] = useState<PromptTool[]>([])
  const [searchQuery, setSearchQuery] = useState('')
  const [showAgentPrompts, setShowAgentPrompts] = useState(false)
  const [selectedPrompt, setSelectedPrompt] = useState<PromptTool | null>(null)
  const [args, setArgs] = useState<Record<string, unknown>>({})
  const [result, setResult] = useState<string>('')
  const [loading, setLoading] = useState(false)
  const [executing, setExecuting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const toast = useToast()

  const isAgentPrompt = useCallback((name: string) => {
    return name.startsWith('conductor_') || name.startsWith('cli_security_')
  }, [])

  const handleSelectPrompt = useCallback((prompt: PromptTool, initialArgs?: Record<string, unknown>) => {
    setSelectedPrompt(prompt)
    setResult('')
    setError(null)

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
  }, [])

  const loadPrompts = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const loaded = await invoke<PromptTool[]>('get_available_prompts')
      setPrompts(loaded)

      // Check if we have incoming state
      const state = location.state as { promptName?: string; args?: Record<string, unknown> } | null
      if (state?.promptName) {
        const target = loaded.find(p => p.name === state.promptName || `prompts__${p.name}` === state.promptName)
        if (target) {
          if (isAgentPrompt(target.name)) {
            setShowAgentPrompts(true)
          }
          handleSelectPrompt(target, state.args)
        }
      }
    } catch (err) {
      console.error('Failed to load prompts:', err)
      setError(`Failed to load prompt library: ${err}`)
      toast.error('Failed to load prompt library')
    } finally {
      setLoading(false)
    }
  }, [handleSelectPrompt, isAgentPrompt, location.state, toast])

  useEffect(() => {
    loadPrompts()
  }, [loadPrompts])

  const handleExecute = async () => {
    if (!selectedPrompt) return

    setExecuting(true)
    setError(null)
    try {
      const properties = selectedPrompt.input_schema?.properties || {}
      const typedArgs =
        Object.keys(properties).length === 0 ? args : buildTypedArgs(properties, args)
      const response = await invoke<{ result: string }>('execute_prompt', {
        request: {
          tool_name: selectedPrompt.name,
          args: typedArgs,
          profile_name: null // will use default
        }
      })
      setResult(response.result)
    } catch (err) {
      console.error('Execution failed:', err)
      setError(`Execution failed: ${err}`)
      toast.error('Prompt execution failed')
    } finally {
      setExecuting(false)
    }
  }

  const searchFilteredPrompts = prompts.filter(p =>
    p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    p.description.toLowerCase().includes(searchQuery.toLowerCase())
  )
  const visiblePrompts = searchFilteredPrompts.filter((p) => showAgentPrompts || !isAgentPrompt(p.name))
  const hiddenAgentCount =
    showAgentPrompts ? 0 : searchFilteredPrompts.filter((p) => isAgentPrompt(p.name)).length

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
              onClick={loadPrompts}
              className="text-xs p-1 rounded transition-colors hover:text-terminal-cyan"
              disabled={loading}
              style={{ color: 'var(--dracula-comment)' }}
              title="Refresh Library"
            >
              {loading ? <span className="animate-spin">⟳</span> : '↻'}
            </button>
          </div>

          <label className="flex items-center gap-2 text-xs" style={{ color: 'var(--dracula-comment)' }}>
            <input
              type="checkbox"
              checked={showAgentPrompts}
              onChange={(e) => setShowAgentPrompts(e.target.checked)}
            />
            Show agent prompts
          </label>

          <input
            type="text"
            placeholder="Search prompts..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="input text-sm"
          />
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
          ) : prompts.length === 0 ? (
            <div className="p-6 text-center flex flex-col items-center gap-3">
              <div className="text-3xl opacity-50">📭</div>
              <p className="text-sm font-medium" style={{ color: 'var(--dracula-fg)' }}>
                No prompts found
              </p>
              <p className="text-xs" style={{ color: 'var(--dracula-comment)' }}>
                Add <code className="px-1 py-0.5 rounded" style={{ background: 'var(--dracula-current-line)' }}>.yaml</code> files to your <code className="px-1 py-0.5 rounded" style={{ background: 'var(--dracula-current-line)' }}>prompts/</code> folder
              </p>
              <button
                onClick={loadPrompts}
                className="btn text-xs mt-2"
                style={{ borderColor: 'var(--dracula-cyan)' }}
              >
                ↻ refresh
              </button>
            </div>
          ) : visiblePrompts.length === 0 ? (
            <div className="p-6 text-center flex flex-col items-center gap-3">
              <div className="text-3xl opacity-50">🔍</div>
              <p className="text-sm" style={{ color: 'var(--dracula-comment)' }}>
                No prompts match &quot;{searchQuery}&quot;
              </p>
              <button
                onClick={() => setSearchQuery('')}
                className="text-xs text-terminal-cyan hover:underline"
              >
                Clear search
              </button>
            </div>
          ) : (
            <div className="flex flex-col">
              {visiblePrompts.map(p => (
                <button
                  key={p.name}
                  onClick={() => handleSelectPrompt(p)}
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
          {visiblePrompts.length} prompts{hiddenAgentCount > 0 ? ` (${hiddenAgentCount} hidden)` : ''}
        </div>
      </div>

      {/* Main Content: Input & Execution */}
      <div className="flex-1 flex flex-col gap-4 overflow-hidden">
        {!selectedPrompt ? (
          <div
            className="flex-1 flex items-center justify-center card"
            style={{ borderColor: 'var(--dracula-comment)', borderStyle: 'dashed' }}
          >
            <div className="text-center" style={{ color: 'var(--dracula-comment)' }}>
              <div className="text-4xl mb-4">🧠</div>
              <p>Select a prompt from the library to begin</p>
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

              <p className="text-sm mb-4" style={{ color: 'var(--dracula-comment)' }}>
                {selectedPrompt.description}
              </p>

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

                <div className="flex justify-end mt-2">
                  <button
                    onClick={handleExecute}
                    disabled={executing}
                    className="btn btn-primary flex items-center gap-2"
                  >
                    {executing ? (
                      <>
                        <span className="animate-spin">⟳</span>
                        Executing...
                      </>
                    ) : (
                      <>
                        <span className="text-terminal-green">❯</span>
                        Execute Prompt
                      </>
                    )}
                  </button>
                </div>
              </div>
            </div>

            {/* Output Section */}
            {(result || executing || error) && (
              <div
                className="flex-1 card flex flex-col p-0 overflow-hidden"
                style={{ borderColor: 'var(--dracula-comment)' }}
              >
                <div
                  className="p-3 border-b flex justify-between items-center"
                  style={{ borderColor: 'var(--dracula-comment)' }}
                >
                  <span className="text-xs uppercase tracking-wider" style={{ color: 'var(--dracula-comment)' }}>
                    Output
                  </span>
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

                <div className="flex-1 p-4 overflow-auto" style={{ background: 'var(--dracula-bg)' }}>
                  {error ? (
                    <div
                      className="p-4 rounded text-sm"
                      style={{
                        background: 'var(--dracula-red)10',
                        border: '1px solid var(--dracula-red)30',
                        color: 'var(--dracula-red)'
                      }}
                    >
                      ✗ {error}
                    </div>
                  ) : executing ? (
                    <div className="flex flex-col gap-2">
                      {[...Array(4)].map((_, i) => (
                        <div
                          key={i}
                          className="h-4 animate-pulse"
                          style={{
                            background: 'var(--dracula-current-line)',
                            width: `${60 + Math.random() * 40}%`
                          }}
                        />
                      ))}
                    </div>
                  ) : (
                    <div className="prose prose-invert max-w-none prose-sm">
                      <ReactMarkdown
                        remarkPlugins={[remarkGfm]}
                        components={{
                          code({ inline, className, children, ...props }: { inline?: boolean; className?: string; children?: React.ReactNode }) {
                            const match = /language-(\w+)/.exec(className || '')
                            return !inline && match ? (
                              <SyntaxHighlighter
                                {...props}
                                style={vscDarkPlus}
                                language={match[1]}
                                PreTag="div"
                              >
                                {String(children).replace(/\n$/, '')}
                              </SyntaxHighlighter>
                            ) : (
                              <code {...props} className={className}>
                                {children}
                              </code>
                            )
                          }
                        }}
                      >
                        {result}
                      </ReactMarkdown>
                    </div>
                  )}
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  )
}
