import { useNavigate } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import { useRepoStore } from '../store'
import { Card } from '../components/ui/Card'
import { useToast } from '../context/ToastContext'

export function WelcomeScreen() {
  const navigate = useNavigate()
  const { setRepo } = useRepoStore()
  const toast = useToast()

  const handleOpenFolder = async () => {
    try {
      const path = await invoke<string | null>('select_folder')
      if (path) {
        const name = path.split('/').pop() || 'Unknown'
        const repoInfo = await invoke<{source: string, git_remote: string | null, git_commit: string | null}>('get_repo_info', { repo_path: path })
        setRepo(path, name, repoInfo.source === 'git')
        navigate('/scan')
      }
    } catch (error) {
      console.error('Failed to open folder:', error)
      toast.error('Failed to open repository. Please try again.')
    }
  }

  return (
    <div className="flex flex-col items-center justify-center h-full animate-fade-in">
      <div className="text-center max-w-2xl">
        <div className="mb-6">
          <span className="text-6xl" role="img" aria-label="logo">🌿</span>
        </div>
        <h1 className="text-5xl font-bold mb-6 text-emerald-50">
          HQE Workbench
        </h1>
        <p className="text-xl mb-4 text-emerald-200/60">
          Health, Quality, and Evolution Analysis
        </p>
        <p className="mb-12 max-w-lg mx-auto text-emerald-200/40">
          Run the HQE Engineer Protocol on your codebase. Get actionable reports,
          prioritized backlogs, and optional AI-powered patch suggestions.
        </p>
        
        <div className="flex gap-4 justify-center">
          <button
            onClick={handleOpenFolder}
            className="btn btn-primary text-lg px-8 py-4 animate-pulse-green flex items-center gap-2"
          >
            <span role="img" aria-label="open">📁</span> Open Repository
          </button>
          
          <button
            onClick={() => navigate('/settings')}
            className="btn btn-secondary text-lg px-8 py-4 flex items-center gap-2"
          >
            <span role="img" aria-label="settings">⚙️</span> Configure Provider
          </button>
        </div>
        
        <div className="mt-16 grid grid-cols-3 gap-6 text-left">
          <Card className="hover:scale-105 transition-transform">
            <div className="text-3xl mb-3" role="img" aria-label="scan">🔍</div>
            <h3 className="font-semibold mb-2 text-emerald-50">
              Deep Scan
            </h3>
            <p className="text-sm text-emerald-200/60">
              Security, code quality, and architectural analysis
            </p>
          </Card>
          
          <Card className="hover:scale-105 transition-transform">
            <div className="text-3xl mb-3" role="img" aria-label="report">📝</div>
            <h3 className="font-semibold mb-2 text-emerald-50">
              Structured Reports
            </h3>
            <p className="text-sm text-emerald-200/60">
              HQE v3 format with evidence-based findings
            </p>
          </Card>
          
          <Card className="hover:scale-105 transition-transform">
            <div className="text-3xl mb-3" role="img" aria-label="privacy">🔒</div>
            <h3 className="font-semibold mb-2 text-emerald-50">
              Privacy First
            </h3>
            <p className="text-sm text-emerald-200/60">
              Local scanning with optional LLM. Secrets redacted.
            </p>
          </Card>
        </div>
      </div>
    </div>
  )
}
