//! Conversation Panel - Unified Output for Reports and Chat
//!
//! This component provides a single UI for both:
//! - One-shot report outputs (Thinktank/Report results)
//! - Multi-turn chat follow-ups
//!
//! The transition from report to chat is seamless - the report output
//! becomes the first assistant message in the chat thread.

import { FC, useCallback, useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter'
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism'
import { ChatMessage, ChatSession } from '../types'
import { useChatStore } from '../store'
import { useToast } from '../context/ToastContext'

interface ContextRef {
  repo_path?: string
  prompt_id?: string
  provider: string
  model: string
}

interface ConversationPanelProps {
  /** Session ID - if provided, loads existing session */
  sessionId?: string
  /** Initial messages - report output becomes first assistant message */
  initialMessages?: ChatMessage[]
  /** Context reference for the conversation */
  contextRef: ContextRef
  /** Callback when user sends a message */
  onSend?: (message: string) => void
  /** Show input box */
  showInput?: boolean
  /** Loading state */
  isLoading?: boolean
  /** Additional CSS class */
  className?: string
}

/**
 * Conversation Panel - Unified output component for reports and chat.
 * 
 * This component handles:
 * - Displaying chat messages (user and assistant)
 * - Rendering markdown with syntax highlighting
 * - Sending new messages
 * - Creating/loading chat sessions
 * - Smooth transition from report output to chat
 */
export const ConversationPanel: FC<ConversationPanelProps> = ({
  sessionId,
  initialMessages = [],
  contextRef,
  onSend,
  showInput = true,
  isLoading = false,
  className = '',
}) => {
  const toast = useToast()
  const messagesEndRef = useRef<HTMLDivElement>(null)
  const [inputValue, setInputValue] = useState('')
  const [localLoading, setLocalLoading] = useState(false)

  const {
    messages,
    currentSession,
    setMessages,
    addMessage,
    setCurrentSession,
    setIsLoading,
  } = useChatStore()

  const loadSession = useCallback(async (id: string) => {
    try {
      const session = await invoke<ChatSession | null>('get_chat_session', { session_id: id })
      if (session) {
        setCurrentSession(session)
        const msgs = await invoke<ChatMessage[]>('get_chat_messages', { session_id: id })
        setMessages(msgs)
      }
    } catch (err) {
      console.error('Failed to load session:', err)
      toast.error('Failed to load chat session')
    }
  }, [setCurrentSession, setMessages, toast])

  const createNewSession = useCallback(async () => {
    try {
      const session = await invoke<ChatSession>('create_chat_session', {
        repo_path: contextRef.repo_path,
        prompt_id: contextRef.prompt_id,
        provider: contextRef.provider,
        model: contextRef.model,
      })
      setCurrentSession(session)
      setMessages([])
    } catch (err) {
      console.error('Failed to create session:', err)
      toast.error('Failed to create chat session')
    }
  }, [contextRef, setCurrentSession, setMessages, toast])

  const createSessionWithMessages = useCallback(async (initialMsgs: ChatMessage[]) => {
    try {
      const session = await invoke<ChatSession>('create_chat_session', {
        repo_path: contextRef.repo_path,
        prompt_id: contextRef.prompt_id,
        provider: contextRef.provider,
        model: contextRef.model,
      })
      setCurrentSession(session)

      // Add initial messages to the session
      for (const msg of initialMsgs) {
        await invoke('add_chat_message', {
          session_id: session.id,
          parent_id: msg.parent_id,
          role: msg.role,
          content: msg.content,
        })
      }

      // Reload messages from DB to get IDs
      const msgs = await invoke<ChatMessage[]>('get_chat_messages', { session_id: session.id })
      setMessages(msgs)
    } catch (err) {
      console.error('Failed to create session with messages:', err)
      toast.error('Failed to initialize chat')
    }
  }, [contextRef, setCurrentSession, setMessages, toast])

  // Initialize session on mount
  useEffect(() => {
    if (sessionId) {
      loadSession(sessionId)
    } else if (initialMessages.length > 0) {
      // Create new session with initial messages (report → chat transition)
      createSessionWithMessages(initialMessages)
    } else {
      // Start fresh session
      createNewSession()
    }
  }, [sessionId, initialMessages, loadSession, createSessionWithMessages, createNewSession])

  // Scroll to bottom when messages change
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  const handleSend = useCallback(async () => {
    if (!inputValue.trim() || !currentSession) return

    const content = inputValue.trim()
    setInputValue('')
    setLocalLoading(true)
    setIsLoading(true)

    try {
      // Call optional callback
      onSend?.(content)

      // Send via Tauri command
      const response = await invoke<{ message: ChatMessage }>('send_chat_message', {
        session_id: currentSession.id,
        content,
        parent_id: messages[messages.length - 1]?.id,
      })

      // Update local state
      addMessage(response.message)
    } catch (err) {
      console.error('Failed to send message:', err)
      toast.error('Failed to send message')
    } finally {
      setLocalLoading(false)
      setIsLoading(false)
    }
  }, [inputValue, currentSession, messages, onSend, addMessage, toast, setIsLoading])

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }

  const displayMessages = messages.length > 0 ? messages : initialMessages
  const loading = isLoading || localLoading

  return (
    <div className={`flex flex-col h-full ${className}`}>
      {/* Messages Area */}
      <div className="flex-1 overflow-auto p-4 space-y-4">
        {displayMessages.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <div className="text-center" style={{ color: 'var(--dracula-comment)' }}>
              <div className="text-4xl mb-4">💬</div>
              <p className="text-sm">Start a conversation</p>
              <p className="text-xs mt-2 opacity-70">
                Ask follow-up questions about the analysis
              </p>
            </div>
          </div>
        ) : (
          <>
            {displayMessages.map((message, idx) => (
              <MessageBubble
                key={message.id || idx}
                message={message}
                isFirst={idx === 0 && initialMessages.length > 0}
              />
            ))}
            {loading && <LoadingBubble />}
            <div ref={messagesEndRef} />
          </>
        )}
      </div>

      {/* Input Area */}
      {showInput && (
        <div
          className="p-4 border-t"
          style={{ borderColor: 'var(--dracula-comment)' }}
        >
          <div className="flex gap-2">
            <textarea
              value={inputValue}
              onChange={(e) => setInputValue(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Ask a follow-up question..."
              className="flex-1 input min-h-[60px] resize-none"
              disabled={loading || !currentSession}
            />
            <button
              onClick={handleSend}
              disabled={!inputValue.trim() || loading || !currentSession}
              className="btn btn-primary px-4"
            >
              {loading ? (
                <span className="animate-spin">⟳</span>
              ) : (
                <span>➤</span>
              )}
            </button>
          </div>
          <div className="flex justify-between mt-2 text-xs" style={{ color: 'var(--dracula-comment)' }}>
            <span>
              {currentSession ? (
                <>
                  Session: <span className="font-mono">{currentSession.id.slice(0, 8)}...</span>
                  {' • '}
                  {currentSession.provider}/{currentSession.model}
                </>
              ) : (
                'Initializing...'
              )}
            </span>
            <span>Press Enter to send, Shift+Enter for new line</span>
          </div>
        </div>
      )}
    </div>
  )
}

/**
 * Individual message bubble component
 */
interface MessageBubbleProps {
  message: ChatMessage
  isFirst?: boolean
}

const MessageBubble: FC<MessageBubbleProps> = ({ message, isFirst }) => {
  const isUser = message.role === 'user'

  return (
    <div
      className={`flex ${isUser ? 'justify-end' : 'justify-start'}`}
    >
      <div
        className={`max-w-[85%] rounded-lg p-4 ${isUser
            ? 'rounded-br-none'
            : 'rounded-bl-none'
          }`}
        style={{
          backgroundColor: isUser
            ? 'var(--dracula-comment)30'
            : isFirst
              ? 'var(--dracula-bg)'
              : 'var(--dracula-current-line)',
          border: `1px solid ${isUser
              ? 'var(--dracula-comment)50'
              : isFirst
                ? 'var(--dracula-green)50'
                : 'var(--dracula-comment)30'
            }`,
        }}
      >
        {/* Role indicator */}
        <div className="flex items-center gap-2 mb-2">
          <span
            className="text-xs font-mono px-2 py-0.5 rounded"
            style={{
              backgroundColor: isUser
                ? 'var(--dracula-comment)30'
                : isFirst
                  ? 'var(--dracula-green)20'
                  : 'var(--dracula-purple)20',
              color: isUser
                ? 'var(--dracula-comment)'
                : isFirst
                  ? 'var(--dracula-green)'
                  : 'var(--dracula-purple)',
            }}
          >
            {isUser ? 'user' : isFirst ? 'report' : 'assistant'}
          </span>
          <span className="text-xs opacity-50" style={{ color: 'var(--dracula-comment)' }}>
            {new Date(message.timestamp).toLocaleTimeString()}
          </span>
        </div>

        {/* Message content */}
        <div className="prose prose-invert max-w-none prose-sm">
          <ReactMarkdown
            remarkPlugins={[remarkGfm]}
            components={{
              code({ inline, className, children, ...props }: { inline?: boolean; className?: string; children?: React.ReactNode }) {
                const match = /language-(\w+)/.exec(className || '')
                return !inline && match ? (
                  <SyntaxHighlighter
                    {...props}
                    style={vscDarkPlus}
                    language={match[1]}
                    PreTag="div"
                  >
                    {String(children).replace(/\n$/, '')}
                  </SyntaxHighlighter>
                ) : (
                  <code {...props} className={className}>
                    {children}
                  </code>
                )
              }
            }}
          >
            {message.content}
          </ReactMarkdown>
        </div>
      </div>
    </div>
  )
}

/**
 * Loading indicator bubble
 */
const LoadingBubble: FC = () => (
  <div className="flex justify-start">
    <div
      className="max-w-[85%] rounded-lg rounded-bl-none p-4"
      style={{
        backgroundColor: 'var(--dracula-current-line)',
        border: '1px solid var(--dracula-comment)30',
      }}
    >
      <div className="flex items-center gap-2 mb-2">
        <span
          className="text-xs font-mono px-2 py-0.5 rounded"
          style={{
            backgroundColor: 'var(--dracula-purple)20',
            color: 'var(--dracula-purple)',
          }}
        >
          assistant
        </span>
      </div>
      <div className="flex gap-1">
        <span className="animate-bounce" style={{ animationDelay: '0ms' }}>●</span>
        <span className="animate-bounce" style={{ animationDelay: '150ms' }}>●</span>
        <span className="animate-bounce" style={{ animationDelay: '300ms' }}>●</span>
      </div>
    </div>
  </div>
)

export default ConversationPanel
