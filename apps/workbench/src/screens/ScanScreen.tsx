import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import { useRepoStore, useScanStore, useReportStore } from '../store'
import { HqeReport, ProviderProfile } from '../types'
import { useToast } from '../context/ToastContext'
import { Card } from '../components/ui/Card'

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
      setPhase('Ingesting repository...')
      setProgress(10)

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
      }

      setPhase('Analyzing code with local heuristics...')
      setProgress(40)

      const report = await invoke<HqeReport>('scan_repo', {
        repo_path: path,
        config,
      })

      setPhase('Generating artifacts...')
      setProgress(80)

      setReport(report)

      setProgress(100)
      // UI-PERF-002: Set isScanning false BEFORE navigation to prevent stale UI state
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
        <div className="text-center">
          <p className="text-emerald-200/60 mb-4">
            No repository selected
          </p>
          <button
            onClick={() => navigate('/')}
            className="btn btn-primary"
          >
            Go to Welcome
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <h1 className="text-3xl font-bold mb-8 text-emerald-50">
        Scan Repository
      </h1>

      <Card>
        <h2 className="text-lg font-semibold mb-4 text-emerald-50">
          Repository
        </h2>
        <div className="flex items-center gap-4 p-4 rounded-lg bg-black/20">
          <div className="text-4xl" role="img" aria-label="folder">📁</div>
          <div>
            <p className="font-medium text-emerald-50">
              {name}
            </p>
            <p className="text-sm text-emerald-200/60">
              {path}
            </p>
          </div>
        </div>
      </Card>

      <Card>
        <h2 className="text-lg font-semibold mb-4 text-emerald-50">
          Scan Options
        </h2>

        <div className="space-y-4">
          <label className="flex items-center gap-3 cursor-pointer">
            <input
              type="checkbox"
              checked={localOnly}
              onChange={(e) => setLocalOnly(e.target.checked)}
              className="w-5 h-5 rounded border-emerald-500/50 bg-black/20 text-emerald-500 focus:ring-emerald-500"
            />
            <div>
              <span className="text-emerald-50">
                Local-only mode
              </span>
              <p className="text-sm text-emerald-200/60">
                No data sent to external providers. Uses local heuristics only.
              </p>
            </div>
          </label>

          <div>
            <label className="block mb-2 text-emerald-50">
              Max files to analyze
            </label>
            <input
              type="range"
              min="10"
              max="100"
              value={maxFiles}
              onChange={(e) => setMaxFiles(parseInt(e.target.value))}
              className="w-full accent-emerald-500"
            />
            <p className="text-sm mt-1 text-emerald-200/60">
              {maxFiles} files
            </p>
          </div>

          {!localOnly && (
            <div>
              <label className="block mb-2 text-emerald-50">
                Provider Profile
              </label>
              {loadingProfiles ? (
                <p className="text-sm text-emerald-200/60">Loading profiles...</p>
              ) : profiles.length > 0 ? (
                <select
                  value={selectedProfile}
                  onChange={(e) => setSelectedProfile(e.target.value)}
                  className="input w-full"
                >
                  {profiles.map((p) => (
                    <option key={p.name} value={p.name}>
                      {p.name} ({p.default_model})
                    </option>
                  ))}
                </select>
              ) : (
                <p className="text-sm text-emerald-200/60">
                  No provider profiles found. Add one in Settings.
                </p>
              )}
            </div>
          )}
        </div>
      </Card>

      {isScanning ? (
        <Card>
          <div className="flex items-center gap-4 mb-4">
            <div className="animate-spin text-2xl text-emerald-500" role="img" aria-label="loading">
              ⏳
            </div>
            <div>
              <p className="font-medium text-emerald-50">
                Scanning...
              </p>
              <p className="text-sm text-emerald-200/60">
                {phase}
              </p>
            </div>
          </div>
          <div className="w-full rounded-full h-2 bg-black/20">
            <div
              className="h-2 rounded-full transition-all duration-300 bg-emerald-500"
              style={{ width: `${progress}%` }}
            />
          </div>
        </Card>
      ) : (
        <button
          onClick={handleScan}
          disabled={!localOnly && !selectedProfile}
          className="btn btn-primary w-full py-4 text-lg flex items-center justify-center gap-2"
        >
          <span role="img" aria-label="rocket">🚀</span> Start Scan
        </button>
      )}
    </div>
  )
}
