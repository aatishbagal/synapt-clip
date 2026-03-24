# SynaptClip

A clipboard manager for Linux and Windows, built with Rust and Tauri v2. Captures clipboard history, provides search, and optionally integrates with Synapt for cross-device clipboard sharing over LAN.

Currently in early development (v0.2.0).

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

### Organization
- Pin clips permanently, user-defined categories, bulk operations
- Smart automatic grouping via Union-Find (same source app or prefix pattern)
- Clip undo -- delete is non-destructive, reverts via persistent data structure

### Storage
- SQLite with Huffman-compressed content for clips over 512 bytes
- Auto-expiry scheduler using a Double Ended Priority Queue

### Synapt Integration (optional)
- Send and receive clipboard content across LAN devices when Synapt is installed
- No dependency on Synapt -- integration activates at runtime if detected

## Data Structures and Algorithms

### Implemented

| Concept | Syllabus Unit | Used For |
|---|---|---|
| Compressed Trie (Patricia Trie) | Unit 3 -- DS for Strings | Prefix search on clip content |
| Suffix Array | Unit 3 -- DS for Strings | Substring search across all clip content |
| Levenshtein Distance | Unit 3 -- Dictionaries Allowing Errors | Fuzzy search fallback |
| Huffman Tree | Unit 1 -- Advanced Trees | Clip content compression before SQLite storage |
| Double Ended Priority Queue | Unit 2 -- Priority Queues | Auto-expiry scheduler, oldest and newest clip access |
| Bloom Filter (Bit Vector) | Unit 6 -- Succinct Representations | Fast duplicate detection before insert |
| Persistent Linked List | Unit 6 -- Persistent Data Structures | Non-destructive clip delete with undo support |
| Disjoint Set Union-Find | Unit 6 -- Miscellaneous | Automatic clip grouping by source or prefix pattern |
| Concurrent Channels (mpsc) | Unit 6 -- Concurrent Data Structures | Thread-safe communication between watcher and storage tasks |

### Planned (not yet implemented)

| Concept | Syllabus Unit | Planned Use |
|---|---|---|
| Skip List | Unit 4 -- Randomized DS | Alternative sorted clip list with probabilistic balancing |
| Treap | Unit 4 -- Randomized DS | Randomized BST for clip ranking by score and recency |
| Splay Tree | Unit 1 -- Advanced Trees | Self-adjusting tree that moves recently accessed clips to root |
| AVL or Red-Black Tree | Unit 1 -- Advanced Trees | Guaranteed O(log n) sorted retrieval by timestamp or score |
| DAWG | Unit 3 -- DS for Strings | Compact index of all substrings for memory-efficient search |
