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

Tailwind extends only transition timing and duration (see Animation System). Colours
are not in the Tailwind theme, so tokens are applied through inline `style` props as
`var(--token)` rather than Tailwind colour classes.

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
| `--muted` | `#919191` | `#666666` | Secondary text, section headings, header actions, icons |

### Border

| Token | Dark | Light | Use |
|---|---|---|---|
| `--border` | `#333333` | `#e0e0e0` | Dividers, row separators, scrollbar thumb, menu outline |

### Accent

| Token | Dark | Light | Use |
|---|---|---|---|
| `--accent` | `#4f8ff7` | `#2160eb` | Active tab underline, focus ring, selected-row bar, accent text |
| `--accent-fill` | `#2563eb` | `#2160eb` | Background behind `--on-accent` text: primary buttons |
| `--accent-soft` | `#1f3a72` | `#dbeafe` | Defined, not used in SynaptClip today |

Tinted variants use `color-mix(in srgb, var(--token) N%, transparent)`: `--accent` 60%
(focus ring), `--danger` 40% (destructive button border) and 10% (destructive hover).

### Semantic

| Token | Dark | Light | Use |
|---|---|---|---|
| `--danger` | `#f16161` | `#d02222` | Clear all, delete, destructive confirmation |
| `--danger-fill` | `#dc2626` | `#d02222` | Background behind `--on-accent` text (no filled danger button exists today) |
| `--success` | `#22c55e` | `#11813b` | Copied confirmation icon |
| `--warning` | `#f59e0b` | `#aa5d05` | Pinned icon, warnings |
| `--warning-bg` | `#3a2e14` | `#fef3c7` | Setup warning banner fill |
| `--warning-border` | `#553c18` | `#fcd34d` | Setup warning banner border |
| `--warning-text` | `#fef08a` | `#78350f` | Text on setup warning banner |
| `--highlight-bg` | `#854d0e` | `#fde68a` | Search match highlight fill |
| `--highlight-text` | `#fef08a` | `#78350f` | Search match highlight text |

### Contrast (WCAG 2.1 AA)

Every text token meets 4.5:1 on each surface it is used on (checked in v0.6.0; the
minimum lightness change that passes was applied to each failing token). Re-check
these pairs whenever a token value changes.

| Pair | Dark | Light |
|---|---|---|
| `--text` on `--bg` | 17.40 | 17.32 |
| `--text` on `--surface-hover` | 14.35 | 15.98 |
| `--muted` on `--bg` | 5.52 | 5.27 |
| `--muted` on `--surface` | 4.93 | 5.74 |
| `--muted` on `--surface-hover` | 4.55 | 4.86 |
| `--on-accent` on `--accent-fill` | 5.17 | 5.33 |
| `--on-accent` on `--danger-fill` | 4.83 | 5.34 |
| `--accent` on `--bg` | 5.47 | 4.89 |
| `--accent` on `--surface-hover` | 4.51 | 4.51 |
| `--danger` on `--bg` | 5.49 | 4.90 |
| `--danger` on `--surface` | 4.90 | 5.34 |
| `--danger` on `--surface-hover` | 4.53 | 4.52 |
| `--warning` on `--bg` | 8.10 | 4.50 |
| `--warning` on `--surface` | 7.23 | 4.91 |
| `--success` on `--bg` | 7.64 | 4.56 |
| `--success` on `--surface` | 6.81 | 4.97 |

Why there are fill tokens: no single blue can carry white text on top of it (needs a
dark blue) and also read as accent text on the dark `--surface-hover` (needs a light
blue). `--accent` is for text, borders and focus; `--accent-fill` is for backgrounds
behind `--on-accent` text. The same split applies to `--danger` and `--danger-fill`.
Muted text never sits on `--border`; badges that did so were removed.

### Theme-independent

Declared once in a separate `:root` block because the value is the same in both themes.

| Token | Value | Use |
|---|---|---|
| `--on-accent` | `#ffffff` | Text on accent- or danger-filled buttons |

All other tokens except the fills differ between dark and light. The themed token set and
values are identical to Synapt's; Synapt additionally defines `--backdrop`.

### Hardcoded colours outside tokens

None.

## Typography

Font family: `system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif`
Monospace: `monospace` (clip content preview, paths in Settings)

Scale:

- 11px - section headings, Settings helper text
- 12px (`text-xs`) - clip metadata, timestamps, header title and actions, tab labels, Settings body text
- 13px - clip content preview (monospace), device names, buttons, confirmation message, Select options
- 14px (`text-sm`) - search input and other `.input` fields, empty-state titles
- 16px (`text-base`) - onboarding title

No component mixes more than two of these sizes (`ClipCard` is 13px content and 12px metadata).

Font weights in use: 400 (default), 500 (header title, onboarding title, section headings, primary buttons, empty-state titles)

Line heights in use: Tailwind defaults (16px with `text-xs`, 20px with `text-sm`, 24px with `text-base`) plus 16px on buttons and headings and 20px on inputs and empty states.

Letter spacing in use: 0.06em (section headings)

## Spacing Scale

Base unit: 8px
Approved values: 4, 8, 12, 16, 24, 32, 48px
Do not use values outside this scale. If a value between these is needed, use the nearest approved value.

Deliberate exceptions, each fixed by a pattern in this document: buttons use `6px 14px`
padding, inputs use `10px 14px`, and category tabs and context-menu items use `py-1.5` (6px).

Off-scale values left in the code: none.

## Border Radius

- 8px - panel root, Settings root, receive popup, inputs and the search field
- 6px - buttons, inline confirmation container, Select popup
- 4px (`rounded`) - icon buttons, chips, context menu
- 2px - compression ratio bar
- 1px - category scrollbar thumb, underline loader

Rule: 8px for containers at window level or floating over the panel and for text fields,
6px for buttons, the confirmation container and Select popup, 4px for icon buttons,
chips and menus.
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

The header and the search strip share `--bg` with no line between them; clip rows are
separated by spacing only. No component uses a shadow.

Current code differs:

- The category tab strip and the Devices section keep a 1px `--border` bottom, because the list scrolls beneath them.

## Interactive States

Every interactive element must implement these states consistently:

- Default: `--surface` fill or transparent, `--text` or `--muted` label
- Hover: `--surface-hover` fill
- Active/Pressed: `--surface-hover` fill, `--accent` label
- Focus: `--accent` - visible focus ring using accent colour at reduced opacity
- Disabled: opacity 0.4, no pointer events

Implementation: shared classes in the `@layer components` block of `src/styles.css`. Use
them instead of re-creating these styles inline.

| Class | Pattern |
|---|---|
| `.btn` + `.btn-primary` / `.btn-secondary` / `.btn-destructive` / `.btn-ghost` | Buttons (see Component Patterns); `.btn-ghost` is the borderless muted "Cancel" / "Later" action |
| `.icon-button` (+ `.icon-button-labeled`, `.icon-button-danger`) | Header and inline icon actions, optionally with a text label |
| `.input` | Input / Search Bar; also applied to a `<label>` wrapping an icon and a bare `<input>` (uses `:focus-within`) |
| `.list-row` with `data-selected` | Result and clip rows |
| `.section-heading` | Section Heading |
| `.empty-state`, `.empty-state-title`, `.empty-state-detail` | Empty State (via the `EmptyState` component) |
| `.error-text` | One-line plain-language error next to the action that failed |

A global `:focus-visible` rule draws a 2px `--accent` ring at 60% with a 1px offset, so
keyboard focus is always visible and mouse clicks never show it. Tailwind's default
`transition` timing is set to `--ease-standard` / 150ms in `tailwind.config.js`.

Remaining difference: list rows still use selection-follows-hover (`onMouseEnter` moves
the keyboard selection), so hover and selection look the same.

## Error Messages

Errors from Tauri commands are written for logs ("peer not online", "os error 2"). Never
show them raw. `src/utils/errors.ts` maps known causes to plain sentences that say what
happened and what to do (`describeError` in catch handlers, which also logs the raw error;
`plainError` when rendering), and falls back to a sentence written for the action that
failed. Show the result with `.error-text` (`role="alert"`) where the action lives, and
never let an action fail silently.

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

Tokens (in the stylesheet as custom properties, mirrored as Tailwind
`ease-*` / `duration-*` utilities):

| Token | Value | Tailwind |
|---|---|---|
| `--ease-standard` | `cubic-bezier(0.25, 0.1, 0.25, 1.0)` | `ease-standard` |
| `--ease-decelerate` | `cubic-bezier(0.0, 0.0, 0.2, 1.0)` | `ease-decelerate` |
| `--ease-accelerate` | `cubic-bezier(0.4, 0.0, 1.0, 1.0)` | `ease-accelerate` |
| `--duration-micro` | `100ms` | `duration-micro` |
| `--duration-short` | `150ms` | `duration-short` |
| `--duration-medium` | `200ms` | `duration-medium` |

Reduced motion: a global `prefers-reduced-motion: reduce` block collapses every CSS
animation and transition to one 0.01ms frame. Infinite loaders run once and hold a
static end frame instead of flickering. Elements marked `data-motion-timer` are exempt
because their animation is a functional countdown, not decoration.
Motion (JS) animations must check reduced motion themselves (`useReducedMotion()`).

Motion presets live in `src/animations.ts` (identical in Synapt and SynaptClip):
`SPRING`, `SPRING_GENTLE`, `OVERLAY_ENTER`, `PANEL_ENTER`, `RESULT_ITEM(index)`, `FADE_IN`,
`INLINE_CONFIRM`, `ITEM_EXIT`. Import from there; do not define one-off springs in components.

The window is shown and hidden from Rust, never unmounted, so the root entrance is
replayed on every window focus by `useEntranceOnShow` (`src/hooks/useEntranceOnShow.ts`)
through animation controls rather than by re-mounting. App roots are wrapped in
`<MotionConfig reducedMotion="user">`, which drops transform animation under reduced
motion; height animation (`INLINE_CONFIRM`) checks `useReducedMotion()` itself.

Currently in use:

- Panel root: `PANEL_ENTER`, replayed on focus
- Clip rows (history and search results): `RESULT_ITEM`, stagger capped at the first 8 rows, `ITEM_EXIT` on removal; the history list remounts per tab
- Receive popup: slides up from `translateY(100%)` on `SPRING`, exits on a stiffer spring through `AnimatePresence`
- Context menu: scale 0.95 to 1 with opacity from the top-left, `SPRING_GENTLE`
- Devices section: `FADE_IN`
- App wrapper fade-in: `.panel-root`, 150ms `--ease-standard`
- Receive popup auto-dismiss bar: `8000ms linear` (a countdown, pauses on hover; marked `data-motion-timer`)
- Popup button opacity: `120ms ease`
- Underline loader: `underline-load 1.2s ease-in-out infinite`
- Search spinner: Tailwind `animate-spin`
- Tailwind `transition-colors` (150ms, `--ease-standard` by default) on rows and buttons

Tailwind `transition-colors` and the `.panel-root` fade now use `--ease-standard`; no transition uses another curve.

## Z-Index

- 50 (`z-50`) - context menu
- 500 - receive popup
- 1000 - Select popup

## Panel Layout

- Main window: 680x480, frameless, always on top, not resizable, hidden until summoned
- Settings window: separate decorated window, 540x620, resizable, loaded at `index.html#settings`
- Structure, top to bottom: 40px header, optional clear-history confirmation, optional
  action error line, optional setup warning banner, search field on the header's `--bg`,
  category tab strip, optional Devices section, scrolling clip list

The panel is meant to be anchored near the tray on macOS, float near the tray on
Windows, and be positioned near the tray on Linux.

Current code differs: the panel is centred on every show (`window.center()` in
`commands.rs`), on all platforms. It is a Spotlight-style centred panel today, not tray-anchored.

### Category Tabs

- Horizontally scrollable strip on `--surface`, 1px `--border` bottom
- Thin 2px scrollbar indicator from the global scrollbar rules; the strip overrides the thumb to 1px radius and `--muted` on hover
- `padding-bottom: 2px` reserves room for the indicator so it never covers labels
- Tab: `px-3 py-1.5`, 12px label; active tab `--text` with a 2px `--accent` bottom border, inactive `--muted` with a transparent border
- The label is a `button` (`role="tab"`) so tabs work from the keyboard; system categories read "Name (auto)" in plain text
- A user category's delete control shows on hover, on the active tab, and on keyboard focus

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
- Scrollbar styling: thin (4px vertical, 2px horizontal) via `::-webkit-scrollbar`, with `scrollbar-width: thin` as the fallback; applied globally

Current code differs: `shadow` is not disabled, so the Tauri default applies.

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

Implemented by `.list-row` with `data-selected`, for both keyboard and bulk selection. Content is 13px monospace and all metadata is 12px `--muted` text (category, sender, size): no badges, no decorative bars.

### Section Heading

- Font size: 11px
- Font weight: 500
- Color: `var(--muted)`
- Text transform: uppercase
- Letter spacing: 0.06em
- Margin bottom: 8px

Implemented by `.section-heading`. Inside a flex column with `gap-2`, add `mb-0` so the gap supplies the 8px.

### Button - Primary

- Background: `var(--accent-fill)`
- Color: `var(--on-accent)`
- Border radius: 6px
- Padding: 6px 14px
- Font size: 13px
- Font weight: 500
- Hover: opacity 0.85

Implemented by `.btn .btn-primary`. The fill is `--accent-fill`, not `--accent`, so white text passes AA in the dark theme.

### Button - Secondary

- Background: transparent
- Color: `var(--text)`
- Border: 1px solid `var(--border)`
- Border radius: 6px
- Padding: 6px 14px
- Font size: 13px
- Hover: background `var(--surface-hover)`

Implemented by `.btn .btn-secondary`. A toggle in its on state (segmented tabs, hotkey recorder) adds the Active style: `--surface-hover` fill, `--accent` label.

### Button - Destructive

- Same as secondary but color: `var(--danger)`, border-color: `var(--danger)` at 0.4 opacity
- Hover: background `var(--danger)` at 0.1 opacity

Implemented by `.btn .btn-destructive`.

### Empty State

- Centered vertically and horizontally in the available space
- Primary text: `var(--text)`, 14px, font-weight 500
- Secondary text: `var(--muted)`, 13px, margin-top 4px
- No icons unless they directly aid comprehension
- No decorative elements

Implemented by the `EmptyState` component (`title`, optional `detail`, optional `action`).

### Input / Search Bar

- Background: `var(--surface)`
- Border: 1px solid `var(--border)`
- Border radius: 8px
- Padding: 10px 14px
- Font size: 14px
- Color: `var(--text)`
- Placeholder color: `var(--muted)`
- Focus: border-color `var(--accent)`, no box-shadow

Implemented by `.input`. The search bar is the full-width `.input` inside a `--bg` strip (`px-3 pb-2`) directly under the header, so header and search read as one surface.

### Inline Confirmation

- Same visual weight as secondary button
- Confirm action: `var(--accent)` text
- Cancel action: `var(--muted)` text
- Keyboard: Enter confirms, Escape cancels

Implemented by `InlineConfirm` with `tone="danger"` (destructive confirm) or the default tone (accent confirm), and a `.btn-ghost` Cancel.

### Context Menu

- `--surface` fill, 1px `--border`, 4px radius, minimum width 180px, `z-50`
- Items `px-3 py-1.5` 12px; hover `--surface-hover`

No shadow. Items are buttons; the Assign Category submenu opens on hover, click or Enter, and Escape closes the menu.

### Receive Popup

- Fixed 16px from the left, right, and bottom edges of the panel, `z-index: 500`
- `--surface` fill, 8px radius, 1px accent border at 40%
- Copy, view, and dismiss actions; auto-dismisses after 8 seconds with a countdown bar that pauses on hover

`--border` outline and no shadow. Copy is `.btn-primary`, View `.btn-secondary`, Dismiss `.btn-ghost`; Enter copies and Escape dismisses.

## What Not To Do

- Do not use accent colour on card borders, section dividers, or status indicators
- Do not add decorative shadows to panel cards or list items
- Do not use more than 2 font sizes within a single component
- Do not create new spacing values outside the approved scale
- Do not add border-radius values not listed in this document
- Do not use colour to convey information without a text label alongside it
- Do not add animations that exceed 300ms
- Do not use emojis anywhere in the UI
- Do not show a raw backend error, and do not let an action fail silently (see Error Messages)

Existing violations to clean up:

- The underline loader loops every 1.2s. It is a progress indicator, not a transition, and holds still under reduced motion.

