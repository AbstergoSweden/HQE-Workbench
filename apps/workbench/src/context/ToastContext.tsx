import { createContext, useContext, useState, useCallback, ReactNode } from 'react'
import { createPortal } from 'react-dom'
import { Toast, ToastType } from '../components/ui/Toast'

interface ToastItem {
  id: string
  message: string
  type: ToastType
  duration?: number
}

interface ToastContextType {
  showToast: (message: string, type?: ToastType, duration?: number) => void
  success: (message: string, duration?: number) => void
  error: (message: string, duration?: number) => void
  info: (message: string, duration?: number) => void
  warning: (message: string, duration?: number) => void
}

const ToastContext = createContext<ToastContextType | undefined>(undefined)

export function useToast() {
  const context = useContext(ToastContext)
  if (!context) {
    throw new Error('useToast must be used within a ToastProvider')
  }
  return context
}

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<ToastItem[]>([])

  const removeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((toast) => toast.id !== id))
  }, [])

  const showToast = useCallback((message: string, type: ToastType = 'info', duration = 3000) => {
    const id = Math.random().toString(36).substring(2, 9)
    setToasts((prev) => [...prev, { id, message, type, duration }])
  }, [])

  const success = useCallback((message: string, duration?: number) => showToast(message, 'success', duration), [showToast])
  const error = useCallback((message: string, duration?: number) => showToast(message, 'error', duration), [showToast])
  const info = useCallback((message: string, duration?: number) => showToast(message, 'info', duration), [showToast])
  const warning = useCallback((message: string, duration?: number) => showToast(message, 'warning', duration), [showToast])

  return (
    <ToastContext.Provider value={{ showToast, success, error, info, warning }}>
      {children}
      {createPortal(
        <div className="fixed top-4 right-4 z-50 flex flex-col gap-2">
          {toasts.map((toast) => (
            <Toast
              key={toast.id}
              {...toast}
              onClose={removeToast}
            />
          ))}
        </div>,
        document.body
      )}
    </ToastContext.Provider>
  )
}
