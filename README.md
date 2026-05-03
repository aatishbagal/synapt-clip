# SynaptClip

A clipboard manager for Linux and Windows, built with Rust and Tauri v2. Captures clipboard history, provides search, and optionally integrates with Synapt for cross-device clipboard sharing over LAN.

Currently in active development (v0.4.0).

## Prerequisites

- Rust (stable toolchain): https://rustup.rs
- Node.js 20 or later
- Tauri CLI v2: `cargo install tauri-cli`

### Linux (Fedora)

```bash
sudo dnf install webkit2gtk4.1-devel libsoup3-devel libayatana-appindicator-gtk3-devel openssl-devel gcc pkg-config
```

### Linux (Ubuntu/Debian)

```bash
sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev libssl-dev gcc pkg-config
```

## Setup

```bash
npm install
cd src-tauri
cargo sqlx database create --database-url "sqlite:./synaptclip.db"
cargo sqlx migrate run --database-url "sqlite:./synaptclip.db"
cd ..
```

## Running in Development

Standard launch:

```bash
cargo tauri dev
```

Force X11 mode on Wayland (recommended for GNOME Wayland):

```bash
GDK_BACKEND=x11 cargo tauri dev
```

Or use the dev script:

```bash
./scripts/dev-x11.sh
```

## Build Notes

If you hit compiler errors related to `clang` and `ccache`, set the C compiler explicitly:

```bash
CC=/usr/bin/clang cargo tauri dev
```

## Architecture

See `references/docs/` for the source of truth on architecture and development guidelines:

- `ROADMAP.md` -- version-by-version feature plan
- `CONTRIBUTING.md` -- dev environment setup, code style, platform detection patterns
- `synaptclip-blueprint.md` -- data model and architecture reference
- `api-contract.md` -- Synapt integration API specification

## Features

### Clipboard Capture
- Background clipboard monitoring on X11, Wayland wlroots, and Wayland GNOME (via GCH extension)
- Runtime backend detection -- single binary, no user configuration required
- Duplicate detection, configurable history limit, source app tracking

### Search
- Prefix search using a compressed Trie (Patricia Trie)
- Substring search using Suffix Array
- Fuzzy search using Levenshtein distance for typo tolerance
- Real-time results with match highlighting and ranked output
- Ranked results ordered by a Skip List sorted index -- pinned and recent clips surface first

### Organization
- Pin clips permanently, user-defined categories, bulk operations
- Smart automatic grouping via Union-Find (same source app or prefix pattern)
- Clip undo -- delete is non-destructive, reverts via persistent data structure
- Auto-categorization -- new clips are automatically classified as Link, File Path, Code, Email, or Color using a classification Trie and keyword set

### System
- Global hotkey to show the panel from anywhere (configurable in Settings)
- Auto-start on login via systemd user service (Linux), installed on first run
- File-based error logging with log rotation; crash reporter writes panic context to disk

### Storage
- SQLite with Huffman-compressed content for clips over 512 bytes
- Auto-expiry scheduler using a Double Ended Priority Queue

### Synapt Integration (optional)
- Send and receive clipboard content across LAN devices when Synapt is installed
- No dependency on Synapt -- integration activates at runtime if detected

## Data Structures and Algorithms

Every data structure below is implemented in the Rust backend, wired into the active code path, and covered by unit tests.

| Concept | Syllabus Unit | Used For |
|---|---|---|
| Compressed Trie (Patricia Trie) | Unit 3 — Data Structures for Strings | Prefix search on clip content; reused for auto-category classification in v0.4 |
| Suffix Array | Unit 3 — Data Structures for Strings | Substring search across all clip content; second search tier after prefix |
| Levenshtein Distance | Unit 3 — Dictionaries Allowing Errors | Fuzzy search fallback when prefix and substring search return no results |
| Huffman Tree | Unit 1 — Advanced Trees | Clip content compression before SQLite storage for clips over 512 bytes |
| Bloom Filter | Unit 6 — Succinct Representations | Duplicate detection before SQLite insert; bit vector with FNV1a and DJB2 hash functions |
| Persistent Linked List | Unit 6 — Persistent Data Structures | Non-destructive clip delete; undo restores previous version without mutating history |
| Disjoint Set Union-Find | Unit 6 — Miscellaneous | Automatic clip grouping by source app and content prefix; path compression and union by rank |
| Double Ended Priority Queue | Unit 6 — Miscellaneous | History limit enforcement; oldest clips evicted by minimum timestamp when limit exceeded |
| Concurrent Channels (mpsc) | Unit 6 — Concurrent Data Structures | Thread-safe message passing between clipboard watcher task and storage layer |
| Skip List | Unit 4 — Randomized Data Structures | Sorted in-memory clip index for search result ranking; probabilistic O(log n) insert and lookup |
| Classification Trie (reuse of search Trie) | Unit 3 — Data Structures for Strings | Content-type detection for auto-categorization; same Trie struct as search, separate instance loaded with URL and file path prefixes |

The Trie serves two roles in SynaptClip: prefix search over clipboard history, and content-type classification for auto-categorization. The Skip List maintains a globally sorted clip index by recency score, combining time decay with a pinned-clip boost, so that the most relevant clips surface first in the panel without a full sort on every query.
