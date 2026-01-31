import { useCallback, useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useToast } from '../context/ToastContext'
import { Card } from '../components/ui/Card'
import { ProviderModelList, ProviderProfile } from '../types'

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
  const [availableModels, setAvailableModels] = useState<{ id: string; name: string; supports_schema: boolean }[]>([])
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
    // Loading profiles is an external sync; allow state updates in this effect.
    // eslint-disable-next-line react-hooks/set-state-in-effect
    loadProfiles()
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
    setKey('')
    setAvailableModels([])
    setDiscoverError(null)
    setTestResult(null)
    setStoredApiKey(null)

    try {
      const result = await invoke<[ProviderProfile, string | null] | null>('get_provider_profile', {
        name: profile.name,
      })
      if (result) {
        const [fullProfile, apiKey] = result
        setHeadersText(JSON.stringify(fullProfile.headers ?? {}, null, 2))
        setOrganization(fullProfile.organization ?? '')
        setProject(fullProfile.project ?? '')
        setTimeout(fullProfile.timeout_s ?? 60)
        setStoredApiKey(apiKey ?? null)
      }
    } catch (error) {
      console.error('Failed to load provider key:', error)
      toast.error('Failed to load stored API key')
    }
  }

  const handleSave = async () => {
    if (!name || !url) return

    setSaving(true)
    try {
      const existingProfile = originalProfileName
        ? profiles.find((p) => p.name === originalProfileName)
        : null
      const apiKeyToSave =
        key.trim() !== ''
          ? key
          : originalProfileName && originalProfileName !== name
            ? storedApiKey
            : null

      const parsedHeaders = parseHeadersInput()
      if (parsedHeaders === null) {
        setHeadersError('Headers must be valid JSON object with string values')
        toast.error('Invalid headers JSON')
        setSaving(false)
        return
      }

      const profile = {
        name,
        base_url: url,
        api_key_id: `api_key:${name}`,
        default_model: model,
        headers: parsedHeaders,
        organization: organization.trim() !== '' ? organization.trim() : null,
        project: project.trim() !== '' ? project.trim() : null,
        provider_kind: existingProfile?.provider_kind ?? null,
        timeout_s: timeout,
      }

      await invoke('save_provider_profile', { profile, api_key: apiKeyToSave })
      if (originalProfileName && originalProfileName !== name) {
        await invoke('delete_provider_profile', { name: originalProfileName })
      }
      setSelectedProfile(name)
      setOriginalProfileName(name)
      if (apiKeyToSave) {
        setStoredApiKey(apiKeyToSave)
      }
      await loadProfiles()
      toast.success('Provider saved successfully!')
    } catch (error) {
      console.error('Failed to save:', error)
      toast.error('Failed to save provider configuration')
    }
    setSaving(false)
  }

  const handleDiscoverModels = async () => {
    if (!url) return
    setDiscovering(true)
    setDiscoverError(null)
    try {
      const parsedHeaders = parseHeadersInput()
      if (parsedHeaders === null) {
        setHeadersError('Headers must be valid JSON object with string values')
        toast.error('Invalid headers JSON')
        setDiscovering(false)
        return
      }

      const apiKeyForDiscovery = key.trim() !== '' ? key : storedApiKey ?? ''
      const result = await invoke<ProviderModelList>('discover_models', {
        input: {
          base_url: url,
          headers: parsedHeaders,
          api_key: apiKeyForDiscovery,
          timeout_s: timeout,
          no_cache: false,
        }
      })
      const models = (result?.models ?? []).map((m) => ({
        id: m.id,
        name: m.name ?? m.id,
        supports_schema: Boolean(m.traits?.supports_response_schema),
      }))
      setAvailableModels(models)
      if (models.length > 0 && !models.find((m) => m.id === model)) {
        const schemaFirst = models.find((m) => m.supports_schema)
        setModel((schemaFirst ?? models[0]).id)
      }
      toast.success(`Discovered ${models.length} text models`)
    } catch (error) {
      console.error('Model discovery failed:', error)
      setDiscoverError('Failed to discover models')
      toast.error('Failed to discover models')
    }
    setDiscovering(false)
  }

  const handleTest = async () => {
    if (!name) return

    setTesting(true)
    try {
      const result = await invoke<boolean>('test_provider_connection', { profile_name: name })
      setTestResult(result)
      if (result) {
        toast.success('Connection test successful!')
      } else {
        toast.error('Connection test failed')
      }
    } catch {
      setTestResult(false)
      toast.error('Connection test failed with error')
    }
    setTesting(false)
  }

  const handleDelete = async () => {
    if (!selectedProfile) return
    try {
      await invoke('delete_provider_profile', { name: selectedProfile })
      toast.success('Provider deleted')
      resetForm()
      await loadProfiles()
    } catch (error) {
      console.error('Failed to delete:', error)
      toast.error('Failed to delete provider')
    }
  }

  const fallbackModels = ['gpt-4o', 'gpt-4o-mini', 'gpt-4-turbo', 'claude-3-opus', 'claude-3-sonnet']
  const fallbackOptions = model && !fallbackModels.includes(model)
    ? [model, ...fallbackModels]
    : fallbackModels

  return (
    <div className="max-w-2xl mx-auto space-y-8">
      <h1 className="text-3xl font-bold text-emerald-50">
        <span role="img" aria-label="settings">⚙️</span> Provider Settings
      </h1>

      <Card className="space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold text-emerald-50">
            Existing Profiles
          </h2>
          <button
            onClick={resetForm}
            className="btn btn-secondary text-sm"
          >
            ➕ New Profile
          </button>
        </div>
        {loadingProfiles ? (
          <p className="text-sm text-emerald-200/60">Loading profiles...</p>
        ) : profiles.length > 0 ? (
          <div className="flex gap-3">
            <select
              value={selectedProfile}
              onChange={(e) => handleSelectProfile(e.target.value)}
              className="input flex-1"
            >
              <option value="" disabled>Select a profile</option>
              {profiles.map((p) => (
                <option key={p.name} value={p.name}>
                  {p.name} ({p.default_model})
                </option>
              ))}
            </select>
            <button
              onClick={handleDelete}
              disabled={!selectedProfile}
              className="btn btn-secondary text-sm"
            >
              🗑️ Delete
            </button>
          </div>
        ) : (
          <p className="text-sm text-emerald-200/60">
            No profiles saved yet. Create one below.
          </p>
        )}
      </Card>

      <Card className="space-y-6">
        <div>
          <label className="block mb-2 text-emerald-50">
            Profile Name
          </label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="e.g., OpenAI, Anthropic"
            className="input w-full"
          />
        </div>

        <div>
          <label className="block mb-2 text-emerald-50">
            Base URL
          </label>
          <input
            type="text"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder="https://api.openai.com/v1"
            className="input w-full"
          />
          <p className="text-sm mt-1 text-emerald-200/60">
            For OpenAI-compatible APIs
          </p>
        </div>

        <div>
          <label className="block mb-2 text-emerald-50">
            API Key
          </label>
          <input
            type="password"
            value={key}
            onChange={(e) => setKey(e.target.value)}
            placeholder="sk-..."
            className="input w-full"
          />
          <p className="text-sm mt-1 text-emerald-200/60">
            Stored securely in macOS Keychain. Leave blank to keep existing key.
          </p>
        </div>

        <div>
          <label className="block mb-2 text-emerald-50">
            Organization (optional)
          </label>
          <input
            type="text"
            value={organization}
            onChange={(e) => setOrganization(e.target.value)}
            placeholder="org_..."
            className="input w-full"
          />
        </div>

        <div>
          <label className="block mb-2 text-emerald-50">
            Project (optional)
          </label>
          <input
            type="text"
            value={project}
            onChange={(e) => setProject(e.target.value)}
            placeholder="project_..."
            className="input w-full"
          />
        </div>

        <div>
          <label className="block mb-2 text-emerald-50">
            Timeout (seconds)
          </label>
          <input
            type="number"
            min={5}
            max={300}
            value={timeout}
            onChange={(e) => setTimeout(parseInt(e.target.value || '0', 10) || 0)}
            className="input w-full"
          />
          <p className="text-xs text-emerald-200/60 mt-1">
            Applies to scans, prompts, and model discovery.
          </p>
        </div>

        <div>
          <label className="block mb-2 text-emerald-50">
            Custom Headers (JSON)
          </label>
          <textarea
            value={headersText}
            onChange={(e) => {
              setHeadersText(e.target.value)
              setHeadersError(null)
            }}
            placeholder='{"X-Title": "HQE Workbench", "HTTP-Referer": "https://github.com/AbstergoSweden/HQE-Workbench"}'
            className="input w-full h-28 font-mono text-xs"
          />
          {headersError ? (
            <p className="text-xs text-red-300 mt-1">{headersError}</p>
          ) : (
            <p className="text-xs text-emerald-200/60 mt-1">
              Provide a JSON object of header key/value pairs.
            </p>
          )}
        </div>

        <div>
          <label className="block mb-2 text-emerald-50">
            Default Model
          </label>
          <select
            value={model}
            onChange={(e) => setModel(e.target.value)}
            className="input w-full"
          >
              {availableModels.length > 0 ? (
                availableModels.map((m) => (
                  <option key={m.id} value={m.id}>
                    {m.name}{m.supports_schema ? ' (JSON)' : ''}
                  </option>
                ))
              ) : (
              <>
                {fallbackOptions.map((opt) => (
                  <option key={opt} value={opt}>
                    {opt}
                  </option>
                ))}
              </>
            )}
          </select>
          <div className="flex items-center justify-between mt-2">
            <p className="text-sm text-emerald-200/60">
              {availableModels.length > 0
                ? `${availableModels.length} text models available`
                : 'Discover models to populate this list'}
            </p>
            <button
              onClick={handleDiscoverModels}
              disabled={discovering || !url}
              className="btn btn-secondary text-sm"
            >
              {discovering ? 'Discovering...' : '🔎 Discover Models'}
            </button>
          </div>
          {availableModels.length > 0 && !availableModels.find((m) => m.id === model)?.supports_schema && (
            <p className="text-sm mt-2 text-amber-300/80">
              Selected model does not advertise JSON/schema support. LLM scans will fall back to best-effort parsing.
            </p>
          )}
          {discoverError && (
            <p className="text-sm mt-1 text-red-400">
              {discoverError}
            </p>
          )}
        </div>

        <div className="flex gap-4 pt-4">
          <button
            onClick={handleSave}
            disabled={saving || !name || !url}
            className="btn btn-primary flex-1"
          >
            {saving ? 'Saving...' : '💾 Save Provider'}
          </button>

          <button
            onClick={handleTest}
            disabled={testing || !name}
            className="btn btn-secondary"
          >
            {testing ? 'Testing...' : '🧪 Test Connection'}
          </button>
        </div>

        {testResult !== null && (
          <div
            className={`p-4 rounded-lg text-center ${testResult ? 'bg-green-500/20 text-green-500' : 'bg-red-500/20 text-red-500'
              }`}
          >
            {testResult ? '✅ Connection successful!' : '❌ Connection failed'}
          </div>
        )}
      </Card>

      <Card className="bg-emerald-500/5 border-emerald-500/10">
        <h3 className="font-semibold mb-2 text-emerald-50">
          <span role="img" aria-label="lightbulb">💡</span> Privacy Note
        </h3>
        <p className="text-sm text-emerald-200/60">
          Your API keys are stored securely in the macOS Keychain and are never
          written to disk in plain text. All LLM requests are made directly from
          your machine - we never proxy or log your code.
        </p>
      </Card>
    </div>
  )
}
