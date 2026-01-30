import { ReactNode } from 'react'

interface CardProps {
  children: ReactNode
  className?: string
  style?: React.CSSProperties
  onClick?: () => void
}

export function Card({ children, className = '', style, onClick }: CardProps) {
  return (
    <div
      onClick={onClick}
      className={`card ${className} ${onClick ? 'cursor-pointer' : ''}`}
      style={style}
    >
      {children}
    </div>
  )
}
