import { type ReactNode } from 'react'
import { Link, useLocation } from 'react-router-dom'

interface LayoutProps {
  children: ReactNode
}

export function Layout({ children }: LayoutProps) {
  const location = useLocation()
  
  const navItems = [
    { path: '/', label: 'Welcome', icon: '🏠' },
    { path: '/scan', label: 'Scan', icon: '🔍' },
    { path: '/thinktank', label: 'Thinktank', icon: '🧠' },
    { path: '/report', label: 'Report', icon: '📊' },
    { path: '/settings', label: 'Settings', icon: '⚙️' },
  ]

  return (
    <div className="flex h-screen" style={{ background: 'var(--color-bg)' }}>
      <nav 
        className="w-64 flex flex-col border-r"
        style={{ 
          background: 'var(--color-surface)',
          borderColor: 'var(--color-border)'
        }}
      >
        <div 
          className="p-6 border-b"
          style={{ borderColor: 'var(--color-border)' }}
        >
          <h1 className="text-xl font-bold" style={{ color: 'var(--color-text)' }}>
            🌿 HQE Workbench
          </h1>
          <p className="text-xs mt-1" style={{ color: 'var(--color-text-muted)' }}>
            v0.1.0
          </p>
        </div>
        
        <div className="flex-1 py-4">
          {navItems.map((item) => (
            <Link
              key={item.path}
              to={item.path}
              className={`flex items-center gap-3 px-6 py-3 transition-all duration-200 ${
                location.pathname === item.path
                  ? 'border-r-2'
                  : ''
              }`}
              style={{
                color: location.pathname === item.path 
                  ? 'var(--color-primary)' 
                  : 'var(--color-text-muted)',
                background: location.pathname === item.path 
                  ? 'rgba(16, 185, 129, 0.1)' 
                  : 'transparent',
                borderColor: location.pathname === item.path 
                  ? 'var(--color-primary)' 
                  : 'transparent',
              }}
              onMouseEnter={(e) => {
                if (location.pathname !== item.path) {
                  e.currentTarget.style.background = 'rgba(16, 185, 129, 0.05)'
                  e.currentTarget.style.color = 'var(--color-text)'
                }
              }}
              onMouseLeave={(e) => {
                if (location.pathname !== item.path) {
                  e.currentTarget.style.background = 'transparent'
                  e.currentTarget.style.color = 'var(--color-text-muted)'
                }
              }}
            >
              <span role="img" aria-label={item.label}>{item.icon}</span>
              <span>{item.label}</span>
            </Link>
          ))}
        </div>
        
        <div 
          className="p-6 border-t"
          style={{ borderColor: 'var(--color-border)' }}
        >
          <div className="text-xs" style={{ color: 'var(--color-text-muted)' }}>
            <p>HQE Protocol v3.0.0</p>
            <p className="mt-1">Local-first scanning</p>
          </div>
        </div>
      </nav>
      
      <main className="flex-1 overflow-auto p-8">
        {children}
      </main>
    </div>
  )
}
