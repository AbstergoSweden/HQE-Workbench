import { useEffect, useState } from 'react'

export type ToastType = 'success' | 'error' | 'info' | 'warning'

export interface ToastProps {
  id: string
  message: string
  type: ToastType
  duration?: number
  onClose: (id: string) => void
}

export function Toast({ id, message, type, duration = 3000, onClose }: ToastProps) {
  const [isExiting, setIsExiting] = useState(false)

  useEffect(() => {
    const timer = setTimeout(() => {
      setIsExiting(true)
    }, duration)

    return () => clearTimeout(timer)
  }, [duration])

  useEffect(() => {
    if (isExiting) {
      const timer = setTimeout(() => {
        onClose(id)
      }, 300) // Match animation duration
      return () => clearTimeout(timer)
    }
  }, [isExiting, id, onClose])

  const getStyles = () => {
    switch (type) {
      case 'success':
        return {
          bg: 'bg-emerald-900/90',
          border: 'border-emerald-500/50',
          icon: '✅',
          text: 'text-emerald-50'
        }
      case 'error':
        return {
          bg: 'bg-red-900/90',
          border: 'border-red-500/50',
          icon: '❌',
          text: 'text-red-50'
        }
      case 'warning':
        return {
          bg: 'bg-amber-900/90',
          border: 'border-amber-500/50',
          icon: '⚠️',
          text: 'text-amber-50'
        }
      default:
        return {
          bg: 'bg-slate-800/90',
          border: 'border-slate-600/50',
          icon: 'ℹ️',
          text: 'text-slate-50'
        }
    }
  }

  const styles = getStyles()

  return (
    <div
      className={`
        flex items-center gap-3 px-4 py-3 rounded-lg border shadow-lg backdrop-blur-sm
        transition-all duration-300 ease-in-out transform
        ${styles.bg} ${styles.border} ${styles.text}
        ${isExiting ? 'opacity-0 translate-x-full' : 'opacity-100 translate-x-0'}
      `}
      role="alert"
    >
      <span className="text-lg" role="img" aria-hidden="true">
        {styles.icon}
      </span>
      <p className="text-sm font-medium">{message}</p>
      <button
        onClick={() => setIsExiting(true)}
        className="ml-2 opacity-50 hover:opacity-100 transition-opacity"
        aria-label="Close notification"
      >
        ✕
      </button>
    </div>
  )
}
