import { type ReactNode } from 'react'
import { Link, useLocation } from 'react-router-dom'

interface TerminalLayoutProps {
  children: ReactNode
}

interface NavItem {
  path: string
  label: string
  shortcut: string
}

export function TerminalLayout({ children }: TerminalLayoutProps) {
  const location = useLocation()

  const navItems: NavItem[] = [
    { path: '/', label: 'home', shortcut: '⌘1' },
    { path: '/scan', label: 'scan', shortcut: '⌘2' },
    { path: '/thinktank', label: 'thinktank', shortcut: '⌘3' },
    { path: '/report', label: 'report', shortcut: '⌘4' },
    { path: '/settings', label: 'settings', shortcut: '⌘5' },
  ]

  const isActive = (path: string) => location.pathname === path

  return (
    <div className="flex h-screen terminal-bg" style={{ backgroundColor: 'var(--dracula-bg)' }}>
      {/* Scanline Effect */}
      <div className="scanline" />

      {/* Sidebar */}
      <aside
        className="w-64 flex flex-col border-r"
        style={{
          backgroundColor: 'var(--dracula-bg)',
          borderColor: 'var(--dracula-comment)'
        }}
      >
        {/* Header */}
        <div
          className="px-4 py-3 border-b"
          style={{ borderColor: 'var(--dracula-comment)' }}
        >
          <div className="flex items-center gap-2">
            <span className="text-terminal-green text-lg">❯</span>
            <h1 className="text-sm font-bold tracking-wider" style={{ color: 'var(--dracula-fg)' }}>
              HQE_WORKBENCH
            </h1>
          </div>
          <div className="text-xs mt-1" style={{ color: 'var(--dracula-comment)' }}>
            v0.1.0 | protocol v3.0
          </div>
        </div>

        {/* Navigation */}
        <nav className="flex-1 py-2">
          <div className="px-3 py-2 text-xs uppercase tracking-wider" style={{ color: 'var(--dracula-comment)' }}>
            Navigation
          </div>

          {navItems.map((item) => (
            <Link
              key={item.path}
              to={item.path}
              className="flex items-center justify-between px-3 py-2 mx-2 text-sm transition-all duration-150"
              style={{
                backgroundColor: isActive(item.path)
                  ? 'var(--dracula-current-line)'
                  : 'transparent',
                borderLeft: isActive(item.path)
                  ? '2px solid var(--dracula-green)'
                  : '2px solid transparent',
                color: isActive(item.path)
                  ? 'var(--dracula-green)'
                  : 'var(--dracula-fg)',
              }}
              onMouseEnter={(e) => {
                if (!isActive(item.path)) {
                  e.currentTarget.style.backgroundColor = 'var(--dracula-current-line)'
                  e.currentTarget.style.color = 'var(--dracula-cyan)'
                }
              }}
              onMouseLeave={(e) => {
                if (!isActive(item.path)) {
                  e.currentTarget.style.backgroundColor = 'transparent'
                  e.currentTarget.style.color = 'var(--dracula-fg)'
                }
              }}
            >
              <span className="flex items-center gap-2">
                <span style={{ color: isActive(item.path) ? 'var(--dracula-green)' : 'var(--dracula-purple)' }}>
                  {isActive(item.path) ? '❯' : '$'}
                </span>
                {item.label}
              </span>
              <span className="text-xs" style={{ color: 'var(--dracula-comment)' }}>
                {item.shortcut}
              </span>
            </Link>
          ))}
        </nav>

        {/* Status Panel */}
        <div
          className="px-3 py-3 border-t text-xs"
          style={{ borderColor: 'var(--dracula-comment)' }}
        >
          <div className="flex items-center gap-2 mb-2">
            <div
              className="w-2 h-2 rounded-full animate-pulse"
              style={{ backgroundColor: 'var(--dracula-green)' }}
            />
            <span style={{ color: 'var(--dracula-comment)' }}>system ready</span>
          </div>
          <div style={{ color: 'var(--dracula-comment)' }}>
            local-first mode
          </div>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 flex flex-col min-w-0">
        {/* Top Bar */}
        <header
          className="h-10 flex items-center px-4 border-b justify-between"
          style={{
            backgroundColor: 'var(--dracula-bg)',
            borderColor: 'var(--dracula-comment)'
          }}
        >
          <div className="flex items-center gap-4 text-xs">
            <span style={{ color: 'var(--dracula-comment)' }}>
              {location.pathname === '/' && '~/welcome'}
              {location.pathname === '/scan' && '~/scan'}
              {location.pathname === '/thinktank' && '~/thinktank'}
              {location.pathname === '/report' && '~/report'}
              {location.pathname === '/settings' && '~/settings'}
            </span>
          </div>
          <div className="flex items-center gap-4 text-xs" style={{ color: 'var(--dracula-comment)' }}>
            <span>rust 1.75+</span>
            <span>|</span>
            <span>tauri 2.0</span>
          </div>
        </header>

        {/* Content Area */}
        <div
          className="flex-1 overflow-hidden relative"
          style={{ backgroundColor: 'var(--dracula-bg)' }}
        >
          {children}
        </div>

        {/* Bottom Status Bar */}
        <footer
          className="h-6 flex items-center px-3 text-xs border-t"
          style={{
            backgroundColor: 'var(--dracula-current-line)',
            borderColor: 'var(--dracula-comment)',
            color: 'var(--dracula-comment)'
          }}
        >
          <span className="text-terminal-green mr-4">NORMAL</span>
          <span className="mr-4">HQE Protocol v3.0</span>
          <span className="ml-auto">-- TERMINAL --</span>
        </footer>
      </main>
    </div>
  )
}
