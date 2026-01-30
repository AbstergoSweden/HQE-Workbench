import { FC, useEffect, useState } from 'react'
import { useLocation } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter'
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism'
import { Card } from '../components/ui/Card'
import { Skeleton } from '../components/ui/Skeleton'

interface PromptTool {
  name: string
  description: string
  input_schema: any
  template: string
}

export const ThinktankScreen: FC = () => {
  const location = useLocation()
  const [prompts, setPrompts] = useState<PromptTool[]>([])
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedPrompt, setSelectedPrompt] = useState<PromptTool | null>(null)
  const [args, setArgs] = useState<Record<string, string>>({})
  const [result, setResult] = useState<string>('')
  const [loading, setLoading] = useState(false)
  const [executing, setExecuting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    loadPrompts()
  }, [])

  const loadPrompts = async () => {
    setLoading(true)
    setError(null)
    try {
      const loaded = await invoke<PromptTool[]>('get_available_prompts')
      setPrompts(loaded)

      // Check if we have incoming state
      if (location.state?.promptName) {
        const target = loaded.find(p => p.name === location.state.promptName || `prompts__${p.name}` === location.state.promptName)
        if (target) {
          handleSelectPrompt(target, location.state.args)
        }
      }
    } catch (err) {
      console.error('Failed to load prompts:', err)
      setError(`Failed to load prompt library: ${err}`)
    } finally {
      setLoading(false)
    }
  }

  const handleSelectPrompt = (prompt: PromptTool, initialArgs?: Record<string, string>) => {
    setSelectedPrompt(prompt)
    setResult('')
    setError(null)

    // Initialize args from schema
    const newArgs: Record<string, string> = {}
    const properties = prompt.input_schema?.properties || {}
    Object.keys(properties).forEach(key => {
      newArgs[key] = initialArgs?.[key] || ''
    })
    setArgs(newArgs)
  }

  const handleExecute = async () => {
    if (!selectedPrompt) return

    setExecuting(true)
    setError(null)
    try {
      const response = await invoke<{ result: string }>('execute_prompt', {
        request: {
          tool_name: selectedPrompt.name,
          args: args,
          profile_name: null // will use default
        }
      })
      setResult(response.result)
    } catch (err) {
      console.error('Execution failed:', err)
      setError(`Execution failed: ${err}`)
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
                {Object.entries(selectedPrompt.input_schema?.properties || {}).map(([key, schema]: [string, any]) => (
                  <div key={key} className="flex flex-col gap-1">
                    <label className="text-xs font-bold uppercase tracking-wider text-emerald-200/60">{key}</label>
                    {key === 'args' || (schema.type === 'string' && !schema.description?.includes('short')) ? (
                      <textarea
                        value={args[key] || ''}
                        onChange={(e) => setArgs({ ...args, [key]: e.target.value })}
                        className="p-3 text-sm rounded border min-h-[120px] focus:ring-1 focus:ring-emerald-500 outline-none bg-black/20 border-emerald-500/10 text-emerald-50"
                        placeholder={schema.description || `Enter ${key}...`}
                      />
                    ) : (
                      <input
                        type="text"
                        value={args[key] || ''}
                        onChange={(e) => setArgs({ ...args, [key]: e.target.value })}
                        className="p-3 text-sm rounded border focus:ring-1 focus:ring-emerald-500 outline-none bg-black/20 border-emerald-500/10 text-emerald-50"
                        placeholder={schema.description || `Enter ${key}...`}
                      />
                    )}
                  </div>
                ))}

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
                          code({ inline, className, children, ...props }: any) {
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
