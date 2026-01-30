import { ReactNode } from 'react'

export type BadgeVariant = 'critical' | 'high' | 'medium' | 'low' | 'info' | 'default'

interface BadgeProps {
  children: ReactNode
  variant?: BadgeVariant
  className?: string
}

export function Badge({ children, variant = 'default', className = '' }: BadgeProps) {
  const getVariantClass = (v: BadgeVariant) => {
    switch (v) {
      case 'critical':
        return 'badge-critical'
      case 'high':
        return 'badge-high'
      case 'medium':
        return 'badge-medium'
      case 'low':
        return 'badge-low'
      case 'info':
        return 'badge-info'
      default:
        return 'bg-slate-700 text-slate-200 border border-slate-600'
    }
  }

  return (
    <span
      className={`px-3 py-1 rounded text-sm font-medium capitalize ${getVariantClass(variant)} ${className}`}
    >
      {children}
    </span>
  )
}
