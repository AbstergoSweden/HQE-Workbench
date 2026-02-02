
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
  provider?: {
    name: string
    base_url?: string | null
    model?: string | null
    llm_enabled?: boolean
  } | null
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

export interface ProviderProfile {
  name: string
  base_url: string
  api_key_id: string
  default_model: string
  headers?: Record<string, string>
  organization?: string | null
  project?: string | null
  provider_kind?: string | null
  timeout_s: number
}

export interface ProviderModel {
  id: string
  name?: string
  context_length?: number
  traits?: {
    supports_vision?: boolean
    supports_tools?: boolean
    supports_reasoning?: boolean
    supports_web_search?: boolean
    supports_response_schema?: boolean
    supports_logprobs?: boolean
    code_optimized?: boolean
  }
}

export interface ProviderModelList {
  provider_kind?: string
  base_url?: string
  fetched_at_unix_s?: number
  models: ProviderModel[]
}

// Chat Types
export interface ChatSession {
  id: string
  repo_path?: string
  prompt_id?: string
  provider: string
  model: string
  created_at: string
  updated_at: string
  message_count: number
}

export interface ChatMessage {
  id: string
  session_id: string
  parent_id?: string
  role: 'system' | 'user' | 'assistant' | 'tool'
  content: string
  timestamp: string
}

// Enhanced Prompt Types
export interface PromptInput {
  name: string
  description: string
  type: 'string' | 'integer' | 'boolean' | 'json' | 'code' | 'textarea'
  required: boolean
  default?: string
  example?: string
}

export interface PromptMetadata {
  id: string
  title: string
  category: string
  description: string
  explanation: string
  version: string
  inputs: PromptInput[]
  allowed_tools: string[]
}

// Provider Spec Types
export interface ProviderSpec {
  id: string
  display_name: string
  base_url: string
  auth_scheme: string
  default_model: string
  default_headers: Record<string, string>
  recommended_timeout_s: number
  quirks: string[]
  website_url: string
  docs_url: string
  supports_streaming: boolean
  supports_tools: boolean
}
