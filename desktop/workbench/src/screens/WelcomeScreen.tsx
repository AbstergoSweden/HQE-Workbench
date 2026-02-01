import { useNavigate } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import { useRepoStore } from '../store'
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

  const features = [
    { 
      cmd: 'scan --deep', 
      desc: 'Security, code quality, and architectural analysis',
      color: 'var(--dracula-cyan)'
    },
    { 
      cmd: 'report --format=hqe', 
      desc: 'Structured HQE v3 reports with evidence-based findings',
      color: 'var(--dracula-purple)'
    },
    { 
      cmd: 'scan --local', 
      desc: 'Privacy-first local scanning with secret redaction',
      color: 'var(--dracula-green)'
    },
  ]

  return (
    <div className="h-full flex flex-col animate-fade-in">
      {/* ASCII Art Header */}
      <div className="mb-8">
        <pre 
          className="text-xs leading-none mb-4"
          style={{ color: 'var(--dracula-green)' }}
        >{`
  ██╗  ██╗ ██████╗ ███████╗    ██╗    ██╗ ██████╗ ██████╗ ██╗  ██╗██████╗ ███████╗███╗   ██╗ ██████╗██╗  ██╗
  ██║  ██║██╔═══██╗██╔════╝    ██║    ██║██╔═══██╗██╔══██╗██║ ██╔╝██╔══██╗██╔════╝████╗  ██║██╔════╝██║  ██║
  ███████║██║   ██║█████╗      ██║ █╗ ██║██║   ██║██████╔╝█████╔╝ ██████╔╝█████╗  ██╔██╗ ██║██║     ███████║
  ██╔══██║██║   ██║██╔══╝      ██║███╗██║██║   ██║██╔══██╗██╔═██╗ ██╔══██╗██╔══╝  ██║╚██╗██║██║     ██╔══██║
  ██║  ██║╚██████╔╝██║         ╚███╔███╔╝╚██████╔╝██║  ██║██║  ██╗██████╔╝███████╗██║ ╚████║╚██████╗██║  ██║
  ╚═╝  ╚═╝ ╚═════╝ ╚═╝          ╚══╝╚══╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═════╝ ╚══════╝╚═╝  ╚═══╝ ╚═════╝╚═╝  ╚═╝
        `}</pre>
        <div className="text-terminal-comment text-sm">
          {'// Health, Quality, and Evolution Analysis'}
        </div>
      </div>

      {/* Main Content Grid */}
      <div className="grid grid-cols-2 gap-6 flex-1">
        {/* Left Column - Actions */}
        <div className="space-y-4">
          <div 
            className="card p-4"
            style={{ borderColor: 'var(--dracula-comment)' }}
          >
            <div className="flex items-center gap-2 mb-4 text-terminal-cyan text-sm">
              <span>$</span>
              <span className="typewriter">initialize_scan</span>
              <span className="terminal-cursor" />
            </div>
            
            <p className="text-sm mb-6" style={{ color: 'var(--dracula-comment)' }}>
              Run the HQE Engineer Protocol on your codebase. Get actionable reports,
              prioritized backlogs, and optional AI-powered patch suggestions.
            </p>
            
            <div className="space-y-3">
              <button
                onClick={handleOpenFolder}
                className="btn btn-primary w-full flex items-center gap-3"
              >
                <span style={{ color: 'var(--dracula-green)' }}>❯</span>
                <span>open_repository</span>
              </button>
              
              <button
                onClick={() => navigate('/settings')}
                className="btn w-full flex items-center gap-3"
              >
                <span style={{ color: 'var(--dracula-purple)' }}>$</span>
                <span>configure_provider</span>
              </button>
            </div>
          </div>

          {/* Quick Stats */}
          <div 
            className="card p-4"
            style={{ borderColor: 'var(--dracula-comment)' }}
          >
            <div className="text-xs uppercase tracking-wider mb-3" style={{ color: 'var(--dracula-comment)' }}>
              System Status
            </div>
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <span style={{ color: 'var(--dracula-comment)' }}>protocol: </span>
                <span className="text-terminal-green">v3.0.0</span>
              </div>
              <div>
                <span style={{ color: 'var(--dracula-comment)' }}>engine: </span>
                <span className="text-terminal-cyan">rust</span>
              </div>
              <div>
                <span style={{ color: 'var(--dracula-comment)' }}>cache: </span>
                <span className="text-terminal-purple">sqlite</span>
              </div>
              <div>
                <span style={{ color: 'var(--dracula-comment)' }}>mode: </span>
                <span className="text-terminal-green">local</span>
              </div>
            </div>
          </div>
        </div>

        {/* Right Column - Features */}
        <div className="space-y-3">
          <div className="text-xs uppercase tracking-wider mb-2" style={{ color: 'var(--dracula-comment)' }}>
            Available Commands
          </div>
          
          {features.map((feature, idx) => (
            <div 
              key={idx}
              className="card p-4 transition-all duration-200 hover:border-terminal-cyan"
              style={{ borderColor: 'var(--dracula-comment)' }}
            >
              <div className="flex items-start gap-3">
                <span style={{ color: feature.color }}>$</span>
                <div className="flex-1">
                  <div 
                    className="font-mono text-sm mb-1"
                    style={{ color: feature.color }}
                  >
                    {feature.cmd}
                  </div>
                  <div className="text-xs" style={{ color: 'var(--dracula-comment)' }}>
                    {feature.desc}
                  </div>
                </div>
              </div>
            </div>
          ))}

          {/* Help Hint */}
          <div 
            className="p-4 mt-4 text-xs"
            style={{ color: 'var(--dracula-comment)' }}
          >
            <div className="flex items-center gap-2 mb-2">
              <span>💡</span>
              <span>Tip:</span>
            </div>
            <p>
              Use <kbd className="px-1 py-0.5 rounded" style={{ background: 'var(--dracula-current-line)' }}>⌘1-5</kbd> to navigate between sections.
              All scans run locally by default for maximum privacy.
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}
