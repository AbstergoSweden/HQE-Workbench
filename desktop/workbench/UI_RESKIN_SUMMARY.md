# HQE Workbench - Terminal UI Re-skin

## Overview
Complete UI re-skin with a minimalistic terminal-style interface using the Dracula color palette.

## Changes Made

### 1. Color Palette (Dracula Theme)
```css
--dracula-bg: #282a36           /* Background */
--dracula-current-line: #44475a /* Surface/Card backgrounds */
--dracula-selection: #44475a     /* Selection */
--dracula-fg: #f8f8f2           /* Foreground text */
--dracula-comment: #6272a4      /* Muted text/Borders */
--dracula-cyan: #8be9fd         /* Info/Accent */
--dracula-green: #50fa7b        /* Success/Primary */
--dracula-orange: #ffb86c       /* Warning */
--dracula-pink: #ff79c6         /* Highlight */
--dracula-purple: #bd93f9       /* Secondary accent */
--dracula-red: #ff5555          /* Error/Danger */
--dracula-yellow: #f1fa8c       /* Warning/Medium priority */
```

### 2. Typography
- **Font**: JetBrains Mono (Google Fonts)
- **Style**: Monospace throughout
- **Feel**: Terminal/command-line aesthetic

### 3. Components Updated

#### TerminalLayout (New)
- Replaced sidebar navigation with terminal-style menu
- Added scanline animation effect
- Status bar at bottom showing mode and protocol version
- Navigation items with `$` and `❯` prompts
- Keyboard shortcut hints (⌘1-5)

#### WelcomeScreen
- ASCII art header
- Terminal prompt styling (`$`, `❯`)
- Command-style button labels (`open_repository`, `configure_provider`)
- System status panel

#### ScanScreen
- Terminal-style form labels (`--flag-name`)
- Progress bar with gradient
- Checklist-style progress indicators
- Scan phase indicators with checkmarks

#### ReportScreen
- Terminal-style badges for severity
- Monospace health score display
- Grid layout for stats
- Color-coded severity badges

#### SettingsScreen
- Terminal-style profile list
- Command-style button labels
- Form labels as CLI flags (`--name`, `--url`, `--api-key`)
- Better organized grid layout

#### ThinktankScreen
- Terminal-style prompt library
- Monospace prompt names
- AGENT badge for agent prompts
- Command-style execute button

### 4. CSS Features

#### Animations
- `blink`: Cursor blinking animation
- `scanline`: Horizontal scanline effect
- `typewriter`: Text typing effect
- `glow`: Pulsing glow effect
- `fadeIn`: Content fade-in

#### Interactive Elements
- Buttons with terminal-style borders
- Hover effects with glow
- Focus states with cyan outline
- Custom checkbox styling
- Custom range slider

#### Effects
- Scanline overlay (subtle retro terminal feel)
- Grid pattern background
- Glow effects on interactive elements
- Border glow animations

### 5. Build Status
✅ TypeScript compilation successful
✅ Vite build successful
⚠️ Tests need updating for new UI text (expected with UI overhaul)

## Files Modified
- `index.html` - Added JetBrains Mono font
- `src/index.css` - Complete rewrite with Dracula theme
- `src/App.tsx` - Updated to use TerminalLayout
- `src/components/TerminalLayout.tsx` - New component
- `src/components/Layout.tsx` - Old layout (kept for reference)
- `src/screens/WelcomeScreen.tsx` - Terminal styling
- `src/screens/ScanScreen.tsx` - Terminal styling
- `src/screens/ReportScreen.tsx` - Terminal styling
- `src/screens/SettingsScreen.tsx` - Terminal styling
- `src/screens/ThinktankScreen.tsx` - Terminal styling
- `src/test/setup.ts` - Fixed matchMedia mock
- `src/__tests__/thinktank.test.tsx` - Updated for new UI

## Key UI Patterns

### Terminal Prompt Style
```tsx
<div className="flex items-center gap-2">
  <span className="text-terminal-green">❯</span>
  <span>command_name</span>
</div>
```

### Card Style
```tsx
<div className="card" style={{ borderColor: 'var(--dracula-comment)' }}>
  {/* Content */}
</div>
```

### Button Style
```tsx
<button className="btn btn-primary">
  <span className="text-terminal-green">❯</span>
  <span>action_name</span>
</button>
```

### Form Label Style
```tsx
<label className="text-terminal-cyan font-mono text-sm">
  --field-name
</label>
```

## Screenshots Preview

### Welcome Screen
- ASCII art header
- Command-style actions
- System status panel

### Scan Screen
- Terminal progress bars
- Phase checklist
- Command-line style options

### Report Screen
- Monospace health score
- Color-coded severity badges
- Terminal-style stats grid

### Settings Screen
- Profile list with terminal styling
- CLI flag form labels
- Command-style buttons

### Thinktank Screen
- Terminal prompt library
- Monospace prompt names
- Command-style execute button

## Testing Notes
Tests need updates for:
- Button text changes ("Run Prompt" → "Execute Prompt")
- Label text changes ("count" → "--count")
- Checkbox label changes ("Show agent/tool prompts" → "Show agent prompts")

These are expected changes with the UI overhaul and don't affect functionality.
