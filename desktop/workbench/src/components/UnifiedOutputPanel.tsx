//! Unified Output Panel - Screen-agnostic output for reports and chat
//!
//! This component provides a single UI for both:
//! - One-shot outputs
//! - Multi-turn chat follow-ups
//!
//! The transition from output to chat is seamless - the first assistant
//! message seeds the thread and drives the message stream.

import { FC, useCallback, useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter'
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism'
import DOMPurify from 'dompurify'
import { ChatMessage, ChatSession, SendChatMessageResponse } from '../types'
import { useChatStore } from '../store'
import { useToast } from '../context/ToastContext'

interface ContextRef {
  repo_path?: string
  prompt_id?: string
  provider: string
  model: string
}

interface UnifiedOutputPanelProps {
  /** Session ID - if provided, loads existing session */
  sessionId?: string
  /** Initial messages - output becomes first assistant message */
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
 * Unified Output Panel - screen-agnostic output component for reports and chat.
 * 
 * This component handles:
 * - Displaying chat messages (user and assistant)
 * - Rendering markdown with syntax highlighting
 * - Sending new messages
 * - Creating/loading chat sessions
 * - Smooth transition from output to chat
 */
export const UnifiedOutputPanel: FC<UnifiedOutputPanelProps> = ({
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
  const messagesRef = useRef<ChatMessage[]>([])  // Track latest messages to avoid stale closures
  const sendingRef = useRef(false)
  const DEFAULT_PAGE_LIMIT = 100
  const [inputValue, setInputValue] = useState('')
  const [localLoading, setLocalLoading] = useState(false)

  const {
    messages,
    currentSession,
    setMessages,
    addMessage,
    prependMessages,
    setCurrentSession,
    setChatState,
    setIsLoading,
    hasMoreHistory,
    isLoadingHistory,
    setHasMoreHistory,
    setIsLoadingHistory,
  } = useChatStore()

  // Keep messagesRef in sync with messages to avoid stale closures
  useEffect(() => {
    messagesRef.current = messages
  }, [messages])

  const loadSession = useCallback(async (id: string) => {
    if (!(window as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__) {
      return
    }
    try {
      const session = await invoke<ChatSession | null>('get_chat_session', {
        session_id: id,
        limit: DEFAULT_PAGE_LIMIT,
      })
      if (session) {
        const msgs = await invoke<ChatMessage[]>('get_chat_messages', {
          session_id: id,
          limit: DEFAULT_PAGE_LIMIT,
        })
        setChatState(session, msgs)
        setHasMoreHistory(
          Boolean(session.message_count && msgs.length < session.message_count)
        )
      }
    } catch (err) {
      console.error('Failed to load session:', err)
      toast.error('Failed to load chat session')
    }
  }, [setChatState, setHasMoreHistory, toast])

  const createNewSession = useCallback(async () => {
    if (!(window as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__) {
      return
    }
    try {
      const session = await invoke<ChatSession>('create_chat_session', {
        repo_path: contextRef.repo_path,
        prompt_id: contextRef.prompt_id,
        provider: contextRef.provider,
        model: contextRef.model,
      })
      // Atomic state update to prevent race conditions
      setChatState(session, [])
    } catch (err) {
      console.error('Failed to create session:', err)
      toast.error('Failed to create chat session')
    }
  }, [contextRef, setChatState, toast])

  const createSessionWithMessages = useCallback(async (initialMsgs: ChatMessage[]) => {
    if (!showInput) {
      setMessages(initialMsgs)
      return
    }

    if (!(window as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__) {
      setMessages(initialMsgs)
      return
    }

    try {
      const session = await invoke<ChatSession>('create_chat_session', {
        repo_path: contextRef.repo_path,
        prompt_id: contextRef.prompt_id,
        provider: contextRef.provider,
        model: contextRef.model,
      })
      setCurrentSession(session)
      if (initialMsgs.length > 0) {
        const first = {
          ...initialMsgs[0],
          session_id: session.id,
        }
        try {
          await invoke('add_chat_message', {
            session_id: session.id,
            role: first.role,
            content: first.content,
            parent_id: null,
          })
        } catch (err) {
          console.error('Failed to persist initial message:', err)
        }
      }
      const msgs = await invoke<ChatMessage[]>('get_chat_messages', {
        session_id: session.id,
        limit: DEFAULT_PAGE_LIMIT,
      })
      setChatState(session, msgs)
      setHasMoreHistory(
        Boolean(session.message_count && msgs.length < session.message_count)
      )
    } catch (err) {
      console.error('Failed to create session with messages:', err)
      toast.error('Failed to initialize chat')
    }
  }, [
    contextRef,
    setChatState,
    setCurrentSession,
    setMessages,
    setHasMoreHistory,
    showInput,
    toast,
  ])

  // Initialize session on mount
  useEffect(() => {
    if (sessionId) {
      loadSession(sessionId)
    } else if (initialMessages.length > 0) {
      if (showInput) {
        // Create new session with initial messages (output → chat transition)
        createSessionWithMessages(initialMessages)
      } else {
        // One-shot output view, no persistence
        setMessages(initialMessages)
      }
    } else if (showInput) {
      // Start fresh session
      createNewSession()
    }
  }, [
    sessionId,
    initialMessages,
    showInput,
    loadSession,
    createSessionWithMessages,
    createNewSession,
    setMessages,
  ])

  // Scroll to bottom when messages change
  useEffect(() => {
    const node = messagesEndRef.current
    if (typeof node?.scrollIntoView === 'function') {
      node.scrollIntoView({ behavior: 'smooth' })
    }
  }, [messages])

  const handleSend = useCallback(async () => {
    if (!inputValue.trim() || !currentSession) return
    if (sendingRef.current) return
    sendingRef.current = true

    const content = inputValue.trim()
    setInputValue('')
    setLocalLoading(true)
    setIsLoading(true)

    try {
      // Call optional callback
      onSend?.(content)

      // Get the latest message ID from the ref to avoid stale closure issues
      // The ref always contains the most recent messages array
      const latestMessages = messagesRef.current
      let latestMessageId: string | undefined
      for (let i = latestMessages.length - 1; i >= 0; i -= 1) {
        const message = latestMessages[i]
        if (message.session_id === currentSession.id) {
          latestMessageId = message.id
          break
        }
      }

      // Send via Tauri command
      const response = await invoke<SendChatMessageResponse>('send_chat_message', {
        session_id: currentSession.id,
        content,
        parent_id: latestMessageId,
      })

      // Update local state
      addMessage(response.user_message)
      addMessage(response.assistant_message)
      setHasMoreHistory(
        Boolean(
          currentSession.message_count &&
            messagesRef.current.length + 2 < currentSession.message_count
        )
      )
    } catch (err) {
      console.error('Failed to send message:', err)
      toast.error('Failed to send message')
    } finally {
      sendingRef.current = false
      setLocalLoading(false)
      setIsLoading(false)
    }
  }, [
    inputValue,
    currentSession,
    onSend,
    addMessage,
    setHasMoreHistory,
    toast,
    setIsLoading,
  ])  // Note: messages removed - using ref instead

  const handleLoadMore = useCallback(async () => {
    if (!currentSession || isLoadingHistory) return
    if (!(window as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__) {
      return
    }
    setIsLoadingHistory(true)
    try {
      const offset = messagesRef.current.length
      const msgs = await invoke<ChatMessage[]>('get_chat_messages', {
        session_id: currentSession.id,
        limit: DEFAULT_PAGE_LIMIT,
        offset,
      })
      if (msgs.length > 0) {
        prependMessages(msgs)
      }
      const total = currentSession.message_count || 0
      setHasMoreHistory(offset + msgs.length < total)
    } catch (err) {
      console.error('Failed to load more messages:', err)
      toast.error('Failed to load more messages')
    } finally {
      setIsLoadingHistory(false)
    }
  }, [
    currentSession,
    isLoadingHistory,
    prependMessages,
    setHasMoreHistory,
    setIsLoadingHistory,
    toast,
  ])

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
            {hasMoreHistory && (
              <div className="flex justify-center">
                <button
                  className="btn text-xs"
                  onClick={handleLoadMore}
                  disabled={isLoadingHistory}
                >
                  {isLoadingHistory ? 'Loading...' : 'Load earlier messages'}
                </button>
              </div>
            )}
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
 * Sanitize HTML content to prevent XSS attacks.
 * This removes dangerous tags and attributes while preserving safe markdown content.
 */
const sanitizeContent = (content: string): string => {
  if (typeof window === 'undefined') {
    // Server-side rendering fallback - return content as-is
    // DOMPurify only works in browser environment
    return content
  }

  return DOMPurify.sanitize(content, {
    ALLOWED_TAGS: [
      'p', 'br', 'strong', 'em', 'u', 's', 'del',
      'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
      'ul', 'ol', 'li',
      'code', 'pre', 'blockquote',
      'a', 'img',
      'table', 'thead', 'tbody', 'tr', 'th', 'td',
      'hr', 'sup', 'sub'
    ],
    ALLOWED_ATTR: [
      'href', 'title',  // for links
      'src', 'alt', 'title', // for images
      'class', // for syntax highlighting
    ],
    // Allow href and src with specific protocols only
    ALLOWED_URI_REGEXP: /^(?:(?:(?:f|ht)tps?|mailto|tel|callto|cid|xmpp|xxx):|[^a-z]|[a-z+.-]+(?:[^a-z+.-:]|$))/i,
    // Prevent javascript: URLs
    FORBID_ATTR: ['style', 'onerror', 'onload', 'onclick', 'onmouseover'],
    FORBID_TAGS: ['script', 'style', 'iframe', 'form', 'input', 'textarea', 'button'],
    // Keep text content of removed elements
    KEEP_CONTENT: true,
  })
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

  // Sanitize content to prevent XSS from malicious LLM output
  const sanitizedContent = sanitizeContent(message.content)

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
            {isUser ? 'user' : isFirst ? 'output' : 'assistant'}
          </span>
          <span className="text-xs opacity-50" style={{ color: 'var(--dracula-comment)' }}>
            {new Date(message.timestamp).toLocaleTimeString()}
          </span>
        </div>

        {/* Message content - now sanitized to prevent XSS */}
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
              },
              // Override link rendering to add security attributes
              a({ href, children, ...props }) {
                return (
                  <a
                    href={href}
                    target="_blank"
                    rel="noopener noreferrer nofollow"
                    {...props}
                  >
                    {children}
                  </a>
                )
              },
              // Override image rendering to prevent tracking/beacons
              img({ src, alt, ...props }) {
                return (
                  <img
                    src={src}
                    alt={alt}
                    loading="lazy"
                    style={{ maxWidth: '100%', height: 'auto' }}
                    {...props}
                  />
                )
              }
            }}
          >
            {sanitizedContent}
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

export default UnifiedOutputPanel
