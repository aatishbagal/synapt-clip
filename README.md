# SynaptClip

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./assets/images/logo/png/SynaptV2_White_PNG_512sq.png">
    <img src="./assets/images/logo/png/SynaptV2_Black_PNG_512sq.png" alt="SynaptClip" width="120">
  </picture>
</p>

A clipboard manager for Linux and Windows, built with Rust and Tauri v2. Captures clipboard history, provides search, and optionally integrates with [Synapt](https://github.com/aatishbagal/synapt) for cross-device clipboard sharing over LAN.

Currently in active development (v0.4.1).

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
- Prefix, substring, and fuzzy (typo-tolerant) search
- Real-time results with match highlighting and ranked output
- Pinned and recent clips surface first

### Organization
- Pin clips permanently, user-defined categories, bulk operations
- Smart automatic grouping by source app or content prefix
- Non-destructive delete with undo
- Auto-categorization -- new clips are classified as Link, File Path, Code, Email, or Color

### System
- Global hotkey to show the panel from anywhere (configurable in Settings)
- Auto-start on login via systemd user service (Linux), installed on first run
- File-based error logging with log rotation; crash reporter writes panic context to disk

### Storage
- SQLite with compressed content for clips over 512 bytes
- Configurable auto-expiry

### Synapt Integration (optional)
- Send and receive clipboard content across LAN devices when [Synapt](https://github.com/aatishbagal/synapt) is installed
- No dependency on Synapt -- integration activates at runtime if detected

## Related

- [Synapt](https://github.com/aatishbagal/synapt) -- Spotlight-style launcher and LAN file utility
- [synapt-core](https://github.com/aatishbagal/synapt-core) -- shared type library used across the Synapt apps
