# SynaptClip — Development Roadmap

---

## Overview

SynaptClip is a clipboard manager for Linux (X11 and Wayland) and Windows, built with Rust and Tauri. It captures clipboard history, provides fast DSA-powered search, and optionally integrates with Synapt for cross-device clipboard sharing over LAN.

This roadmap covers all versions from initial scaffold to v1.0, which is the first publicly installable release with full Synapt compatibility.

---

## Platform Targets

| Platform | Session | Clipboard Backend | Status |
|---|---|---|---|
| Linux (Fedora) | X11 | arboard (direct) | Primary — v0.1 |
| Linux (Fedora) | Wayland, wlroots (Sway, Hyprland) | wl-paste --watch subprocess | v0.3 |
| Linux (Fedora) | Wayland, GNOME (Mutter) | GCH extension file watcher via inotify | v0.3 |
| Windows 10/11 | Win32 | arboard (direct) | v0.4 |
| macOS | — | — | Not planned |

### Single binary, runtime detection

All platform backends are compiled into one binary. At launch, the app detects the session type and compositor, then selects the appropriate clipboard backend automatically. Users do not configure this manually.

Detection order:

1. Check `XDG_SESSION_TYPE`
2. If Wayland: check for wlroots `zwlr_data_control_v1` protocol availability
3. If wlroots available: use `wl-paste --watch` subprocess
4. If not (GNOME, KDE): use GCH file watcher via inotify
5. If not Wayland: use arboard (X11 or Windows)

### GNOME Wayland requirement

On GNOME Wayland, background clipboard access is not possible for regular applications due to Mutter's security model. The Clipboard History extension by alexsaveau.dev (`clipboard-history@alexsaveau.dev`) runs inside GNOME Shell with elevated access and writes clipboard entries to disk. SynaptClip watches that file via inotify to receive new entries.

This extension must be installed by the user. SynaptClip detects its absence at runtime and displays a setup prompt rather than failing silently.

### Development on GNOME Wayland (for contributors)

To develop and test the X11 backend path without switching sessions:

```bash
GDK_BACKEND=x11 cargo tauri dev
```

This forces the app into XWayland mode. The arboard backend works normally. No session switch required. See CONTRIBUTING.md for full dev environment setup.

---

## Milestone Summary

| Version | Name | Key Deliverable | Demo-ready |
|---|---|---|---|
| v0.1 | Foundation | Clip capture and panel UI on X11 | No |
| v0.2 | Search | DSA search layer, Huffman compression, keyboard nav | No |
| v0.3 | Wayland + Polish | Full platform support, persistent undo, Union-Find grouping, UI complete | Yes — professor demo |
| v0.4 | DSA Expansion + System | Skip List ranking, auto-categorization, global hotkey, auto-start, error logging, onboarding | Yes — professor demo |
| v0.5 | Synapt Bridge | Cross-device clipboard via Synapt | No |
| v1.0 | Release | Installable, documented, stable | Yes — public release |

---

## v0.1 — Foundation

Target: The app builds, runs on Linux X11, captures clipboard entries, and displays them in a basic panel. This is the skeleton everything else is built on.

### Backend
- Tauri v2 project scaffolded with Rust backend and React + TypeScript frontend
- SQLite database initialized on first launch via sqlx migrations
- ClipboardWatcher implemented using arboard, polling every 500ms in a background Tokio task
- New clips sent through a tokio::sync::mpsc channel to the storage layer
- Duplicate detection: consecutive identical entries are not stored
- Clips stored in SQLite with id, content, content_type, created_at, source_app fields
- History limit enforced: oldest non-pinned entries deleted when limit exceeded (default 500)
- Platform detection module in place (returns X11 for all cases in this version, expanded in v0.3)

### Frontend
- System tray icon with left-click to show/hide panel
- Panel renders as a floating window anchored near tray
- Scrollable list of clip cards, most recent at top
- Each clip card shows: truncated content preview, timestamp, click to copy to clipboard
- Basic loading state while clips are fetched from backend on panel open
- Tailwind CSS base styles applied, no theme switching yet

### Out of scope
- Search of any kind
- Pinning or categories
- Wayland support
- Windows support
- Settings page
- Synapt integration

---

## v0.2 — Search

Target: Fast, DSA-powered search over clipboard history. This is the academically significant version — the search layer is the core contribution of the project.

### Search architecture

The search layer lives entirely in the Rust backend. It does not use synapt-core. SynaptClip owns its own search implementation.

| Structure | Implementation | Purpose |
|---|---|---|
| Trie | Custom Rust, compressed trie (radix/Patricia), built in-memory from SQLite on launch | Prefix search on clip text |
| Suffix Array | Custom Rust, per-clip index, binary search for substring match | Substring search; second tier after prefix |
| Levenshtein distance | Custom Rust | Fuzzy matching for typos and partial terms; fallback when prefix and substring return empty |
| Ranking | Recency + pinned boost | Pinned clips ranked above unpinned, recency as tiebreaker |

The Trie is rebuilt from the database on each app launch and updated incrementally as new clips arrive. It is not persisted to disk.

### Backend
- Trie struct with insert, prefix_search, and remove operations
- Levenshtein distance function with configurable threshold (default: 2)
- Search engine that runs prefix search first, falls back to fuzzy if no prefix results
- search_clips Tauri command: takes query string, returns ranked list of clip IDs and scores
- Trie updated on every new clip captured, and on delete
- Unit tests for Trie, Levenshtein, and ranking logic

### Clip compression
- Clip content longer than 512 bytes is compressed using Huffman encoding before storing in SQLite
- Huffman tree is built from character frequencies of the clip content at insert time
- Frequency table stored alongside compressed bytes to enable decompression on read
- Clips too short to benefit from compression are stored raw with a flag
- Panel clip card shows original size vs stored size as a muted stat line
- DSA: Huffman Tree (Unit 1)

### Frontend
- Search bar at top of panel, always visible
- Real-time search as user types, debounced at 150ms
- Results list replaces full clip list while query is active
- Keyboard navigation: arrow keys move selection, Enter copies selected clip, Escape clears search
- Match highlighting: matched substring visually highlighted in clip preview
- Empty state shown when search returns no results

---

## v0.3 — Wayland Support, Organization, and UI Polish

Target: Full Linux platform support (both X11 and Wayland), clip organization features, and a polished UI. This is the professor demo milestone. The app must be visually presentable and functionally complete enough to demonstrate the core value proposition.

This version is the midpoint checkpoint. After v0.3, the app is feature-complete on Linux and ready to show to stakeholders.

### Wayland backend (single binary)

- Platform detection module expanded to identify session type and compositor at runtime
- WlrDataControl backend: spawns `wl-paste --watch` as a subprocess, reads stdout line by line, sends new clips through the same mpsc channel as arboard
- GchFile backend: locates GCH storage file at `~/.local/share/gnome-shell/extensions/clipboard-history@alexsaveau.dev/`, watches via inotify using the notify crate, parses GCH binary log format on change
- Runtime selection: one backend is instantiated at startup, the rest are never used
- Graceful degradation on GNOME Wayland if GCH is not installed: setup prompt shown in panel with install instructions, no crash

All three backends implement the same ClipboardWatcher trait:

```rust
trait ClipboardWatcher: Send {
    async fn watch(&self, tx: mpsc::Sender<NewClip>) -> Result<()>;
}
```

### Clip organization
- Pinning: clips can be pinned, pinned clips appear in a dedicated section at top of panel and are never auto-deleted
- Categories: user-defined text tags on clips, stored in SQLite
- Category filter tabs in panel UI: All, Pinned, and one tab per user-created category
- Right-click context menu on clip card: Copy, Pin/Unpin, Assign Category, Delete
- Bulk select mode: hold Shift and click to select multiple clips, then delete or categorize
- Clear history action in panel header: deletes all non-pinned clips with confirmation prompt

### Clip undo via persistent data structures
- Deleting a clip creates a new version of the clip list rather than mutating it, implemented as a persistent linked list
- An "Undo" action in the panel header restores the last deleted clip by reverting to the previous version
- Version chain is kept in memory for the current session only, not persisted to SQLite
- DSA: Persistent Data Structures (Unit 6)

### Smart clip grouping via Union-Find
- Automatic clip grouping using Disjoint Set Union-Find with path compression and weighted union
- Clips are unioned when they share the same source app or match a prefix pattern
- User can view clips by group in a separate Groups tab in the panel
- Union and find operations run at O(alpha(n)) amortized
- DSA: Disjoint Set Union-Find (Unit 6)

### Settings page
- History limit: number of clips to retain (50 / 100 / 500 / 1000 / unlimited)
- Default expiry: auto-delete non-pinned clips older than N days (off by default)
- Excluded apps: list of application names whose clipboard activity is ignored
- Global hotkey: configurable keyboard shortcut to show/hide panel (default: Super + Shift + V)
- Theme: light, dark, system
- Wayland setup status: shows which backend is active, shows GCH install prompt if needed

### UI polish for demo
- Consistent visual design across panel, settings, and context menus
- Dark and light theme both complete and visually coherent
- App icon finalized
- Smooth open/close animation on panel
- Tray icon reflects app state (active, paused, Wayland warning)
- Timestamps shown as relative ("2 minutes ago") not absolute

---

## v0.4 — DSA Expansion and System Features

Target: Two new DSA contributions (Skip List, Classification Trie), system-level features (global hotkey, auto-start, error logging), and a first-run onboarding experience.

### New data structures

| Structure | Unit | Role |
|---|---|---|
| Skip List | Unit 4 — Randomized Data Structures | In-memory sorted clip index; probabilistic O(log n) insert, remove, and top-N lookup; score = recency decay + pinned boost |
| Classification Trie (reuse of search Trie) | Unit 3 — Data Structures for Strings | Content-type classifier; same Trie struct as search, separate instance loaded with URL and file path prefixes |

### Skip List ranking
- Index-based arena implementation (no unsafe Rust): nodes stored in `Vec<Option<SkipNode>>` with free-list recycling
- Score function: `1.0 / (1.0 + seconds_since_created) + if pinned { 0.5 } else { 0.0 }`
- `get_ranked_clips` Tauri command returns a ranked list of clip IDs; Panel "all" tab merges this with full clip records
- Skip List updated on every insert, delete, and pin toggle; no periodic rebuild required
- 10 unit tests covering insert order, removal, top-N, score update, and duplicate handling

### Auto-categorization via Classification Trie
- ClipClassifier struct wraps a Trie pre-loaded with URL and file path prefixes
- Detection order: empty → PlainText, trie hit + string check → Link / FilePath, `@` → Email, `#hex` → Color, keyword set → Code, PlainText
- `trie_prefix_hit` tries lengths 1–9 of the lowercased content to find stored prefixes (resolves the direction mismatch between search Trie and classifier use case)
- Auto-categories (Link, File Path, Code, Email, Color) appear as tabs in the panel alongside user-defined categories; visually distinguished with a muted "auto" badge
- `get_auto_categories` Tauri command returns the fixed set of auto-category names
- 14 unit tests covering all detection branches

### Global hotkey
- `tauri-plugin-global-shortcut` registered on launch; default `Super+Shift+V`
- Configurable in Settings; saved to persistent settings store; takes effect immediately without restart
- On trigger: centers and focuses the main window
- Wayland note shown in Settings when GCH or Wayland session is detected

### Auto-start (Linux)
- Writes a systemd user service file to `~/.config/systemd/user/synaptclip.service` on first run
- `ExecStart` set to the current executable path at install time
- `is_service_installed` check prevents duplicate installs
- `get_autostart_status` and `install_autostart` Tauri commands expose status and manual install to Settings UI

### Error logging and crash reporting
- `tracing-subscriber` writes structured logs to `~/.local/share/synaptclip/synaptclip.log` and stderr simultaneously via a custom `DualWriter` struct
- Log rotation: file over 5 MB renamed to `synaptclip.log.old` on startup
- Panic hook captures panic message + backtrace and appends a crash report block to the log file
- Log file path exposed in Settings diagnostics section via `get_log_path` command

### First-run onboarding
- On first launch, an onboarding overlay is shown before the main panel
- Covers three key features: clipboard capture, search and organization, hotkey access
- Clicking "Get started" sets `first_run = "done"` in settings and dismisses the overlay
- Onboarding is skipped on subsequent launches and never shown on the settings window

---

## v0.5 — Synapt Bridge

Target: When Synapt is installed and running on the same machine, SynaptClip gains the ability to send and receive clipboard content across devices on LAN.

### Integration model

SynaptClip does not depend on Synapt as a build-time dependency. The integration is entirely runtime. On startup, SynaptClip polls Synapt's local HTTP API to check if it is running. If found, integration features are enabled. If not, the app works normally with no indication of failure.

Synapt's API runs on `http://127.0.0.1:57321`. SynaptClip's incoming listener runs on `http://127.0.0.1:57322`. Both are loopback-only.

### Backend
- bridge.rs module: polls GET /v1/health on startup and every 10 seconds
- On Synapt detected: fetch peer list from GET /v1/peers, cache for session
- send_to_peer Tauri command: calls POST /v1/clips/send with peer ID and clip content
- Incoming listener: Axum server on 57322 receives POST /v1/clips/receive from Synapt, stores clip in SQLite with source_app = "synapt" and sender name noted
- Incoming listener always starts on app launch regardless of Synapt presence

### Frontend
- Devices section in panel, visible only when Synapt bridge is active
- Shows list of connected peers with online/offline status
- Each clip card context menu gains "Send to device" submenu when bridge is active
- Tray icon tooltip shows "Synapt connected — N devices" when bridge is active
- Received clips visually distinguished in clip list (source device shown)

### API contract

See `references/docs/api-contract.md` for the full endpoint specification, error format, mock server, and integration checklist.

---

## v1.0 — Release

Target: The first publicly installable, documented, stable release. The app works on Linux (X11 and Wayland) and Windows. Synapt integration is functional. Installation requires no manual steps beyond downloading a binary or package.

### Packaging
- Linux: `.rpm` package for Fedora/RHEL built via Tauri bundler
- Linux: `.deb` package for Debian/Ubuntu
- Linux: AppImage for distro-agnostic install
- Windows: `.msi` installer via NSIS bundler
- All packages built in CI on tag push via GitHub Actions

### Documentation
- README: what SynaptClip is, install instructions per platform, Wayland setup guide for GNOME users, Synapt integration setup
- CONTRIBUTING.md: dev environment setup, how to run on X11/Wayland, how to run tests, PR guidelines
- Changelog maintained from v0.1 onward

### Quality gates for v1.0
- All unit tests passing on Linux and Windows CI
- No known crashes on clean installs of Fedora 40+ and Windows 11
- GCH integration tested and documented
- Synapt bridge tested against Synapt v0.5 or later
- App starts in under 1 second on both platforms
- Memory usage under 80MB at rest with 500 clips in history

### What v1.0 is not
- It is not a complete feature set forever. Post-v1.0 candidates include image clip support, KDE Wayland native support, and a GNOME Shell extension to replace the GCH dependency.
- macOS is not a v1.0 target.

---

## Dependency Reference

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon", "notification"] }
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio"] }
arboard = "3"
notify = "6"              # inotify wrapper for GCH file watching
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1"
tracing = "1"
tracing-subscriber = "0.3"
axum = "0.7"              # incoming Synapt bridge listener
reqwest = { version = "0.11", features = ["json"] }
```
