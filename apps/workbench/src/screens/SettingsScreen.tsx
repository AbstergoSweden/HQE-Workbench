import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useToast } from '../context/ToastContext'
import { Card } from '../components/ui/Card'

export function SettingsScreen() {
  const [name, setName] = useState('')
  const [url, setUrl] = useState('')
  const [key, setKey] = useState('')
  const [model, setModel] = useState('gpt-4o-mini')
  const [saving, setSaving] = useState(false)
  const [testing, setTesting] = useState(false)
  const [testResult, setTestResult] = useState<boolean | null>(null)
  const toast = useToast()

  const handleSave = async () => {
    if (!name || !url || !key) return
    
    setSaving(true)
    try {
      const profile = {
        name,
        base_url: url,
        api_key_id: `api_key:${name}`,
        default_model: model,
        headers: null,
        organization: null,
        project: null,
      }
      
      await invoke('save_provider_config', { profile, apiKey: key })
      toast.success('Provider saved successfully!')
    } catch (error) {
      console.error('Failed to save:', error)
      toast.error('Failed to save provider configuration')
    }
    setSaving(false)
  }

  const handleTest = async () => {
    if (!name) return
    
    setTesting(true)
    try {
      const result = await invoke<boolean>('test_provider_connection', { profileName: name })
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

  return (
    <div className="max-w-2xl mx-auto space-y-8">
      <h1 className="text-3xl font-bold text-emerald-50">
        <span role="img" aria-label="settings">⚙️</span> Provider Settings
      </h1>

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
            Stored securely in macOS Keychain
          </p>
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
            <option value="gpt-4o">GPT-4o</option>
            <option value="gpt-4o-mini">GPT-4o Mini</option>
            <option value="gpt-4-turbo">GPT-4 Turbo</option>
            <option value="claude-3-opus">Claude 3 Opus</option>
            <option value="claude-3-sonnet">Claude 3 Sonnet</option>
          </select>
        </div>

        <div className="flex gap-4 pt-4">
          <button
            onClick={handleSave}
            disabled={saving || !name || !url || !key}
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
            className={`p-4 rounded-lg text-center ${
              testResult ? 'bg-green-500/20 text-green-500' : 'bg-red-500/20 text-red-500'
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
