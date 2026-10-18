# SynaptClip Design System

This document is the single source of truth for all design decisions in SynaptClip.
All UI development must read this file before writing any component, CSS, or layout code.

Token values below were read from `src/styles.css`, `tailwind.config.js`, and the
components under `src/`. Where the current code does not yet match the rule, the
gap is listed under "Current code differs" so it can be fixed, not copied.

## Guiding Principles

1. Functional over decorative - every visual element serves a purpose
2. Consistent over clever - use the system, do not invent exceptions
3. Platform-aware - the panel must feel at home on macOS, Windows, and Linux
4. No decorative accents - the accent colour is reserved for interactive and active states only, never used for decoration, borders on cards, or status indicators
5. No status dots or pill badges - use text labels instead

## Colour Tokens

Defined in `src/styles.css`. The theme is chosen by `data-theme` on `<html>`. Only
`dark` and `light` blocks exist in CSS. For `system`, `applyTheme` in `App.tsx`
resolves the OS preference once with `matchMedia` and writes `dark` or `light`.
The bare `:root` block repeats the dark values as the default.

Tailwind is not extended (`theme.extend` is empty), so tokens are applied through
inline `style` props as `var(--token)` rather than Tailwind colour classes.

Current code differs: the `system` theme does not follow OS changes while the app is
running, because no `prefers-color-scheme` listener or media-query block exists.

### Background

| Token | Dark | Light | Use |
|---|---|---|---|
| `--bg` | `#1a1a1a` | `#f5f5f5` | Body, panel root, header bar |

### Surface

| Token | Dark | Light | Use |
|---|---|---|---|
| `--surface` | `#242424` | `#ffffff` | Search bar, category tab strip, context menu, receive popup, inputs |
| `--surface-hover` | `#2a2a2a` | `#ececec` | Hovered and selected clip rows, menu hover, badge fill |

### Text

| Token | Dark | Light | Use |
|---|---|---|---|
| `--text` | `#ffffff` | `#111111` | Primary text, clip content, active tab label |
| `--muted` | `#888888` | `#666666` | Secondary text, section headings, header actions, icons |

### Border

| Token | Dark | Light | Use |
|---|---|---|---|
| `--border` | `#333333` | `#e0e0e0` | Dividers, row separators, scrollbar thumb, menu outline |

### Accent

| Token | Dark | Light | Use |
|---|---|---|---|
| `--accent` | `#3b82f6` | `#2563eb` | Active tab underline, primary buttons, bulk-selected row border |
| `--accent-soft` | `#1f3a72` | `#dbeafe` | Compression ratio bar on clip cards |

Tinted accent variants use `color-mix(in srgb, var(--accent) 40%, transparent)` (receive
popup border), with a hardcoded `rgba(59, 130, 246, 0.4)` fallback for older WebKit.

### Semantic

| Token | Dark | Light | Use |
|---|---|---|---|
| `--danger` | `#ef4444` | `#dc2626` | Clear all, delete, destructive confirmation |
| `--success` | `#22c55e` | `#16a34a` | Copied confirmation icon |
| `--warning` | `#f59e0b` | `#d97706` | Pinned icon, warnings |
| `--warning-bg` | `#3a2e14` | `#fef3c7` | Setup warning banner fill |
| `--warning-border` | `#553c18` | `#fcd34d` | Setup warning banner border |
| `--warning-text` | `#fef08a` | `#78350f` | Text on setup warning banner |
| `--highlight-bg` | `#854d0e` | `#fde68a` | Search match highlight fill |
| `--highlight-text` | `#fef08a` | `#78350f` | Search match highlight text |

The token set and values are currently identical to Synapt's. No token has the same
value in dark and light.

### Hardcoded colours outside tokens

- `#fff` / `#ffffff` - text on accent- and danger-filled buttons (`InlineConfirm`, `Onboarding`, `Select`, `ClipReceivePopup`)
- `text-blue-400` / `hover:text-blue-300` - GCH extension install link in Settings
- `rgba(59, 130, 246, 0.4)` - receive popup border fallback (dark accent only; wrong in light theme)
- Shadows: `rgba(0, 0, 0, 0.5)` context menu, `rgba(0, 0, 0, 0.4)` receive popup, `rgba(0, 0, 0, 0.3)` Select popup

## Typography

Font family: `system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif`
Monospace: `monospace` (clip content preview)

Scale:

- 9px - item count badge on category tabs
- 10px - category badges on clip rows
- 11px - clip metadata, Devices section heading, device action buttons, Settings section headings, receive popup header
- 12px (`text-xs`) - header title and actions, tab labels, timestamps, buttons, Settings body text
- 13px - clip content preview (monospace), device names, confirmation message, Select options
- 14px (`text-sm`) - search input, empty-state messages
- 16px (`text-base`) - onboarding title

Font weights in use: 400 (default), 500 (`font-medium`; header title, onboarding title, Settings labels)

Line heights in use: Tailwind defaults (16px with `text-xs`, 20px with `text-sm`, 24px with `text-base`),
plus `lineHeight: 1` on the tab delete control and `lineHeight: 0` on the popup close button.

Letter spacing in use: 0.05em (`tracking-wider` on Settings headings, popup header), 0.06em (Devices section heading)

## Spacing Scale

Base unit: 8px
Approved values: 4, 8, 12, 16, 24, 32, 48px
Do not use values outside this scale. If a value between these is needed, use the nearest approved value.

Off-scale values currently in use, flagged for cleanup in Prompt 2:

- 2px - `py-0.5`, `gap-0.5`, `p-0.5` (badges, text stacks, banner close button)
- 3px - `padding: 3px 8px` (device action button), `padding: 3px 10px` (confirmation buttons)
- 5px - `padding: 5px 0` (popup actions), `marginRight: 5` (compression bar)
- 6px - `py-1.5` (tabs, menu items, device rows), `gap-1.5`, `marginTop: 6`
- 10px - `py-2.5` (clip rows), `marginTop: 10`, `padding: 3px 10px`
- 14px - `padding: 12px 14px 0` and `-14px` side margins (receive popup)
- 20px - `gap-5` (Settings sections, onboarding)

## Border Radius

- 8px - panel root, Settings root, receive popup
- 6px - inline confirmation container, Select popup
- 4px (`rounded`) - header buttons, icon buttons, badges, context menu, inputs, device action buttons
- 2px - compression ratio bar
- 1px - category scrollbar thumb, underline loader
- `rounded-full` - 7px online/offline status dot in the Devices section

Rule: 8px for containers at window level or floating over the panel, 4px for controls
and menus inside them, 6px only for the confirmation container and Select popup.
Progress and scroll indicators use 1-2px.

## Elevation and Surfaces

SynaptClip uses background colour differentiation for elevation - no shadows, no borders between surface layers.

SynaptClip is a persistent panel rather than a transient launcher: while open it is the
user's working surface, so the clip list sits directly on the base background and only
chrome and floating elements step up.

Surface hierarchy:

1. `--bg` - panel root, header bar, clip list background
2. `--surface` - search bar, category tab strip, context menu, receive popup, Settings inputs
3. `--surface-hover` - hovered and selected clip rows, menu item hover, badge fill

Current code differs:

- Layers are also separated by 1px `--border` lines: header, search bar, tab strip, Devices section, and every clip row.
- Context menu (`0 6px 16px rgba(0,0,0,0.5)`), receive popup (`0 4px 16px rgba(0,0,0,0.4)`), and Select popup (`0 4px 12px rgba(0,0,0,0.3)`) use shadows.

## Interactive States

Every interactive element must implement these states consistently:

- Default: `--surface` fill or transparent, `--text` or `--muted` label
- Hover: `--surface-hover` fill
- Active/Pressed: `--surface-hover` fill, `--accent` label
- Focus: `--accent` - visible focus ring using accent colour at reduced opacity
- Disabled: opacity 0.4, no pointer events

Current code differs:

- Clip rows and menu items set hover through `onMouseEnter` / `onMouseLeave` style writes rather than CSS.
- Header and icon buttons declare `transition-colors` but no hover colour.
- No custom focus style exists; the panel root and search input set `outline: none`.
- The category tab delete control is hidden until hover (`opacity-0 group-hover:opacity-100`), so it cannot be found by keyboard or touch.

## Animation System

Principles:

- All animations use Apple-style critically damped spring or `cubic-bezier(0.25, 0.1, 0.25, 1.0)`
- No bounce - stiffness: 300, damping: 30 for spring animations
- Duration guidelines:
  - Micro-interactions (hover, focus): 100-150ms, `cubic-bezier(0.25, 0.1, 0.25, 1.0)`
  - Panel/overlay entrance: 200ms spring
  - Content transitions: 150ms `cubic-bezier(0.25, 0.1, 0.25, 1.0)`
  - Never exceed 300ms for any UI animation
- Respect `prefers-reduced-motion`: all animations must be disabled when this media query is set

Motion library: Framer Motion (`motion` package) for spring and presence animations.
CSS cubic-bezier for micro-interactions and hover states.

Currently in use:

- Panel fade-in: `opacity 120ms ease-out` (`.panel-root`)
- Receive popup: slide in `200ms ease-out`, slide out `150ms ease-in`
- Receive popup auto-dismiss bar: `8000ms linear` (a countdown, pauses on hover)
- Popup button opacity: `120ms ease`
- Underline loader: `underline-load 1.2s ease-in-out infinite`
- Search spinner: Tailwind `animate-spin`
- Tailwind `transition-colors` (150ms, `cubic-bezier(0.4, 0, 0.2, 1)`) on rows and buttons

Current code differs:

- `motion` / Framer Motion is not a dependency yet.
- No transition uses the specified cubic-bezier.
- `prefers-reduced-motion` is not handled anywhere.

## Z-Index

- 50 (`z-50`) - context menu
- 500 - receive popup
- 1000 - Select popup

## Panel Layout

- Main window: 680x480, frameless, always on top, not resizable, hidden until summoned
- Settings window: separate decorated window, 540x620, resizable, loaded at `index.html#settings`
- Structure, top to bottom: 40px header, optional setup warning banner, 48px search bar,
  category tab strip, optional Devices section, scrolling clip list

The panel is meant to be anchored near the tray on macOS, float near the tray on
Windows, and be positioned near the tray on Linux.

Current code differs: the panel is centred on every show (`window.center()` in
`commands.rs`), on all platforms. It is a Spotlight-style centred panel today, not tray-anchored.

### Category Tabs

- Horizontally scrollable strip on `--surface`, 1px `--border` bottom
- Thin 2px scrollbar indicator: WebKit `::-webkit-scrollbar` height 2px, thumb `--border` with 1px radius, `--muted` on hover; Firefox fallback `scrollbar-width: thin`
- `padding-bottom: 2px` reserves room for the indicator so it never covers labels
- Tab: `px-3 py-1.5`, 12px label; active tab `--text` with a 2px `--accent` bottom border, inactive `--muted` with a transparent border

## Platform Considerations

### macOS

- Window is frameless
- No Dock icon (NSApplication activation policy: Accessory)
- Tray icon in menu bar - single icon shared with Synapt
- Corner radius on the panel: 8px CSS matches system expectation
- System accent colour not used - app uses its own accent token

Current code differs: the main window is not transparent (`transparent` is unset and
`body` paints `--bg`), so the 8px CSS radius on the panel root sits inside a square window.

### Windows

- Window is frameless
- Tray icon in system tray notification area
- No window shadow (disabled to avoid rendering artifacts)
- Scrollbar styling: thin (2px) using CSS scrollbar-width

Current code differs: `shadow` is not disabled, so the Tauri default applies. Only the
category strip has thin scrollbar styling; the clip list uses the default scrollbar.

### Linux

- X11: standard window compositing
- Wayland GNOME: requires AppIndicator extension for tray; clipboard capture needs the GCH extension
- Wayland wlroots: native tray support
- No window decorations on the main panel

## Component Patterns

### Clip Row / List Item

- Height: 48px minimum
- Padding: 12px 16px
- Selected state: background `var(--surface-hover)`, left border 2px `var(--accent)`
- Hover state: background `var(--surface-hover)`
- No card borders between list items - separation by spacing only

Current code differs (`ClipCard.tsx`):

- Padding is `px-3 py-2.5` (12px 10px).
- Selected state is `--surface-hover` only, with no left accent border.
- Rows are separated by a 1px bottom border, which turns `--accent` when the row is bulk-selected.
- Content preview is 13px monospace. Metadata is 11-12px `--muted`. Category badges are 10px bordered pills: `--muted` for system categories, `--accent` text and border for user categories.

### Section Heading

- Font size: 11px
- Font weight: 500
- Color: `var(--muted)`
- Text transform: uppercase
- Letter spacing: 0.06em
- Margin bottom: 8px

Current code differs: weight is 400 everywhere. Settings headings use `tracking-wider`
(0.05em). The Devices heading uses 0.06em with `pt-2 pb-1` (8px / 4px).

### Button - Primary

- Background: `var(--accent)`
- Color: `#ffffff`
- Border radius: 6px
- Padding: 6px 14px
- Font size: 13px
- Font weight: 500
- Hover: opacity 0.85

Current code differs: accent-filled buttons (onboarding, receive popup) use 4px radius,
12px text, and no hover rule.

### Button - Secondary

- Background: transparent
- Color: `var(--text)`
- Border: 1px solid `var(--border)`
- Border radius: 6px
- Padding: 6px 14px
- Font size: 13px
- Hover: background `var(--surface-hover)`

Current code differs: Settings buttons use `--surface` fill, 4px radius, `px-2 py-1`, and
12px text. Header actions are borderless `--muted` text buttons (`px-2 py-1 rounded text-xs`).
Neither has a hover state.

### Button - Destructive

- Same as secondary but color: `var(--danger)`, border-color: `var(--danger)` at 0.4 opacity
- Hover: background `var(--danger)` at 0.1 opacity

Current code differs: the header "Clear all" is borderless `--danger` text. The deletion
confirm in `InlineConfirm` is a solid `--danger` fill with `#fff` text.

### Empty State

- Centered vertically and horizontally in the available space
- Primary text: `var(--text)`, 14px, font-weight 500
- Secondary text: `var(--muted)`, 13px, margin-top 4px
- No icons unless they directly aid comprehension
- No decorative elements

Current code differs (`Panel.tsx`, `ClipList.tsx`): a single 14px `--muted` line at weight 400, with no secondary text.

### Input / Search Bar

- Background: `var(--surface)`
- Border: 1px solid `var(--border)`
- Border radius: 8px
- Padding: 10px 14px
- Font size: 14px
- Color: `var(--text)`
- Placeholder color: `var(--muted)`
- Focus: border-color `var(--accent)`, no box-shadow

Current code differs: the search bar is a full-width 48px strip (`px-3`, `gap-2`) with a
bottom border only, no radius, and no focus state. Settings inputs use 4px radius and no focus state.

### Inline Confirmation

- Same visual weight as secondary button
- Confirm action: `var(--accent)` text
- Cancel action: `var(--muted)` text
- Keyboard: Enter confirms, Escape cancels

Current code differs (`InlineConfirm.tsx`): the container is `--surface` with a `--border`
outline, 6px radius, and `8px 12px` padding. Confirm is a solid `--danger` button because it
guards deletions. Buttons are 12px with `3px 10px` padding.

### Context Menu

- `--surface` fill, 1px `--border`, 4px radius, minimum width 180px, `z-50`
- Items `px-3 py-1.5` 12px; hover `--surface-hover`

Current code differs: uses `box-shadow: 0 6px 16px rgba(0,0,0,0.5)`.

### Receive Popup

- Fixed 16px from the left, right, and bottom edges of the panel, `z-index: 500`
- `--surface` fill, 8px radius, 1px accent border at 40%
- Copy, view, and dismiss actions; auto-dismisses after 8 seconds with a countdown bar that pauses on hover

Current code differs: uses an accent border (decorative accent) and a drop shadow.

## What Not To Do

- Do not use accent colour on card borders, section dividers, or status indicators
- Do not add decorative shadows to panel cards or list items
- Do not use more than 2 font sizes within a single component
- Do not create new spacing values outside the approved scale
- Do not add border-radius values not listed in this document
- Do not use colour to convey information without a text label alongside it
- Do not add animations that exceed 300ms
- Do not use emojis anywhere in the UI

Existing violations to clean up:

- Status dots: Devices section online indicator (7px, `--accent` when online).
- Accent borders: bulk-selected clip row bottom border, receive popup outline, user-category badges on clip rows.
- Pill badges: category badges on clip rows and the category count badge.
- Hardcoded Tailwind colours: `text-blue-400` / `hover:text-blue-300` on the GCH install link.
- More than 2 font sizes in one component: `ClipCard` (13px, 12px, 11px, 10px).
