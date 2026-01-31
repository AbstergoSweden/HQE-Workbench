
import { useNavigate } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import { useReportStore } from '../store'
import { Finding, TodoItem, Severity } from '../types'
import { Card } from '../components/ui/Card'
import { Badge, BadgeVariant } from '../components/ui/Badge'
import { useToast } from '../context/ToastContext'

export function ReportScreen() {
  const navigate = useNavigate()
  const { report } = useReportStore()
  const toast = useToast()

  if (!report) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <p className="text-emerald-200/60 mb-4">
            No scan results available
          </p>
          <button
            onClick={() => navigate('/scan')}
            className="btn btn-primary"
          >
            Run a Scan
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

  const handleViewReport = async () => {
    try {
      const targetDir = await invoke<string | null>('select_folder')
      if (!targetDir) return
      await invoke('export_artifacts', { run_id, target_dir: targetDir })
      toast.success('Report exported. Open report.md in the selected folder.')
    } catch (error) {
      console.error('Report export failed:', error)
      toast.error('Failed to export report')
    }
  }

  const getScoreColorClass = (score: number) => {
    if (score >= 8) return 'text-emerald-500'
    if (score >= 5) return 'text-yellow-500'
    return 'text-red-500'
  }

  const getScoreBgStyle = (score: number) => {
    if (score >= 8) return 'rgba(34, 197, 94, 0.1)'
    if (score >= 5) return 'rgba(234, 179, 8, 0.1)'
    return 'rgba(239, 68, 68, 0.1)'
  }

  // Flatten findings from all categories
  const allFindings: Finding[] = [
    ...deep_scan_results.security,
    ...deep_scan_results.code_quality,
    ...deep_scan_results.frontend,
    ...deep_scan_results.backend,
    ...deep_scan_results.testing,
  ]

  const getSeverityColor = (severity: Severity) => {
    switch (severity.toLowerCase()) {
      case 'critical': return '#ef4444'
      case 'high': return '#f97316'
      case 'medium': return '#eab308'
      case 'low': return '#10b981'
      default: return '#3b82f6'
    }
  }

  const renderEvidence = (evidence: Finding['evidence']) => {
    switch (evidence.type) {
      case 'file_line':
        return (
          <>
            <div className="flex items-center gap-2 mb-2 pb-2 border-b border-emerald-500/10 text-emerald-200/60">
              <span><span role="img" aria-label="file">📄</span> {evidence.file}</span>
              {evidence.line && <span>:{evidence.line}</span>}
            </div>
            {evidence.snippet && (
              <pre className="overflow-auto text-emerald-200/80">
                <code>{evidence.snippet}</code>
              </pre>
            )}
          </>
        )
      case 'file_function':
        return (
          <>
            <div className="flex items-center gap-2 mb-2 pb-2 border-b border-emerald-500/10 text-emerald-200/60">
              <span><span role="img" aria-label="function">🧩</span> {evidence.file}</span>
              {evidence.function && <span>::{evidence.function}</span>}
            </div>
            {evidence.snippet && (
              <pre className="overflow-auto text-emerald-200/80">
                <code>{evidence.snippet}</code>
              </pre>
            )}
          </>
        )
      case 'reproduction':
        return (
          <>
            <div className="flex items-center gap-2 mb-2 pb-2 border-b border-emerald-500/10 text-emerald-200/60">
              <span><span role="img" aria-label="repro">🧪</span> Reproduction</span>
            </div>
            {evidence.steps && evidence.steps.length > 0 && (
              <ol className="list-decimal pl-5 text-emerald-200/80 space-y-1">
                {evidence.steps.map((step, idx) => (
                  <li key={idx}>{step}</li>
                ))}
              </ol>
            )}
            {evidence.observed && (
              <p className="mt-2 text-emerald-200/80">
                <span className="text-emerald-200/60">Observed:</span> {evidence.observed}
              </p>
            )}
          </>
        )
      default:
        return (
          <p className="text-emerald-200/60">No evidence provided.</p>
        )
    }
  }

  return (
    <div className="max-w-6xl mx-auto space-y-8">
      <h1 className="text-3xl font-bold text-emerald-50">
        <span role="img" aria-label="report">📊</span> Scan Report
      </h1>

      {/* Executive Summary */}
      <Card>
        <h2 className="text-lg font-semibold mb-6 text-emerald-50">
          Executive Summary
        </h2>

        <div className="grid grid-cols-3 gap-6">
          <div
            className="text-center p-6 rounded-lg"
            style={{ background: getScoreBgStyle(health_score || 0) }}
          >
            <p className="text-sm mb-2 text-emerald-200/60">
              Health Score
            </p>
            <p className={`text-5xl font-bold ${getScoreColorClass(health_score || 0)}`}>
              {health_score}/10
            </p>
          </div>

          <div className="text-center p-6 rounded-lg bg-black/20">
            <p className="text-sm mb-2 text-emerald-200/60">
              TODO Items
            </p>
            <p className="text-5xl font-bold text-emerald-50">
              {master_todo_backlog.length}
            </p>
          </div>

          <div className="text-center p-6 rounded-lg bg-black/20">
            <p className="text-sm mb-2 text-emerald-200/60">
              Run ID
            </p>
            <p className="text-sm font-mono truncate text-emerald-50" title={run_id}>
              {run_id}
            </p>
            {report.provider?.name && (
              <p className="text-xs mt-2 text-emerald-200/60">
                Provider: {report.provider.name}
                {report.provider.model ? ` · ${report.provider.model}` : ''}
              </p>
            )}
          </div>
        </div>
      </Card>

      {/* Local Mode Notice */}
      {isLocalOnly && (
        <div className="p-4 rounded-lg bg-emerald-500/10 border border-emerald-500/20">
          <div className="flex items-start gap-3">
            <span className="text-xl" role="img" aria-label="info">💡</span>
            <div>
              <p className="font-medium text-emerald-50">
                Local-Only Analysis
              </p>
              <p className="text-sm mt-1 text-emerald-200/60">
                This report was generated using local heuristics. For AI-powered analysis
                and patch generation, configure an LLM provider in Settings.
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Findings */}
      <Card>
        <h2 className="text-lg font-semibold mb-6 text-emerald-50">
          <span role="img" aria-label="findings">🔍</span> Detailed Findings
        </h2>

        {allFindings.length === 0 ? (
          <p className="text-center py-8 opacity-50">No findings identified in this scan.</p>
        ) : (
          <div className="space-y-4">
            {allFindings.map((finding) => (
              <div
                key={finding.id}
                className="rounded-lg overflow-hidden bg-black/20 border-l-4"
                style={{ borderColor: getSeverityColor(finding.severity) }}
              >
                {/* Header */}
                <div className="p-4 flex items-center gap-4 border-b border-emerald-500/10">
                  <Badge variant={finding.severity as BadgeVariant}>
                    {finding.severity}
                  </Badge>
                  <span className="font-mono text-sm text-emerald-400">
                    {finding.id}
                  </span>
                  <span className="text-sm text-emerald-200/60">
                    {finding.category}
                  </span>
                </div>

                {/* Content */}
                <div className="p-4 space-y-4">
                  <p className="font-medium text-emerald-50">
                    {finding.title}
                  </p>

                  {/* Evidence / Snippet */}
                  <div className="rounded-lg p-3 font-mono text-sm bg-black/30 border border-emerald-500/10">
                    {renderEvidence(finding.evidence)}
                  </div>

                  {/* Impact & Recommendation */}
                  <div className="space-y-2 text-sm">
                    <p className="text-emerald-50">
                      <span className="text-emerald-200/60">Impact: </span>
                      {finding.impact}
                    </p>
                    <p className="text-emerald-50">
                      <span className="text-emerald-400 font-medium">
                        <span role="img" aria-label="fix">💡</span> Fix:
                      </span>
                      {finding.recommendation}
                    </p>
                  </div>

                  {/* Actions */}
                  <div className="pt-4 border-t border-emerald-500/10 flex justify-end">
                    <button
                      onClick={() => navigate('/thinktank', {
                        state: {
                          promptName: finding.category.toLowerCase().includes('security') ? 'code_review_security' : 'code_review_best_practices',
                          args: { args: finding.evidence.snippet || '' }
                        }
                      })}
                      className="text-xs px-3 py-1.5 rounded bg-emerald-600/20 text-emerald-400 hover:bg-emerald-600/30 transition-colors border border-emerald-600/30 font-medium flex items-center gap-1.5"
                    >
                      <span role="img" aria-label="analyze">🧠</span> Deep Analyze
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </Card>

      {/* Master TODO Backlog */}
      <Card>
        <h2 className="text-lg font-semibold mb-6 text-emerald-50">
          <span role="img" aria-label="todo">📝</span> Master TODO Backlog
        </h2>

        <div className="space-y-3">
          {master_todo_backlog.length === 0 ? (
            <p className="opacity-50 text-sm italic">No items in backlog.</p>
          ) : (
            master_todo_backlog.map((item: TodoItem) => (
              <div
                key={item.id}
                className="flex items-center gap-4 p-4 rounded-lg bg-black/20 border-l-4"
                style={{ borderColor: getSeverityColor(item.severity) }}
              >
                <span className="font-mono text-sm text-emerald-400 min-w-[80px]">
                  {item.id}
                </span>
                <div className="flex-1 min-w-0">
                  <p className="text-emerald-50 truncate">
                    {item.title}
                  </p>
                  <p className="text-sm text-emerald-200/60 truncate">
                    Root Cause: {item.root_cause}
                  </p>
                </div>
                <Badge variant={item.severity as BadgeVariant}>
                  {item.severity}
                </Badge>
              </div>
            ))
          )}
        </div>
      </Card>

      <div className="flex gap-4">
        <button className="btn btn-primary" onClick={handleExport}>
          📦 Export Artifacts
        </button>
        <button className="btn btn-secondary" onClick={handleViewReport}>
          📄 View Full Report
        </button>
      </div>
    </div>
  )
}
