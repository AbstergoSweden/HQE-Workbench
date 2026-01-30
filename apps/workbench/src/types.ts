
export type Severity = 'critical' | 'high' | 'medium' | 'low' | 'info'
export type RiskLevel = 'low' | 'medium' | 'high'

export interface Evidence {
  type: 'file_line' | 'file_function' | 'reproduction'
  file?: string
  line?: number
  function?: string
  snippet?: string
  steps?: string[]
  observed?: string
}

export interface Finding {
  id: string
  severity: Severity
  risk: RiskLevel
  category: string
  title: string
  evidence: Evidence
  impact: string
  recommendation: string
}

export interface TodoItem {
  id: string
  severity: Severity
  risk: RiskLevel
  category: string
  title: string
  root_cause: string
  evidence: Evidence
  fix_approach: string
  verify: string
  blocked_by: string | null
}

export interface ExecutiveSummary {
  health_score: number
  top_priorities: string[]
  critical_findings: string[]
  blockers: Array<{
    description: string
    reason: string
    how_to_obtain: string
  }>
}

export interface HqeReport {
  run_id: string
  executive_summary: ExecutiveSummary
  deep_scan_results: {
    security: Finding[]
    code_quality: Finding[]
    frontend: Finding[]
    backend: Finding[]
    testing: Finding[]
  }
  master_todo_backlog: TodoItem[]
}
