# 🎨 Palette's UX Journal
> Critical UX/a11y learnings specific to this codebase only.

## Design System Notes
- Component library: Tailwind CSS 4, @heroicons/react, custom "Dracula" theme
- Key patterns: Custom `btn`, `input`, `card` classes in `index.css`
- Accessibility utilities: `focus-visible` styles are globally defined in `index.css`

## Learnings
<!-- Add entries below using the template -->

## 2026-02-06 — UnifiedOutputPanel Accessibility
**Context:** Auditing `UnifiedOutputPanel` for accessibility.
**Learning:** The chat interface uses custom button components that rely on visual icons (e.g., Unicode arrows) without accessible names.
**Evidence:** The "Send" button was just a `button` with a `span` containing "➤". Screen readers announced "button" or "Black Right-Pointing Pointer".
**Future Action:** Always check icon-only buttons in custom UI components for `aria-label` or `title`.
