import { useCallback, useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useToast } from '../context/ToastContext'
import { ProviderModelList, ProviderProfile, ProviderModel } from '../types'

export function SettingsScreen() {
  const [profiles, setProfiles] = useState<ProviderProfile[]>([])
  const [selectedProfile, setSelectedProfile] = useState<string>('')
  const [loadingProfiles, setLoadingProfiles] = useState(false)
  const [originalProfileName, setOriginalProfileName] = useState<string | null>(null)
  const [storedApiKey, setStoredApiKey] = useState<string | null>(null)
  const [name, setName] = useState('')
  const [url, setUrl] = useState('')
  const [key, setKey] = useState('')
  const [model, setModel] = useState('gpt-4o-mini')
  const [headersText, setHeadersText] = useState('')
  const [headersError, setHeadersError] = useState<string | null>(null)
  const [organization, setOrganization] = useState('')
  const [project, setProject] = useState('')
  const [timeout, setTimeout] = useState(60)
  const [availableModels, setAvailableModels] = useState<ProviderModel[]>([])
  const [saving, setSaving] = useState(false)
  const [testing, setTesting] = useState(false)
  const [discovering, setDiscovering] = useState(false)
  const [discoverError, setDiscoverError] = useState<string | null>(null)
  const [testResult, setTestResult] = useState<boolean | null>(null)
  const toast = useToast()

  const loadProfiles = useCallback(async () => {
    setLoadingProfiles(true)
    try {
      const result = await invoke<ProviderProfile[]>('list_provider_profiles')
      setProfiles(result ?? [])
    } catch (error) {
      console.error('Failed to load profiles:', error)
      toast.error('Failed to load provider profiles')
    }
    setLoadingProfiles(false)
  }, [toast])

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- This is a standard fetch-on-mount pattern with async callback
    void loadProfiles()
  }, [loadProfiles])

  const resetForm = () => {
    setSelectedProfile('')
    setOriginalProfileName(null)
    setStoredApiKey(null)
    setName('')
    setUrl('')
    setKey('')
    setModel('gpt-4o-mini')
    setHeadersText('')
    setHeadersError(null)
    setOrganization('')
    setProject('')
    setTimeout(60)
    setAvailableModels([])
    setDiscoverError(null)
    setTestResult(null)
  }

  const parseHeadersInput = () => {
    if (!headersText.trim()) {
      return {}
    }
    try {
      const parsed = JSON.parse(headersText)
      if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
        return null
      }
      const out: Record<string, string> = {}
      for (const [key, value] of Object.entries(parsed)) {
        if (typeof value !== 'string') {
          return null
        }
        out[key] = value
      }
      return out
    } catch {
      return null
    }
  }

  const handleSelectProfile = async (profileName: string) => {
    const profile = profiles.find((p) => p.name === profileName)
    if (!profile) return
    setSelectedProfile(profile.name)
    setOriginalProfileName(profile.name)
    setName(profile.name)
    setUrl(profile.base_url)
    setModel(profile.default_model)
    setOrganization(profile.organization ?? '')
    setProject(profile.project ?? '')
    setTimeout(profile.timeout_s ?? 60)
    setHeadersText(JSON.stringify(profile.headers ?? {}, null, 2))
    setHeadersError(null)
    setTestResult(null)
    try {
      const apiKeyId = `api_key:${profile.name}`
      const stored = await invoke<string | null>('get_api_key', { apiKeyId })
      setStoredApiKey(stored)
    } catch {
      setStoredApiKey(null)
    }
  }

  const handleDiscoverModels = async () => {
    if (!url) return
    setDiscovering(true)
    setDiscoverError(null)
    setAvailableModels([])
    try {
      const keyToUse = key || storedApiKey || ''
      const result = await invoke<ProviderModelList>('discover_models', {
        baseUrl: url,
        apiKey: keyToUse,
      })
      if (result.models.length === 0) {
        setDiscoverError('No models discovered')
      } else {
        setAvailableModels(result.models)
        if (!model && result.models[0]) {
          setModel(result.models[0].id)
        }
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      setDiscoverError(message)
    }
    setDiscovering(false)
  }

  const handleTestProfile = async () => {
    if (!name || !url) return
    setTesting(true)
    setTestResult(null)
    try {
      const keyToUse = key || storedApiKey || ''
      const result = await invoke<boolean>('test_provider', {
        name,
        baseUrl: url,
        apiKey: keyToUse,
        defaultModel: model || 'gpt-4o-mini',
      })
      setTestResult(result)
      toast.success(result ? 'Connection successful' : 'Connection failed')
    } catch (error) {
      console.error('Test failed:', error)
      setTestResult(false)
      toast.error('Test failed')
    }
    setTesting(false)
  }

  const handleSave = async () => {
    if (!name || !url) {
      toast.error('Name and URL are required')
      return
    }
    const parsedHeaders = parseHeadersInput()
    if (parsedHeaders === null) {
      setHeadersError('Invalid JSON format')
      toast.error('Invalid headers JSON')
      return
    }
    setHeadersError(null)
    setSaving(true)
    try {
      await invoke('save_provider_profile', {
        originalName: originalProfileName,
        profile: {
          name,
          base_url: url,
          default_model: model || 'gpt-4o-mini',
          headers: parsedHeaders,
          organization: organization || null,
          project: project || null,
          timeout_s: timeout,
        },
        apiKey: key || null,
      })
      toast.success(originalProfileName ? 'Profile updated' : 'Profile created')
      await loadProfiles()
      resetForm()
    } catch (error) {
      console.error('Save failed:', error)
      toast.error('Failed to save profile')
    }
    setSaving(false)
  }

  const handleDelete = async () => {
    if (!originalProfileName) return
    if (!window.confirm(`Delete profile "${originalProfileName}"?`)) return
    try {
      await invoke('delete_provider_profile', { name: originalProfileName })
      toast.success('Profile deleted')
      await loadProfiles()
      resetForm()
    } catch (error) {
      console.error('Delete failed:', error)
      toast.error('Failed to delete profile')
    }
  }

  return (
    <div className="max-w-4xl mx-auto space-y-4">
      {/* Header */}
      <div className="flex items-center gap-2 mb-6">
        <span className="text-terminal-green">❯</span>
        <h1 className="text-lg font-bold" style={{ color: 'var(--dracula-fg)' }}>
          configure_providers
        </h1>
      </div>

      <div className="grid grid-cols-3 gap-4">
        {/* Profile List */}
        <div
          className="card p-4 col-span-1"
          style={{ borderColor: 'var(--dracula-comment)' }}
        >
          <div className="flex items-center gap-2 mb-3 text-xs uppercase tracking-wider" style={{ color: 'var(--dracula-comment)' }}>
            <span>📋</span>
            Profiles
          </div>

          {loadingProfiles ? (
            <div className="flex items-center gap-2 text-sm" style={{ color: 'var(--dracula-comment)' }}>
              <span className="animate-spin">⟳</span>
              loading...
            </div>
          ) : profiles.length === 0 ? (
            <p className="text-sm" style={{ color: 'var(--dracula-comment)' }}>
              No profiles configured
            </p>
          ) : (
            <div className="space-y-1">
              {profiles.map((p) => (
                <button
                  key={p.name}
                  onClick={() => handleSelectProfile(p.name)}
                  className="w-full text-left px-3 py-2 text-sm transition-all duration-150"
                  style={{
                    backgroundColor: selectedProfile === p.name
                      ? 'var(--dracula-bg)'
                      : 'transparent',
                    borderLeft: selectedProfile === p.name
                      ? '2px solid var(--dracula-green)'
                      : '2px solid transparent',
                    color: selectedProfile === p.name
                      ? 'var(--dracula-green)'
                      : 'var(--dracula-fg)',
                  }}
                >
                  <div className="flex items-center gap-2">
                    <span>{selectedProfile === p.name ? '❯' : '$'}</span>
                    <span className="truncate">{p.name}</span>
                  </div>
                </button>
              ))}
            </div>
          )}

          <button
            onClick={resetForm}
            className="btn w-full mt-4 text-sm"
          >
            <span className="text-terminal-purple">+</span> new_profile
          </button>
        </div>

        {/* Profile Form */}
        <div
          className="card p-4 col-span-2"
          style={{ borderColor: 'var(--dracula-comment)' }}
        >
          <div className="flex items-center gap-2 mb-4 text-xs uppercase tracking-wider" style={{ color: 'var(--dracula-comment)' }}>
            <span>⚙</span>
            {originalProfileName ? 'edit_profile' : 'new_profile'}
          </div>

          <div className="space-y-4">
            {/* Name */}
            <div>
              <label className="text-terminal-cyan font-mono text-sm block mb-1">
                --name
              </label>
              <input
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="my-provider"
                className="input"
              />
            </div>

            {/* URL */}
            <div>
              <label className="text-terminal-cyan font-mono text-sm block mb-1">
                --url
              </label>
              <input
                type="text"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                placeholder="https://api.openai.com/v1"
                className="input"
              />
            </div>

            {/* API Key */}
            <div>
              <label className="text-terminal-cyan font-mono text-sm block mb-1">
                --api-key
                {storedApiKey && (
                  <span className="text-terminal-green text-xs ml-2">(stored)</span>
                )}
              </label>
              <input
                type="password"
                value={key}
                onChange={(e) => setKey(e.target.value)}
                placeholder={storedApiKey ? '••••••••' : 'sk-...'}
                className="input"
              />
            </div>

            {/* Model */}
            <div>
              <label className="text-terminal-cyan font-mono text-sm block mb-1">
                --model
              </label>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={model}
                  onChange={(e) => setModel(e.target.value)}
                  placeholder="gpt-4o-mini"
                  className="input flex-1"
                />
                <button
                  onClick={handleDiscoverModels}
                  disabled={discovering || !url}
                  className="btn text-sm whitespace-nowrap"
                >
                  {discovering ? '...' : 'discover'}
                </button>
              </div>
              {discoverError && (
                <p className="text-xs mt-1 text-terminal-red">{discoverError}</p>
              )}
              {availableModels.length > 0 && (
                <select
                  value={model}
                  onChange={(e) => setModel(e.target.value)}
                  className="input mt-2"
                >
                  {availableModels.map((m) => (
                    <option key={m.id} value={m.id}>
                      {m.name || m.id} {m.traits?.supports_response_schema ? '(schema)' : ''}
                    </option>
                  ))}
                </select>
              )}
            </div>

            {/* Headers */}
            <div>
              <label className="text-terminal-cyan font-mono text-sm block mb-1">
                --headers
              </label>
              <textarea
                value={headersText}
                onChange={(e) => {
                  setHeadersText(e.target.value)
                  setHeadersError(null)
                }}
                placeholder='{"X-Custom-Header": "value"}'
                className="input min-h-[80px] text-xs"
              />
              {headersError && (
                <p className="text-xs mt-1 text-terminal-red">{headersError}</p>
              )}
            </div>

            {/* Timeout */}
            <div>
              <label className="text-terminal-cyan font-mono text-sm block mb-1">
                --timeout
              </label>
              <div className="flex items-center gap-3">
                <input
                  type="range"
                  min="10"
                  max="300"
                  value={timeout}
                  onChange={(e) => setTimeout(parseInt(e.target.value))}
                  className="flex-1"
                />
                <span className="font-mono text-sm w-16 text-right">{timeout}s</span>
              </div>
            </div>

            {/* Action Buttons */}
            <div className="flex gap-2 pt-4 border-t" style={{ borderColor: 'var(--dracula-comment)' }}>
              <button
                onClick={handleSave}
                disabled={saving || !name || !url}
                className="btn btn-primary flex-1"
              >
                <span className="text-terminal-green">❯</span>
                {saving ? 'saving...' : (originalProfileName ? 'update' : 'create')}
              </button>

              <button
                onClick={handleTestProfile}
                disabled={testing || !name || !url}
                className="btn"
              >
                {testing ? '...' : 'test'}
              </button>

              {originalProfileName && (
                <button
                  onClick={handleDelete}
                  className="btn btn-danger"
                >
                  delete
                </button>
              )}
            </div>

            {testResult !== null && (
              <div className={`text-sm ${testResult ? 'text-terminal-green' : 'text-terminal-red'}`}>
                {testResult ? '✓ connection successful' : '✗ connection failed'}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
