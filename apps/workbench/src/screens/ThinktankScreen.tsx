import { FC, useCallback, useEffect, useState } from 'react'
import { useLocation } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter'
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism'
import { Card } from '../components/ui/Card'
import { Skeleton } from '../components/ui/Skeleton'
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
  const [selectedPrompt, setSelectedPrompt] = useState<PromptTool | null>(null)
  const [args, setArgs] = useState<Record<string, unknown>>({})
  const [result, setResult] = useState<string>('')
  const [loading, setLoading] = useState(false)
  const [executing, setExecuting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const toast = useToast()

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
  }, [handleSelectPrompt, location.state])

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

  const filteredPrompts = prompts.filter(p =>
    p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    p.description.toLowerCase().includes(searchQuery.toLowerCase())
  )

  return (
    <div className="flex h-full gap-6">
      {/* Sidebar: Prompt List */}
      <Card className="w-1/3 flex flex-col p-0 overflow-hidden">
        <div className="p-4 border-b border-emerald-500/10 flex flex-col gap-3">
          <div className="flex justify-between items-center">
            <span className="font-bold flex items-center gap-2 text-emerald-50">
              <span role="img" aria-label="library">📚</span> Library
            </span>
            <button
              onClick={loadPrompts}
              className="text-xs hover:bg-emerald-500/10 p-1.5 rounded transition-colors text-emerald-200/60"
              disabled={loading}
              title="Refresh Library"
            >
              {loading ? <span role="img" aria-label="loading">⏳</span> : <span role="img" aria-label="refresh">🔃</span>}
            </button>
          </div>
          <input
            type="text"
            placeholder="Search prompts..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full p-2 text-sm rounded bg-black/20 border border-emerald-500/10 focus:border-emerald-500/50 outline-none transition-colors text-emerald-50"
          />
        </div>

        <div className="flex-1 overflow-auto">
          {loading ? (
            <div className="p-4 space-y-4">
              <Skeleton className="h-12 w-full" count={6} />
            </div>
          ) : prompts.length === 0 ? (
            <div className="p-8 text-center text-sm text-emerald-200/40">No prompts found in /prompts</div>
          ) : filteredPrompts.length === 0 ? (
            <div className="p-8 text-center text-sm text-emerald-200/40">No matching prompts</div>
          ) : (
            <div className="flex flex-col">
              {filteredPrompts.map(p => (
                <button
                  key={p.name}
                  onClick={() => handleSelectPrompt(p)}
                  className={`p-4 text-left text-sm border-b border-emerald-500/10 transition-all hover:bg-emerald-500/5 ${selectedPrompt?.name === p.name
                    ? 'bg-emerald-500/10 border-l-4 border-l-emerald-500 pl-[1.25rem]'
                    : 'border-l-4 border-l-transparent'
                    }`}
                >
                  <div className="font-bold mb-1 truncate flex items-center gap-2 text-emerald-50">
                    {p.name.replace(/_/g, ' ')}
                  </div>
                  <div className="text-xs text-emerald-200/60 line-clamp-2">{p.description}</div>
                </button>
              ))}
            </div>
          )}
        </div>
        <div className="p-2 border-t border-emerald-500/10 text-center text-xs text-emerald-200/40 bg-black/10">
          {filteredPrompts.length} prompts available
        </div>
      </Card>

      {/* Main Content: Input & Execution */}
      <div className="flex-1 flex flex-col gap-6">
        {!selectedPrompt ? (
          <div className="flex-1 flex items-center justify-center border-2 border-dashed border-emerald-500/10 rounded-lg">
            <div className="text-center text-emerald-200/40">
              <div className="text-4xl mb-4"><span role="img" aria-label="brain">🧠</span></div>
              <p>Select a prompt from the library to begin</p>
            </div>
          </div>
        ) : (
          <div className="flex-1 flex flex-col gap-6 overflow-hidden">
            {/* Input Section */}
            <Card className="p-6">
              <h2 className="text-lg font-bold mb-4 flex items-center gap-2 text-emerald-50">
                <span>{selectedPrompt.name.replace(/_/g, ' ')}</span>
                <span className="text-xs font-normal text-emerald-200/40">({selectedPrompt.name})</span>
              </h2>
              <p className="text-sm mb-6 text-emerald-200/80">{selectedPrompt.description}</p>

              <div className="flex flex-col gap-4">
                {Object.entries(selectedPrompt.input_schema?.properties || {}).map(([key, schema]) => {
                  const type = getSchemaType(schema)
                  const enumValues = Array.isArray(schema.enum) ? schema.enum : null
                  const value = args[key]
                  const inputId = `prompt-field-${key}`
                  return (
                    <div key={key} className="flex flex-col gap-1">
                      <label
                        htmlFor={inputId}
                        className="text-xs font-bold uppercase tracking-wider text-emerald-200/60"
                      >
                        {key}
                      </label>
                      {enumValues ? (
                        <select
                          id={inputId}
                          value={String(value ?? enumValues[0] ?? '')}
                          onChange={(e) => {
                            const selected = enumValues.find((opt) => String(opt) === e.target.value)
                            setArgs((prev) => ({ ...prev, [key]: selected ?? e.target.value }))
                          }}
                          className="p-3 text-sm rounded border focus:ring-1 focus:ring-emerald-500 outline-none bg-black/20 border-emerald-500/10 text-emerald-50"
                        >
                          {enumValues.map((opt) => (
                            <option key={String(opt)} value={String(opt)}>
                              {String(opt)}
                            </option>
                          ))}
                        </select>
                      ) : type === 'boolean' ? (
                        <div className="inline-flex items-center gap-2 text-sm text-emerald-200/80">
                          <input
                            id={inputId}
                            type="checkbox"
                            checked={Boolean(value)}
                            onChange={(e) => setArgs((prev) => ({ ...prev, [key]: e.target.checked }))}
                            className="w-4 h-4 rounded border-emerald-500/50 bg-black/20 text-emerald-500 focus:ring-emerald-500"
                          />
                          {schema.description || `Enable ${key}`}
                        </div>
                      ) : type === 'number' || type === 'integer' ? (
                        <input
                          id={inputId}
                          type="number"
                          step={type === 'integer' ? '1' : 'any'}
                          value={value === undefined || value === null ? '' : String(value)}
                          onChange={(e) => setArgs((prev) => ({ ...prev, [key]: e.target.value }))}
                          className="p-3 text-sm rounded border focus:ring-1 focus:ring-emerald-500 outline-none bg-black/20 border-emerald-500/10 text-emerald-50"
                          placeholder={schema.description || `Enter ${key}...`}
                        />
                      ) : type === 'object' || type === 'array' || key === 'args' || (type === 'string' && !schema.description?.includes('short')) ? (
                        <textarea
                          id={inputId}
                          value={value === undefined ? '' : String(value)}
                          onChange={(e) => setArgs((prev) => ({ ...prev, [key]: e.target.value }))}
                          className="p-3 text-sm rounded border min-h-[120px] focus:ring-1 focus:ring-emerald-500 outline-none bg-black/20 border-emerald-500/10 text-emerald-50"
                          placeholder={schema.description || `Enter ${key}...`}
                        />
                      ) : (
                        <input
                          id={inputId}
                          type="text"
                          value={value === undefined ? '' : String(value)}
                          onChange={(e) => setArgs((prev) => ({ ...prev, [key]: e.target.value }))}
                          className="p-3 text-sm rounded border focus:ring-1 focus:ring-emerald-500 outline-none bg-black/20 border-emerald-500/10 text-emerald-50"
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
                    className="btn btn-primary flex items-center gap-2 shadow-lg active:scale-95"
                  >
                    {executing ? (
                      <>
                        <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                        Thinking...
                      </>
                    ) : (
                      <>
                        <span role="img" aria-label="rocket">🚀</span> Run Prompt
                      </>
                    )}
                  </button>
                </div>
              </div>
            </Card>

            {/* Output Section */}
            {(result || executing || error) && (
              <Card className="flex-1 flex flex-col p-0 overflow-hidden">
                <div className="p-3 border-b border-emerald-500/10 text-xs font-bold uppercase tracking-widest flex justify-between items-center bg-black/5 text-emerald-200/60">
                  <span>Results</span>
                  {result && (
                    <button
                      onClick={() => navigator.clipboard.writeText(result)}
                      className="hover:text-emerald-500 transition-colors"
                    >
                      Copy Output
                    </button>
                  )}
                </div>

                <div className="flex-1 p-6 overflow-auto bg-black/20">
                  {error ? (
                    <div className="p-4 rounded bg-red-500/10 border border-red-500/20 text-red-500 text-sm">
                      {error}
                    </div>
                  ) : executing ? (
                    <div className="flex flex-col gap-4">
                      <Skeleton className="h-4 w-3/4" />
                      <Skeleton className="h-4 w-full" />
                      <Skeleton className="h-4 w-5/6" />
                      <Skeleton className="h-4 w-2/3" />
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
              </Card>
            )}
          </div>
        )}
      </div>
    </div>
  )
}
