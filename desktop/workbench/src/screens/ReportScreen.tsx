import { useNavigate } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import { useReportStore, useRepoStore } from '../store'
import { Finding, Severity, ChatMessage } from '../types'
import { useToast } from '../context/ToastContext'
import { UnifiedOutputPanel } from '../components/UnifiedOutputPanel'

const buildReportMarkdown = (report: {
  run_id: string
  provider?: { name: string; model?: string | null; llm_enabled?: boolean } | null
  executive_summary: { health_score: number; top_priorities: string[]; critical_findings: string[]; blockers: Array<{ description: string; reason: string; how_to_obtain: string }> }
  deep_scan_results: {
    security: Finding[]
    code_quality: Finding[]
    frontend: Finding[]
    backend: Finding[]
    testing: Finding[]
  }
  master_todo_backlog: Array<{ title: string; severity: Severity; category: string; fix_approach: string }>
}) => {
  const { run_id, executive_summary, deep_scan_results, master_todo_backlog, provider } = report
  const sections: string[] = []

  sections.push(`# HQE Report`)
  sections.push(`**Run ID:** ${run_id}`)
  if (provider?.name) {
    sections.push(`**Provider:** ${provider.name}${provider.model ? ` (${provider.model})` : ''}`)
  }
  sections.push(`**Mode:** ${provider?.llm_enabled ? 'llm-enhanced' : 'local-only'}`)
  sections.push('')
  sections.push(`## Executive Summary`)
  sections.push(`- Health score: **${executive_summary.health_score}/10**`)
  if (executive_summary.top_priorities.length > 0) {
    sections.push(`- Top priorities:`)
    executive_summary.top_priorities.forEach((priority) => {
      sections.push(`  - ${priority}`)
    })
  }
  if (executive_summary.critical_findings.length > 0) {
    sections.push(`- Critical findings:`)
    executive_summary.critical_findings.forEach((finding) => {
      sections.push(`  - ${finding}`)
    })
  }
  if (executive_summary.blockers.length > 0) {
    sections.push(`- Blockers:`)
    executive_summary.blockers.forEach((blocker) => {
      sections.push(`  - ${blocker.description} (reason: ${blocker.reason})`)
    })
  }
  sections.push('')

  const categories: Array<{ label: string; items: Finding[] }> = [
    { label: 'Security', items: deep_scan_results.security },
    { label: 'Code Quality', items: deep_scan_results.code_quality },
    { label: 'Frontend', items: deep_scan_results.frontend },
    { label: 'Backend', items: deep_scan_results.backend },
    { label: 'Testing', items: deep_scan_results.testing },
  ]

  sections.push('## Findings')
  categories.forEach((category) => {
    sections.push(`### ${category.label} (${category.items.length})`)
    if (category.items.length === 0) {
      sections.push(`- None`)
      return
    }
    category.items.forEach((finding) => {
      sections.push(`- **${finding.severity.toUpperCase()}** ${finding.title} (${finding.id})`)
      sections.push(`  - Category: ${finding.category}`)
      sections.push(`  - Impact: ${finding.impact}`)
      sections.push(`  - Recommendation: ${finding.recommendation}`)
    })
  })

  sections.push('')
  sections.push(`## TODO Backlog (${master_todo_backlog.length})`)
  if (master_todo_backlog.length === 0) {
    sections.push(`- None`)
  } else {
    master_todo_backlog.slice(0, 10).forEach((todo) => {
      sections.push(`- **${todo.severity.toUpperCase()}** ${todo.title}`)
      sections.push(`  - Category: ${todo.category}`)
      sections.push(`  - Fix approach: ${todo.fix_approach}`)
    })
    if (master_todo_backlog.length > 10) {
      sections.push(`- ... and ${master_todo_backlog.length - 10} more items`)
    }
  }

  return sections.join('\n')
}

export function ReportScreen() {
  const navigate = useNavigate()
  const { report } = useReportStore()
  const { path: repoPath } = useRepoStore()
  const toast = useToast()

  if (!report) {
    return (
      <div className="flex items-center justify-center h-full">
        <div 
          className="card p-6 text-center"
          style={{ borderColor: 'var(--dracula-comment)' }}
        >
          <div className="text-terminal-comment mb-4 text-lg">$</div>
          <p className="text-sm mb-4" style={{ color: 'var(--dracula-comment)' }}>
            No scan results available
          </p>
          <button
            onClick={() => navigate('/scan')}
            className="btn btn-primary"
          >
            <span>❯</span> run_scan
          </button>
        </div>
      </div>
    )
  }

  const { run_id, executive_summary, deep_scan_results, master_todo_backlog } = report
  const { health_score } = executive_summary as { health_score: number }
  const isLocalOnly = !report.provider?.llm_enabled

  const handleExport = async () => {
    try {
      const targetDir = await invoke<string | null>('select_folder')
      if (!targetDir) return
      await invoke('export_artifacts', { run_id, target_dir: targetDir })
      toast.success('Artifacts exported successfully')
    } catch (error) {
      console.error('Export failed:', error)
      toast.error('Failed to export artifacts')
    }
  }

  const getScoreColor = (score: number) => {
    if (score >= 8) return 'var(--dracula-green)'
    if (score >= 5) return 'var(--dracula-orange)'
    return 'var(--dracula-red)'
  }

  const getSeverityBadge = (severity: Severity) => {
    const colors: Record<string, string> = {
      critical: 'var(--dracula-red)',
      high: 'var(--dracula-orange)',
      medium: 'var(--dracula-yellow)',
      low: 'var(--dracula-green)',
    }
    return colors[severity] || 'var(--dracula-cyan)'
  }

  // Flatten findings from all categories
  const allFindings: Finding[] = [
    ...deep_scan_results.security,
    ...deep_scan_results.code_quality,
    ...deep_scan_results.frontend,
    ...deep_scan_results.backend,
    ...deep_scan_results.testing,
  ]

  const criticalCount = allFindings.filter(f => f.severity === 'critical').length
  const highCount = allFindings.filter(f => f.severity === 'high').length
  const mediumCount = allFindings.filter(f => f.severity === 'medium').length
  const lowCount = allFindings.filter(f => f.severity === 'low').length

  const reportContent = buildReportMarkdown({
    run_id,
    provider: report.provider ?? undefined,
    executive_summary,
    deep_scan_results,
    master_todo_backlog: master_todo_backlog.map((todo) => ({
      title: todo.title,
      severity: todo.severity,
      category: todo.category,
      fix_approach: todo.fix_approach,
    })),
  })

  const reportMessages: ChatMessage[] = [{
    id: `report-${run_id}`,
    session_id: '',
    role: 'assistant',
    content: reportContent,
    timestamp: new Date().toISOString(),
  }]

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="text-terminal-green">❯</span>
          <h1 className="text-lg font-bold" style={{ color: 'var(--dracula-fg)' }}>
            scan_report
          </h1>
        </div>
        <button
          onClick={handleExport}
          className="btn text-sm"
        >
          <span className="text-terminal-cyan">↓</span> export_artifacts
        </button>
      </div>

      {/* Health Score Card */}
      <div 
        className="card p-4"
        style={{ borderColor: 'var(--dracula-comment)' }}
      >
        <div className="flex items-center justify-between">
          <div>
            <div className="text-xs uppercase tracking-wider mb-1" style={{ color: 'var(--dracula-comment)' }}>
              Health Score
            </div>
            <div 
              className="text-5xl font-bold font-mono"
              style={{ color: getScoreColor(health_score) }}
            >
              {health_score}<span className="text-2xl" style={{ color: 'var(--dracula-comment)' }}>/10</span>
            </div>
          </div>
          
          <div className="text-right">
            <div className="font-mono text-sm" style={{ color: 'var(--dracula-comment)' }}>
              run_id: <span style={{ color: 'var(--dracula-fg)' }}>{run_id.slice(0, 8)}...</span>
            </div>
            <div className="font-mono text-sm mt-1" style={{ color: 'var(--dracula-comment)' }}>
              mode: <span style={{ color: isLocalOnly ? 'var(--dracula-green)' : 'var(--dracula-cyan)' }}>
                {isLocalOnly ? 'local-only' : 'llm-enhanced'}
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Summary Stats */}
      <div className="grid grid-cols-4 gap-3">
        <div 
          className="card p-3 text-center"
          style={{ borderColor: 'var(--dracula-red)' }}
        >
          <div className="text-2xl font-bold font-mono" style={{ color: 'var(--dracula-red)' }}>
            {criticalCount}
          </div>
          <div className="text-xs" style={{ color: 'var(--dracula-comment)' }}>Critical</div>
        </div>
        <div 
          className="card p-3 text-center"
          style={{ borderColor: 'var(--dracula-orange)' }}
        >
          <div className="text-2xl font-bold font-mono" style={{ color: 'var(--dracula-orange)' }}>
            {highCount}
          </div>
          <div className="text-xs" style={{ color: 'var(--dracula-comment)' }}>High</div>
        </div>
        <div 
          className="card p-3 text-center"
          style={{ borderColor: 'var(--dracula-yellow)' }}
        >
          <div className="text-2xl font-bold font-mono" style={{ color: 'var(--dracula-yellow)' }}>
            {mediumCount}
          </div>
          <div className="text-xs" style={{ color: 'var(--dracula-comment)' }}>Medium</div>
        </div>
        <div 
          className="card p-3 text-center"
          style={{ borderColor: 'var(--dracula-green)' }}
        >
          <div className="text-2xl font-bold font-mono" style={{ color: 'var(--dracula-green)' }}>
            {lowCount}
          </div>
          <div className="text-xs" style={{ color: 'var(--dracula-comment)' }}>Low</div>
        </div>
      </div>

      {/* Findings List */}
      <div 
        className="card"
        style={{ borderColor: 'var(--dracula-comment)' }}
      >
        <div 
          className="px-4 py-3 border-b flex items-center justify-between"
          style={{ borderColor: 'var(--dracula-comment)' }}
        >
          <span className="text-xs uppercase tracking-wider" style={{ color: 'var(--dracula-comment)' }}>
            Findings ({allFindings.length})
          </span>
        </div>
        
        <div className="max-h-64 overflow-auto">
          {allFindings.length === 0 ? (
            <div className="p-4 text-center text-sm" style={{ color: 'var(--dracula-comment)' }}>
              No findings detected
            </div>
          ) : (
            allFindings.map((finding, idx) => (
              <div 
                key={idx}
                className="px-4 py-3 border-b transition-colors hover:bg-opacity-50"
                style={{ 
                  borderColor: 'var(--dracula-comment)',
                  backgroundColor: idx % 2 === 0 ? 'transparent' : 'var(--dracula-bg)'
                }}
              >
                <div className="flex items-start gap-3">
                  <span 
                    className="text-xs px-2 py-0.5 rounded font-mono"
                    style={{ 
                      backgroundColor: `${getSeverityBadge(finding.severity)}20`,
                      color: getSeverityBadge(finding.severity),
                      border: `1px solid ${getSeverityBadge(finding.severity)}`
                    }}
                  >
                    {finding.severity}
                  </span>
                  <div className="flex-1 min-w-0">
                    <div className="font-mono text-sm truncate" style={{ color: 'var(--dracula-fg)' }}>
                      {finding.title}
                    </div>
                    <div className="font-mono text-xs truncate" style={{ color: 'var(--dracula-comment)' }}>
                      {finding.category} • {finding.id}
                    </div>
                  </div>
                </div>
              </div>
            ))
          )}
        </div>
      </div>

      {/* TODO Backlog */}
      <div 
        className="card"
        style={{ borderColor: 'var(--dracula-comment)' }}
      >
        <div 
          className="px-4 py-3 border-b flex items-center justify-between"
          style={{ borderColor: 'var(--dracula-comment)' }}
        >
          <span className="text-xs uppercase tracking-wider" style={{ color: 'var(--dracula-comment)' }}>
            TODO Backlog ({master_todo_backlog.length})
          </span>
        </div>
        
        <div className="max-h-48 overflow-auto">
          {master_todo_backlog.length === 0 ? (
            <div className="p-4 text-center text-sm" style={{ color: 'var(--dracula-comment)' }}>
              No TODO items
            </div>
          ) : (
            master_todo_backlog.slice(0, 10).map((todo, idx) => (
              <div 
                key={idx}
                className="px-4 py-2 border-b flex items-center gap-3"
                style={{ borderColor: 'var(--dracula-comment)' }}
              >
                <input type="checkbox" disabled className="opacity-50" />
                <span 
                  className="text-xs px-1.5 py-0.5 rounded font-mono"
                  style={{ 
                    backgroundColor: `${getSeverityBadge(todo.severity)}20`,
                    color: getSeverityBadge(todo.severity)
                  }}
                >
                  {todo.severity[0]}
                </span>
                <span className="font-mono text-sm truncate flex-1" style={{ color: 'var(--dracula-fg)' }}>
                  {todo.title}
                </span>
              </div>
            ))
          )}
          {master_todo_backlog.length > 10 && (
            <div className="px-4 py-2 text-center text-xs" style={{ color: 'var(--dracula-comment)' }}>
              ... and {master_todo_backlog.length - 10} more items
            </div>
          )}
        </div>
      </div>

      {/* Unified Output Panel */}
      <div
        className="card overflow-hidden"
        style={{ borderColor: 'var(--dracula-comment)' }}
      >
        <UnifiedOutputPanel
          initialMessages={reportMessages}
          contextRef={{
            repo_path: repoPath ?? undefined,
            prompt_id: 'report',
            provider: report.provider?.name ?? 'local',
            model: report.provider?.model ?? 'default',
          }}
          showInput={true}
        />
      </div>
    </div>
  )
}
