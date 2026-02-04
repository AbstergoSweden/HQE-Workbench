import { useCallback, useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useToast } from '../context/ToastContext'
import { ProviderModelList, ProviderProfile, ProviderModel, ProviderSpec } from '../types'

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
  const [modelsDiscovered, setModelsDiscovered] = useState(false)
  const [saving, setSaving] = useState(false)
  const [testing, setTesting] = useState(false)
  const [discovering, setDiscovering] = useState(false)
  const [validating, setValidating] = useState(false)
  const [discoverError, setDiscoverError] = useState<string | null>(null)
  const [testResult, setTestResult] = useState<boolean | null>(null)
  const [keyLocked, setKeyLocked] = useState(true) // When locked, key is persisted to secure storage
  const [providerSpecs, setProviderSpecs] = useState<ProviderSpec[] | null>(null)
  const [selectedSpec, setSelectedSpec] = useState<string>('')
  const toast = useToast()

  // Load provider specs on mount
  useEffect(() => {
    const loadSpecs = async () => {
      try {
        const specs = await invoke<ProviderSpec[]>('get_provider_specs')
        setProviderSpecs(specs)
      } catch (error) {
        console.error('Failed to load provider specs:', error)
      }
    }
    loadSpecs()
  }, [])

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
    void loadProfiles()
  }, [loadProfiles])

  // Auto-generate default headers based on URL
  useEffect(() => {
    if (!url) return
    // Only auto-generate if headers are empty or unset
    if (headersText.trim()) return

    const defaultHeaders: Record<string, string> = {
      'Content-Type': 'application/json'
    }

    // Add provider-specific headers based on URL
    if (url.includes('anthropic.com')) {
      defaultHeaders['anthropic-version'] = '2023-06-01'
    } else if (url.includes('openrouter.ai')) {
      defaultHeaders['HTTP-Referer'] = window.location.origin
      defaultHeaders['X-Title'] = 'HQE Workbench'
    }

    setHeadersText(JSON.stringify(defaultHeaders, null, 2))
    // eslint-disable-next-line react-hooks/exhaustive-deps -- Only trigger on URL change
  }, [url])

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
    setModelsDiscovered(false)
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
    if (!url) {
      toast.error('URL is required for discovery')
      return
    }
    const keyToUse = key || storedApiKey || ''
    if (!keyToUse) {
      toast.error('API key is required for discovery')
      return
    }
    setDiscovering(true)
    setDiscoverError(null)
    setAvailableModels([])
    setModelsDiscovered(false)
    try {
      if (!name) {
        toast.error('Profile name is required')
        setDiscovering(false)
        return
      }
      if (!keyLocked && keyToUse) {
        await invoke('set_session_api_key', {
          profileName: name,
          apiKey: keyToUse,
        })
      }
      const result = await invoke<ProviderModelList>('discover_models', {
        profileName: name,
      })
      if (result.models.length === 0) {
        setDiscoverError('No models discovered')
        setModelsDiscovered(false)
      } else {
        setAvailableModels(result.models)
        setModelsDiscovered(true)
        // Auto-select first model if none selected
        if (!model && result.models[0]) {
          setModel(result.models[0].id)
        }
        toast.success(`Discovered ${result.models.length} model(s)`)
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      setModelsDiscovered(false)
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
      if (!keyToUse) {
        toast.error('API key is required for testing')
        setTesting(false)
        return
      }
      if (!keyLocked && keyToUse) {
        await invoke('set_session_api_key', {
          profileName: name,
          apiKey: keyToUse,
        })
      }
      const result = await invoke<boolean>('test_provider_connection', {
        profileName: name,
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
      if (!keyLocked && key) {
        await invoke('set_session_api_key', {
          profileName: name,
          apiKey: key,
        })
      }
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
        // Only persist API key if lock is enabled
        apiKey: keyLocked && key ? key : null,
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

  // Default profiles to import
  const defaultProfiles = [
    { name: 'openai', base_url: 'https://api.openai.com/v1', default_model: 'gpt-4o-mini', headers: { 'Content-Type': 'application/json' }, timeout_s: 60 },
    { name: 'anthropic', base_url: 'https://api.anthropic.com/v1', default_model: 'claude-3-5-sonnet-latest', headers: { 'Content-Type': 'application/json', 'anthropic-version': '2023-06-01' }, timeout_s: 60 },
    { name: 'venice', base_url: 'https://api.venice.ai/api/v1', default_model: 'deepseek-r1-671b', headers: { 'Content-Type': 'application/json' }, timeout_s: 120 },
    { name: 'openrouter', base_url: 'https://openrouter.ai/api/v1', default_model: 'openai/gpt-4o-mini', headers: { 'Content-Type': 'application/json', 'HTTP-Referer': 'https://hqe-workbench.local', 'X-Title': 'HQE Workbench' }, timeout_s: 120 },
    { name: 'xai-grok', base_url: 'https://api.x.ai/v1', default_model: 'grok-2-latest', headers: { 'Content-Type': 'application/json' }, timeout_s: 60 },
    { name: 'google-gemini', base_url: 'https://generativelanguage.googleapis.com/v1beta/openai', default_model: 'gemini-2.0-flash', headers: { 'Content-Type': 'application/json' }, timeout_s: 60 },
  ]

  const handleLoadDefaults = async () => {
    try {
      const imported = await invoke<number>('import_default_profiles', { profiles: defaultProfiles })
      if (imported > 0) {
        toast.success(`Imported ${imported} default profile(s)`)
        await loadProfiles()
      } else {
        toast.info('All default profiles already exist')
      }
    } catch (error) {
      console.error('Failed to load defaults:', error)
      toast.error('Failed to import default profiles')
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

          {profiles.length === 0 && (
            <button
              onClick={handleLoadDefaults}
              className="btn w-full mt-2 text-sm"
              style={{ borderColor: 'var(--dracula-cyan)' }}
            >
              <span className="text-terminal-cyan">⬇</span> load_defaults
            </button>
          )}
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
            {/* Provider Spec Selector */}
            <div>
              <label className="text-terminal-cyan font-mono text-sm block mb-1">
                --provider_template
              </label>
              <select
                value={selectedSpec}
                onChange={(e) => {
                  const specId = e.target.value
                  setSelectedSpec(specId)
                  if (specId && providerSpecs) {
                    const spec = providerSpecs.find(s => s.id === specId)
                    if (spec) {
                      setName(spec.display_name.toLowerCase().replace(/\s+/g, '_'))
                      setUrl(spec.base_url)
                      setModel(spec.default_model)
                      setTimeout(spec.recommended_timeout_s)
                      setHeadersText(JSON.stringify(spec.default_headers, null, 2))
                      setModelsDiscovered(false)
                      setAvailableModels([])
                      toast.success(`Loaded ${spec.display_name} template`)
                    }
                  }
                }}
                className="input"
              >
                <option value="">Select a provider template...</option>
                {providerSpecs?.map((spec) => (
                  <option key={spec.id} value={spec.id}>
                    {spec.display_name}
                  </option>
                ))}
              </select>
              {selectedSpec && (
                <>
                  <p className="text-xs mt-1" style={{ color: 'var(--dracula-comment)' }}>
                    Template loaded. Paste your API key and click Discover.
                  </p>
                  {(providerSpecs?.find(s => s.id === selectedSpec)?.quirks.length || 0) > 0 && (
                    <div className="mt-2 p-2 rounded text-xs" style={{ background: 'var(--dracula-bg)' }}>
                      <span style={{ color: 'var(--dracula-yellow)' }}>⚠ Notes:</span>
                      <ul className="mt-1 ml-4 list-disc">
                        {providerSpecs?.find(s => s.id === selectedSpec)?.quirks.map((quirk, i) => (
                          <li key={i} style={{ color: 'var(--dracula-comment)' }}>{quirk}</li>
                        ))}
                      </ul>
                    </div>
                  )}
                </>
              )}
            </div>

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
                <span
                  className="text-xs ml-2 cursor-pointer"
                  style={{ color: keyLocked ? 'var(--dracula-green)' : 'var(--dracula-comment)' }}
                  onClick={() => setKeyLocked(!keyLocked)}
                  title={keyLocked ? 'Key will be saved to secure storage' : 'Key is session-only (not saved)'}
                >
                  {keyLocked ? '🔒' : '🔓'}
                </span>
              </label>
              <input
                type="password"
                value={key}
                onChange={(e) => setKey(e.target.value)}
                placeholder={storedApiKey ? '••••••••' : 'sk-...'}
                className="input"
              />
              {!keyLocked && key && (
                <p className="text-xs mt-1" style={{ color: 'var(--dracula-orange)' }}>
                  Key is session-only and won&apos;t be persisted
                </p>
              )}
              {(key || storedApiKey) && url && (
                <button
                  onClick={async () => {
                    const keyToUse = key || storedApiKey || ''
                    if (!keyToUse || !url) {
                      toast.error('URL and API key are required')
                      return
                    }
                    setValidating(true)
                    try {
                      if (!name) {
                        toast.error('Profile name is required')
                        setValidating(false)
                        return
                      }
                      if (!keyLocked && keyToUse) {
                        await invoke('set_session_api_key', {
                          profileName: name,
                          apiKey: keyToUse,
                        })
                      }
                      await invoke('discover_models', { profileName: name })
                      toast.success('✓ API key is valid')
                    } catch (error) {
                      const msg = error instanceof Error ? error.message : String(error)
                      toast.error(`✗ Invalid API key: ${msg}`)
                    }
                    setValidating(false)
                  }}
                  className="btn text-xs mt-2"
                  disabled={validating}
                  style={{ borderColor: 'var(--dracula-cyan)' }}
                >
                  {validating ? '⟳ validating...' : '✓ validate_key'}
                </button>
              )}
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
                  placeholder={modelsDiscovered ? 'gpt-4o-mini' : '(run discover first)'}
                  disabled={!modelsDiscovered && availableModels.length === 0 && !originalProfileName}
                  className="input flex-1"
                  style={{ opacity: (!modelsDiscovered && availableModels.length === 0 && !originalProfileName) ? 0.5 : 1 }}
                />
                <button
                  onClick={handleDiscoverModels}
                  disabled={discovering || !url || (!key && !storedApiKey)}
                  className="btn text-sm whitespace-nowrap"
                  title={!key && !storedApiKey ? 'Enter API key first' : ''}
                >
                  {discovering ? '...' : 'discover'}
                </button>
              </div>
              {!modelsDiscovered && availableModels.length === 0 && !originalProfileName && (
                <p className="text-xs mt-1" style={{ color: 'var(--dracula-comment)' }}>
                  Click "discover" to load available models
                </p>
              )}
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
