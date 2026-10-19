# Changelog
All notable changes to this project are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.6.0] - 2026-10-19
### Added
- Design system (DESIGN.md) with 8px grid, Radix/shadcn aesthetic, and token standardisation
- Motion (Framer Motion) spring animations for overlay entrance, panels, and dropdowns
- CSS cubic-bezier micro-interaction curves on all interactive elements
- First-launch onboarding flow for both apps
- Crash detection popup with opt-in mailto email report
- What is new modal shown once per version after an update
- Experimental UI flag in Settings with app restart prompt
### Fixed
- All Nielsen usability heuristic and WCAG 2.1 AA contrast findings
- SynaptClip splash screen appearing intermittently on launch
- Inconsistent spacing, colour usage, and interactive states across components

## [0.5.4] - 2026-10-17
### Added
- Updates section shows the bundled SynaptCore version, compatibility warnings, and check errors
### Changed
- Bundles SynaptCore 0.2.1; release builds wait for the matching SynaptCore release
### Fixed
- Category tab scrollbar replaced with a thin 2px indicator that no longer covers labels
- Global hotkey now fires correctly on GNOME Wayland via the xdg-desktop-portal GlobalShortcuts interface

## [0.5.3] - 2026-10-16
### Added
- System categories (Links, Email, Phone, File Path, Color, Code) with auto-detection and per-category toggles in Settings
- Deletion confirmation for user-created categories
### Changed
- System category detection replaces the earlier classification trie
### Fixed
- Windows hotkey recorder no longer triggers the current shortcut and closes the panel while recording

## [0.5.2] - 2026-08-28
### Added
- Auto-updater with signed update endpoint for all platforms
- Automatic update check preference and reworked Updates section in Settings
- Crash log written to platform data directory on panic

## [0.5.1] - 2026-08-25
### Added
- Receive notification popup for incoming clips with copy, view, and dismiss options
- NSAlert confirmation dialog when quitting from the tray on macOS
- PRIVACY.md
- CONTRIBUTING.md, SECURITY.md, and GitHub issue templates
### Fixed
- macOS modifier keys (Command, Option) now detected correctly in the hotkey recorder and global shortcut registration
- Cmd+Q and the close button are suppressed on macOS; quit is only available from the tray menu
- Apache 2.0 copyright format corrected to 'Copyright 2026 Aatish Bagal'

## [0.5.0] - 2026-08-21
### Added
- Synapt bridge: detects Synapt automatically by polling its health endpoint every 10 seconds
- Devices section in the panel listing trusted peers when Synapt is running
- Send to device option in the clip card context menu
- Incoming clip listener on port 57322 stores clips received from peers
- Received clips are marked with the sender's device name
- Tray tooltip shows the number of connected devices
- macOS universal build with signing and notarization support
- New app icon built with Icon Composer; tray icon stays transparent
### Changed
- Dev server moved to port 1430 to avoid clashing with Synapt
### Fixed
- App is hidden from the Dock on macOS and runs from the tray only, like Synapt
- Duplicate tray icon from a leftover trayIcon block in tauri.conf.json

## [0.4.1] - 2026-06-04
### Added
- Single system tray icon shared with Synapt via synapt-core, hosted by whichever app launches first
- Auto-start on login toggled from a Settings checkbox, with no manual command required
- GitHub Actions CI and release pipeline
### Fixed
- GNOME Wayland GCH extension detection and dual X11/Wayland clipboard capture
- Dropdown menus use CSS variables instead of a white background with invisible text

## [0.4.0]
### Added
- Global hotkey to show the panel from anywhere
- Auto-start on login via a systemd user service on Linux
- File-based logging and a crash reporter
- First-run onboarding
- Skip list sorted clip index
- Classification trie that auto-categorizes new clips

## [0.3.1]
### Fixed
- Settings opens in a separate window
- Panel recenters every time it is shown

## [0.3.0]
### Added
- Settings page
- Theme system
- Tray tooltip
### Changed
- Visual polish across the panel

## [0.2.0]
### Added
- Search layer: compressed trie, suffix array, Levenshtein fuzzy search, Bloom filter, and match highlighting
- Huffman-compressed clip storage
- Double-ended priority queue for history eviction
- Union-find clip grouping with automatic merging
- Persistent linked list with undo for deleted clips
- Wayland wlroots clipboard backend via wl-paste
- GNOME Wayland clipboard backend via the GCH extension file watcher
- Context menu, category tabs, groups view, and bulk selection mode
- Horizontal panel layout with header, logo, version, and clear all button
### Fixed
- Repeated wl-paste failures

## [0.1.0]
### Added
- Tauri v2 project scaffold with React and TypeScript frontend
- Platform detection for X11, Wayland GNOME, Wayland wlroots, Windows, and macOS
- SQLite storage layer
- Clipboard capture via arboard
- Tauri IPC commands and clipboard event pipeline
- Spotlight-style panel with search that hides on blur
- System tray menu and app logo
