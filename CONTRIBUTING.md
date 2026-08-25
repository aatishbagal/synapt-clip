# Contributing to synapt-clip

## Development setup

### Prerequisites

- Rust stable (install via rustup: https://rustup.rs)
- Node.js 18 or later
- Tauri CLI: `cargo install tauri-cli --version '^2'`
- Linux: `libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`
- macOS: Xcode Command Line Tools (`xcode-select --install`)

### Setup

`synapt-clip` depends on the `synapt-core` crate through a relative path
(`../../synapt-core`), so the two repositories must sit side by side. There is
no `install.sh` in this repository, so clone `synapt-core` yourself:

```bash
git clone https://github.com/aatishbagal/synapt-clip.git
git clone https://github.com/aatishbagal/synapt-core.git   # sibling of synapt-clip
cd synapt-clip
npm install
```

Your directory layout should look like this:

```
parent/
  synapt-clip/
  synapt-core/
```

### Running in development

```bash
RUST_LOG=info cargo tauri dev
```

### Running tests

```bash
cargo test
cargo clippy -- -D warnings
npm run build
```

All three must pass before submitting a pull request.

## Pull request guidelines

- Open an issue before starting work on a non-trivial change
- One logical change per pull request
- All tests must pass: `cargo test`, `cargo clippy -- -D warnings`, `npm run build`
- No emojis in code, comments, strings, or documentation
- Commit messages follow the format: `type(scope): description`
  Valid types: feat, fix, chore, docs, refactor, test
- Prefer small modules. If a new module grows past roughly 300 lines, consider
  splitting it. Some existing modules are considerably larger; treat those as
  debt to be paid down, not as a pattern to copy.
- No `unwrap()` or `expect()` in non-test code

## Commit message format

```
feat(network): add peer discovery retry on startup
fix(macos): clear WKWebView background for transparent corners
chore(release): v0.5.1
docs(readme): add Linux Wayland setup instructions
```

## Architecture overview

See the README for the high-level architecture. Key modules:

- `src-tauri/src/clipboard/` - clipboard watcher and capture backends (arboard, wlr, GNOME Clipboard History)
- `src-tauri/src/synapt/` - Synapt bridge (listener on loopback port 57322, bridge polling against port 57321)
- `src-tauri/src/search/` - Trie, fuzzy matching, Bloom filter, Huffman, suffix array, classifier
- `src-tauri/src/dsa/` - persistent list, skip list, and union-find used for history and grouping
- `src-tauri/src/storage/` - SQLite database and migrations
- `src-tauri/src/platform/` - platform-specific setup and autostart
- `src/` - React and TypeScript frontend
