# Contributing to SynaptClip

This document covers everything needed to get the development environment running, understand the codebase structure, and contribute effectively. Place this file at `references/docs/CONTRIBUTING.md` in the repository.

---

## Repository Structure

```
synapt-clip/
├── references/
│   └── docs/
│       ├── CONTRIBUTING.md         # this file
│       ├── ROADMAP.md              # version-by-version feature plan
│       ├── api-contract.md         # Synapt integration API specification
│       └── synaptclip-blueprint.md # architecture and data model reference
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs
│       ├── commands.rs
│       ├── clipboard/
│       │   ├── mod.rs
│       │   ├── watcher.rs          # ClipboardWatcher trait and backend selection
│       │   ├── arboard.rs          # X11 and Windows backend
│       │   ├── wlr.rs              # wlroots Wayland backend
│       │   └── gch.rs              # GNOME Wayland / GCH file watcher backend
│       ├── search/
│       │   ├── trie.rs
│       │   ├── fuzzy.rs
│       │   └── engine.rs
│       ├── storage/
│       │   └── db.rs
│       ├── platform/
│       │   └── detect.rs           # runtime session and compositor detection
│       └── synapt/
│           └── bridge.rs
└── src/
    ├── main.tsx
    ├── components/
    └── pages/
```

---

## Prerequisites

### All platforms

- Rust (stable toolchain via rustup): https://rustup.rs
- Node.js 20 or later
- Tauri CLI v2: `cargo install tauri-cli`

### Linux

```bash
# Fedora
sudo dnf install webkit2gtk4.1-devel libayatana-appindicator-gtk3-devel \
  openssl-devel gcc pkg-config

# Ubuntu / Debian
sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev \
  libssl-dev gcc pkg-config
```

### Windows

- Visual Studio Build Tools with the C++ workload, or Visual Studio Community
- WebView2 (pre-installed on Windows 11, downloadable for Windows 10)

---

## Running the App

### Standard launch (detects session automatically)

```bash
cargo tauri dev
```

The app detects your session type at runtime and selects the appropriate clipboard backend.

### Force X11 mode on a Wayland session

If you are running GNOME Wayland and want to develop against the X11/arboard backend without switching sessions:

```bash
GDK_BACKEND=x11 cargo tauri dev
```

This is the recommended approach for all developers on GNOME Wayland. It runs the app through XWayland, which is available by default on Fedora. The arboard clipboard backend works normally in this mode.

You can also create a dev launch script in the repo root:

```bash
#!/bin/bash
# scripts/dev-x11.sh
GDK_BACKEND=x11 cargo tauri dev
```

### Testing the Wayland GCH backend

To test the GNOME Wayland path:

1. Ensure the Clipboard History extension is installed: https://extensions.gnome.org/extension/4839/clipboard-history/
2. Enable it: `gnome-extensions enable clipboard-history@alexsaveau.dev`
3. Launch the app without `GDK_BACKEND=x11`:

```bash
cargo tauri dev
```

The app detects GNOME Wayland, locates the GCH storage file, and starts the inotify watcher. Copy something to your clipboard and confirm it appears in the SynaptClip panel.

### Testing the wlroots Wayland backend

This requires a wlroots-based compositor (Sway, Hyprland, River). On such a session:

```bash
cargo tauri dev
```

The app detects wlroots support and spawns `wl-paste --watch`. Ensure `wl-clipboard` is installed:

```bash
# Fedora
sudo dnf install wl-clipboard

# Ubuntu
sudo apt install wl-clipboard
```

---

## Platform Detection

The detection logic lives in `src-tauri/src/platform/detect.rs`. It is the single place where session type and compositor are identified. All clipboard backend selection is driven by this module.

```rust
pub enum ClipboardBackend {
    Arboard,        // X11, XWayland, Windows
    WlrDataControl, // Wayland with wlroots (Sway, Hyprland)
    GchFile,        // Wayland with GNOME/Mutter — requires GCH extension
}

pub fn detect_backend() -> ClipboardBackend {
    // Check XDG_SESSION_TYPE, then WAYLAND_DISPLAY,
    // then probe for wlroots protocol availability,
    // then fall back to GchFile for GNOME Wayland
}
```

Do not add platform-specific branches anywhere else in the codebase. If you need platform-specific behavior, add it here and expose it through this module.

---

## Clipboard Backends

Each backend implements the `ClipboardWatcher` trait:

```rust
pub trait ClipboardWatcher: Send {
    async fn watch(&self, tx: mpsc::Sender<NewClip>) -> Result<()>;
}
```

The trait is the only interface the rest of the app uses. Storage, search, and the frontend are completely unaware of which backend is active.

### arboard (X11 / Windows)

- Polls the system clipboard every 500ms
- Compares against last captured value, discards duplicates
- Source: `src-tauri/src/clipboard/arboard.rs`

### wlr-data-control (wlroots Wayland)

- Spawns `wl-paste --watch` as a child process
- Reads stdout line by line
- Restarts the subprocess with backoff if it exits unexpectedly
- Source: `src-tauri/src/clipboard/wlr.rs`

### GCH file watcher (GNOME Wayland)

- Locates GCH storage file at `~/.local/share/gnome-shell/extensions/clipboard-history@alexsaveau.dev/`
- Watches the file via inotify using the `notify` crate
- Parses GCH binary log format on each write event to extract new entries
- If the file does not exist, the backend emits a `GchNotInstalled` error which the UI surfaces as a setup prompt
- Source: `src-tauri/src/clipboard/gch.rs`

### GCH binary format

GCH stores entries in an append-only binary log. Each operation begins with a one-byte type field:

```
0x01 = Add
0x02 = Delete
0x03 = Move
```

An Add operation is followed by UTF-8 encoded text terminated by a NUL byte (`0x00`). When parsing, read from the last known file offset, process each new operation sequentially, and emit NewClip events only for Add operations.

---

## Search Layer

The search layer is in `src-tauri/src/search/`. It does not depend on any external crate beyond the standard library. All structures are custom Rust implementations.

### Trie

- Supports `insert(text: &str, clip_id: i64)`, `prefix_search(prefix: &str) -> Vec<i64>`, and `remove(clip_id: i64)`
- Built in memory from SQLite on app start
- Updated incrementally on every new clip and every delete
- Not persisted to disk

### Fuzzy (Levenshtein)

- Computes edit distance between query and clip content
- Used as fallback when prefix search returns zero results
- Default threshold: distance 2

### Engine

- `search(query: &str) -> Vec<RankedClip>` runs prefix search first
- If prefix results exist, returns them ranked by recency and pinned status
- If no prefix results, runs fuzzy search across all clips
- Pinned clips receive a rank boost regardless of search method

---

## Synapt Integration

The Synapt bridge is in `src-tauri/src/synapt/bridge.rs`. It is inactive unless Synapt is running on the same machine.

On startup, the bridge polls `http://127.0.0.1:57321/health`. If Synapt responds, the bridge enables integration features. SynaptClip also starts an Axum listener on `http://127.0.0.1:57322` to receive incoming clips from Synapt.

For full endpoint specifications, request/response shapes, error formats, and a mock server for development without Synapt installed, see `references/docs/api-contract.md`.

### Developing the bridge without Synapt

Run the mock server:

```bash
python references/docs/mock-synapt.py
```

This starts a minimal HTTP server on port 57321 that responds to all Synapt API endpoints with realistic mock data. SynaptClip's bridge will detect it and enable integration features as if Synapt were running.

---

## Testing

Run all tests:

```bash
cargo test
```

Run tests for a specific module:

```bash
cargo test search::trie
cargo test clipboard::gch
```

Each module in `src-tauri/src/` should have a `#[cfg(test)]` block with unit tests covering normal behavior and edge cases. When implementing a new module, write tests in the same file alongside the implementation.

Required test coverage per module:

| Module | Required tests |
|---|---|
| search/trie.rs | insert, prefix_search, remove, empty input, unicode |
| search/fuzzy.rs | exact match, distance 1, distance 2, over threshold, empty string |
| search/engine.rs | prefix hit, prefix miss falls back to fuzzy, pinned boost |
| clipboard/gch.rs | add op parsing, delete op parsing, malformed input, missing file |
| platform/detect.rs | X11, Wayland wlroots, Wayland GNOME (mocked env vars) |

---

## Code Style

- Run `cargo fmt` before committing. The project uses default rustfmt settings.
- Run `cargo clippy` and resolve all warnings before opening a PR.
- No `unwrap()` or `expect()` in production code paths. Use `?` and proper error types.
- All public functions and structs must have doc comments.
- Keep modules small and focused. If a file exceeds 200 lines, consider splitting it.

---

## Git Workflow

- `main` branch is always buildable and passing tests
- Feature branches named `feat/description`, bug fixes named `fix/description`
- One PR per logical unit of work
- PRs require at least one review before merge
- Commit messages in present tense: "Add Trie prefix_search" not "Added Trie prefix_search"

---

## Working with Claude Code

Claude Code is used for implementing modules. Keep sessions scoped to one module or one task at a time. Before starting a session, point Claude Code at the relevant source file and the corresponding section of ROADMAP.md.

The `references/docs/` directory is the source of truth for architecture decisions. If Claude Code produces something that conflicts with the blueprint or roadmap, correct it rather than updating the docs to match.

Example session start:

```
Implement the Trie struct in src-tauri/src/search/trie.rs.
Requirements are in references/docs/ROADMAP.md under v0.2 — Search.
The struct should support insert(text: &str, clip_id: i64),
prefix_search(prefix: &str) -> Vec<i64>, and remove(clip_id: i64).
Include unit tests in the same file.
```

---

## Common Issues

### App does not capture clipboard on GNOME Wayland

The GCH extension is required. Install it from https://extensions.gnome.org/extension/4839/clipboard-history/ and enable it. Then restart the app.

If you want to bypass this during development, use `GDK_BACKEND=x11 cargo tauri dev` instead.

### wl-paste not found

Install the `wl-clipboard` package:

```bash
sudo dnf install wl-clipboard   # Fedora
sudo apt install wl-clipboard   # Ubuntu
```

### Tray icon not visible on GNOME Wayland

GNOME does not show system tray icons by default. Install the AppIndicator extension:

```bash
sudo dnf install gnome-shell-extension-appindicator  # Fedora
```

Then enable it in GNOME Extensions or via:

```bash
gnome-extensions enable appindicatorsupport@rgcjonas.gmail.com
```

### Build fails on Linux: missing webkit2gtk

Install the system dependencies listed under Prerequisites above for your distribution.

---

## Contact

For questions about the Synapt API contract or the synapt-core crate, open an issue on the Synapt repository or reach the Synapt maintainer directly. The API contract in `references/docs/api-contract.md` is frozen for v0.5 and changes to it require coordination between both repos.
