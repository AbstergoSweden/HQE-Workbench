import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import { useRepoStore, useScanStore, useReportStore } from '../store'
import { HqeReport, ProviderProfile } from '../types'
import { useToast } from '../context/ToastContext'

export function ScanScreen() {
  const navigate = useNavigate()
  const { path, name } = useRepoStore()
  const { isScanning, phase, progress, setScanning, setPhase, setProgress, reset } = useScanStore()
  const { setReport } = useReportStore()
  const toast = useToast()

  const [localOnly, setLocalOnly] = useState(true)
  const [maxFiles, setMaxFiles] = useState(40)
  const [profiles, setProfiles] = useState<ProviderProfile[]>([])
  const [selectedProfile, setSelectedProfile] = useState<string>('')
  const [loadingProfiles, setLoadingProfiles] = useState(false)
  const [veniceParameters, setVeniceParameters] = useState('')
  const [parallelToolCalls, setParallelToolCalls] = useState<'default' | 'true' | 'false'>('default')

  const selectedProfileInfo = profiles.find((p) => p.name === selectedProfile)
  const isVeniceProfile = Boolean(
    selectedProfileInfo &&
    (selectedProfileInfo.provider_kind === 'venice' ||
      selectedProfileInfo.base_url.toLowerCase().includes('venice.ai'))
  )

  useEffect(() => {
    if (localOnly) return
    const loadProfiles = async () => {
      setLoadingProfiles(true)
      try {
        const result = await invoke<ProviderProfile[]>('list_provider_profiles')
        setProfiles(result ?? [])
        if (result && result.length > 0) {
          setSelectedProfile((current) => current || result[0].name)
        }
      } catch (error) {
        console.error('Failed to load profiles:', error)
        toast.error('Failed to load provider profiles')
      }
      setLoadingProfiles(false)
    }
    loadProfiles()
  }, [localOnly, toast])

  const handleScan = async () => {
    if (!path) return

    setScanning(true)
    reset()

    try {
      setPhase('ingesting repository...')
      setProgress(10)

      let veniceParamsValue: Record<string, unknown> | null = null
      if (!localOnly && isVeniceProfile && veniceParameters.trim() !== '') {
        try {
          const parsed = JSON.parse(veniceParameters)
          if (parsed && typeof parsed === 'object') {
            veniceParamsValue = parsed
          } else {
            throw new Error('Venice parameters must be a JSON object')
          }
        } catch (error) {
          const message = error instanceof Error ? error.message : 'Invalid JSON'
          toast.error(`Invalid Venice parameters: ${message}`)
          setScanning(false)
          return
        }
      }

      const config = {
        llm_enabled: !localOnly,
        provider_profile: localOnly ? null : (selectedProfile || null),
        limits: {
          max_files_sent: maxFiles,
          max_total_chars_sent: 250000,
          snippet_chars: 4000,
        },
        local_only: localOnly,
        timeout_seconds: 120,
        venice_parameters: !localOnly && isVeniceProfile ? veniceParamsValue : null,
        parallel_tool_calls:
          !localOnly && isVeniceProfile && parallelToolCalls !== 'default'
            ? parallelToolCalls === 'true'
            : null,
      }

      setPhase('analyzing code with local heuristics...')
      setProgress(40)

      const report = await invoke<HqeReport>('scan_repo', {
        repo_path: path,
        config,
      })

      setPhase('generating artifacts...')
      setProgress(80)

      setReport(report)

      setProgress(100)
      setTimeout(() => {
        setScanning(false)
        navigate('/report')
      }, 500)

    } catch (error) {
      console.error('Scan failed:', error)
      toast.error('Scan failed. Please check logs for details.')
      setScanning(false)
    }
  }

  if (!path) {
    return (
      <div className="flex items-center justify-center h-full">
        <div 
          className="card p-6 text-center"
          style={{ borderColor: 'var(--dracula-comment)' }}
        >
          <div className="text-terminal-red mb-4 text-lg">✗ error</div>
          <p className="text-sm mb-4" style={{ color: 'var(--dracula-comment)' }}>
            No repository selected
          </p>
          <button
            onClick={() => navigate('/')}
            className="btn btn-primary"
          >
            <span>❯</span> go_home
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className="max-w-4xl mx-auto space-y-4">
      {/* Header */}
      <div className="flex items-center gap-2 mb-6">
        <span className="text-terminal-green">❯</span>
        <h1 className="text-lg font-bold" style={{ color: 'var(--dracula-fg)' }}>
          scan_repository
        </h1>
      </div>

      {/* Repository Info */}
      <div 
        className="card p-4"
        style={{ borderColor: 'var(--dracula-comment)' }}
      >
        <div className="flex items-center gap-2 mb-3 text-xs uppercase tracking-wider" style={{ color: 'var(--dracula-comment)' }}>
          <span>📁</span>
          Target Repository
        </div>
        <div className="flex items-center gap-3 p-3" style={{ background: 'var(--dracula-bg)' }}>
          <span className="text-terminal-cyan text-lg">📂</span>
          <div className="flex-1 min-w-0">
            <p className="font-mono text-sm truncate" style={{ color: 'var(--dracula-fg)' }}>
              {name}
            </p>
            <p className="font-mono text-xs truncate" style={{ color: 'var(--dracula-comment)' }}>
              {path}
            </p>
          </div>
        </div>
      </div>

      {/* Scan Configuration */}
      <div 
        className="card p-4"
        style={{ borderColor: 'var(--dracula-comment)' }}
      >
        <div className="flex items-center gap-2 mb-4 text-xs uppercase tracking-wider" style={{ color: 'var(--dracula-comment)' }}>
          <span>⚙</span>
          Configuration
        </div>

        <div className="space-y-4">
          {/* Local Only Toggle */}
          <label className="flex items-start gap-3 cursor-pointer p-3 transition-colors hover:bg-opacity-50" style={{ background: 'var(--dracula-bg)' }}>
            <input
              type="checkbox"
              checked={localOnly}
              onChange={(e) => setLocalOnly(e.target.checked)}
            />
            <div className="flex-1">
              <div className="flex items-center gap-2">
                <span className="text-terminal-green font-mono text-sm">--local-only</span>
                <span className="text-xs px-2 py-0.5 rounded" style={{ background: 'var(--dracula-current-line)', color: 'var(--dracula-green)' }}>
                  recommended
                </span>
              </div>
              <p className="text-xs mt-1" style={{ color: 'var(--dracula-comment)' }}>
                No data sent to external providers. Uses local heuristics only.
              </p>
            </div>
          </label>

          {/* Max Files Slider */}
          <div className="p-3" style={{ background: 'var(--dracula-bg)' }}>
            <div className="flex items-center justify-between mb-2">
              <label className="text-terminal-cyan font-mono text-sm">
                --max-files
              </label>
              <span className="font-mono text-sm" style={{ color: 'var(--dracula-fg)' }}>
                {maxFiles}
              </span>
            </div>
            <input
              type="range"
              min="10"
              max="100"
              value={maxFiles}
              onChange={(e) => setMaxFiles(parseInt(e.target.value))}
            />
          </div>

          {/* Provider Profile */}
          {!localOnly && (
            <div className="p-3" style={{ background: 'var(--dracula-bg)' }}>
              <label className="text-terminal-purple font-mono text-sm block mb-2">
                --provider
              </label>
              {loadingProfiles ? (
                <div className="flex items-center gap-2 text-sm" style={{ color: 'var(--dracula-comment)' }}>
                  <span className="animate-spin">⟳</span>
                  loading profiles...
                </div>
              ) : profiles.length > 0 ? (
                <select
                  value={selectedProfile}
                  onChange={(e) => setSelectedProfile(e.target.value)}
                  className="input"
                >
                  {profiles.map((p) => (
                    <option key={p.name} value={p.name}>
                      {p.name} ({p.default_model})
                    </option>
                  ))}
                </select>
              ) : (
                <p className="text-sm" style={{ color: 'var(--dracula-comment)' }}>
                  No provider profiles found. Add one in Settings.
                </p>
              )}
            </div>
          )}

          {/* Venice Options */}
          {!localOnly && isVeniceProfile && (
            <div className="p-3 space-y-3" style={{ background: 'var(--dracula-bg)' }}>
              <div className="text-terminal-orange font-mono text-sm">
                # Venice Advanced Options
              </div>

              <div>
                <label className="text-xs block mb-1" style={{ color: 'var(--dracula-comment)' }}>
                  --venice-params (JSON)
                </label>
                <textarea
                  value={veniceParameters}
                  onChange={(e) => setVeniceParameters(e.target.value)}
                  placeholder='{"enable_web_search": "off", "include_venice_system_prompt": true}'
                  className="input min-h-[80px] text-xs"
                />
              </div>

              <div>
                <label className="text-xs block mb-1" style={{ color: 'var(--dracula-comment)' }}>
                  --parallel-tool-calls
                </label>
                <select
                  value={parallelToolCalls}
                  onChange={(e) => setParallelToolCalls(e.target.value as 'default' | 'true' | 'false')}
                  className="input"
                >
                  <option value="default">default</option>
                  <option value="true">true</option>
                  <option value="false">false</option>
                </select>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Scan Progress or Start Button */}
      {isScanning ? (
        <div 
          className="card p-4"
          style={{ borderColor: 'var(--dracula-comment)' }}
        >
          <div className="flex items-center gap-3 mb-4">
            <span className="text-terminal-cyan animate-pulse">⟳</span>
            <div className="flex-1">
              <p className="font-mono text-sm" style={{ color: 'var(--dracula-fg)' }}>
                scanning...
              </p>
              <p className="font-mono text-xs" style={{ color: 'var(--dracula-comment)' }}>
                {phase}
              </p>
            </div>
            <span className="font-mono text-sm text-terminal-green">{progress}%</span>
          </div>
          
          {/* Progress Bar */}
          <div 
            className="h-2 w-full"
            style={{ background: 'var(--dracula-bg)' }}
          >
            <div
              className="h-full transition-all duration-300"
              style={{ 
                width: `${progress}%`,
                background: 'linear-gradient(90deg, var(--dracula-green), var(--dracula-cyan))'
              }}
            />
          </div>
          
          {/* Terminal Output Lines */}
          <div className="mt-4 font-mono text-xs space-y-1" style={{ color: 'var(--dracula-comment)' }}>
            <div className="flex gap-2">
              <span>[{progress >= 10 ? '✓' : ' '}]</span>
              <span style={{ color: progress >= 10 ? 'var(--dracula-green)' : undefined }}>ingest repository</span>
            </div>
            <div className="flex gap-2">
              <span>[{progress >= 40 ? '✓' : ' '}]</span>
              <span style={{ color: progress >= 40 ? 'var(--dracula-green)' : undefined }}>local heuristics</span>
            </div>
            <div className="flex gap-2">
              <span>[{progress >= 80 ? '✓' : ' '}]</span>
              <span style={{ color: progress >= 80 ? 'var(--dracula-green)' : undefined }}>generate artifacts</span>
            </div>
          </div>
        </div>
      ) : (
        <button
          onClick={handleScan}
          disabled={!localOnly && !selectedProfile}
          className="btn btn-primary w-full py-3 flex items-center justify-center gap-2"
        >
          <span className="text-terminal-green">❯</span>
          <span>execute_scan</span>
        </button>
      )}
    </div>
  )
}
