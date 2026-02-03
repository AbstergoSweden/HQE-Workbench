import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { HqeReport, ChatSession, ChatMessage } from './types'

export interface RepoState {
  path: string | null
  name: string | null
  isGit: boolean
  setRepo: (path: string, name: string, isGit: boolean) => void
  clearRepo: () => void
}

export interface ScanState {
  isScanning: boolean
  phase: string | null
  progress: number
  error: string | null
  setScanning: (scanning: boolean) => void
  setPhase: (phase: string) => void
  setProgress: (progress: number) => void
  setError: (error: string | null) => void
  reset: () => void
}

export interface ReportState {
  report: HqeReport | null
  setReport: (report: HqeReport) => void
  clearReport: () => void
}

export interface ChatState {
  sessions: ChatSession[]
  currentSession: ChatSession | null
  messages: ChatMessage[]
  isLoading: boolean
  setSessions: (sessions: ChatSession[]) => void
  setCurrentSession: (session: ChatSession | null) => void
  setMessages: (messages: ChatMessage[]) => void
  addMessage: (message: ChatMessage) => void
  setIsLoading: (loading: boolean) => void
  clearChat: () => void
}

export const useRepoStore = create<RepoState>()(
  persist(
    (set) => ({
      path: null,
      name: null,
      isGit: false,
      setRepo: (path, name, isGit) => set({ path, name, isGit }),
      clearRepo: () => set({ path: null, name: null, isGit: false }),
    }),
    {
      name: 'hqe-repo-storage',
    }
  )
)

export const useScanStore = create<ScanState>((set) => ({
  isScanning: false,
  phase: null,
  progress: 0,
  error: null,
  setScanning: (scanning) => set({ isScanning: scanning }),
  setPhase: (phase) => set({ phase }),
  setProgress: (progress) => set({ progress }),
  setError: (error) => set({ error }),
  reset: () => set({ isScanning: false, phase: null, progress: 0, error: null }),
}))

export const useReportStore = create<ReportState>()(
  persist(
    (set) => ({
      report: null,
      setReport: (report) => set({ report }),
      clearReport: () => set({ report: null }),
    }),
    {
      name: 'hqe-report-storage',
    }
  )
)

export const useChatStore = create<ChatState>((set) => ({
  sessions: [],
  currentSession: null,
  messages: [],
  isLoading: false,
  setSessions: (sessions) => set({ sessions }),
  setCurrentSession: (session) => set({ currentSession: session }),
  setMessages: (messages) => set({ messages }),
  addMessage: (message) => set((state) => ({ messages: [...state.messages, message] })),
  setIsLoading: (loading) => set({ isLoading: loading }),
  clearChat: () => set({ currentSession: null, messages: [] }),
}))
