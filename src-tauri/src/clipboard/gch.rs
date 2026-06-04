use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::Duration;

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

use super::watcher::{ClipboardWatcher, NewClip, WatcherError};

const EXTENSION_ID: &str = "clipboard-history@alexsaveau.dev";
const OP_ADD: u8 = 0x01;
const OP_DELETE: u8 = 0x02;
const OP_MOVE: u8 = 0x03;

/// How long to wait for the GCH database to appear before giving up, when the
/// extension is enabled but has not written any data yet.
const DB_WAIT_TIMEOUT: Duration = Duration::from_secs(30);
/// Interval between database existence polls while waiting.
const DB_POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Installed/enabled status of the GNOME Clipboard History extension.
#[derive(Debug, Clone, serde::Serialize)]
pub struct GchStatus {
    /// Whether the extension directory exists in any known location.
    pub installed: bool,
    /// Whether the extension id appears in `org.gnome.shell enabled-extensions`.
    pub enabled: bool,
    /// Path to the extension's append-only database, if found.
    pub db_path: Option<PathBuf>,
    /// Path to the extension installation directory, if found.
    pub extension_path: Option<PathBuf>,
}

/// Candidate directories where the extension may be installed.
fn extension_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(local) = dirs::data_local_dir() {
        dirs.push(local.join("gnome-shell/extensions").join(EXTENSION_ID));
    }
    dirs.push(PathBuf::from("/usr/share/gnome-shell/extensions").join(EXTENSION_ID));
    dirs.push(PathBuf::from("/usr/local/share/gnome-shell/extensions").join(EXTENSION_ID));
    dirs
}

/// Candidate database file locations, in priority order.
///
/// Current versions of the extension write `database.log` under the user cache
/// directory (`GLib.get_user_cache_dir()`), so that is checked first. Older
/// versions wrote inside the extension directory or under the data directory,
/// which are kept as fallbacks.
fn db_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    // Current location: $XDG_CACHE_HOME/<ext>/database.log
    if let Some(cache) = dirs::cache_dir() {
        candidates.push(cache.join(EXTENSION_ID).join("database.log"));
    }

    // Older / alternate locations under the data directory.
    if let Some(local) = dirs::data_local_dir() {
        candidates.push(local.join(EXTENSION_ID).join("database.log"));
        candidates.push(
            local
                .join("gnome-shell/extensions")
                .join(EXTENSION_ID)
                .join("database.log"),
        );
    }

    // Explicit XDG_DATA_HOME-aware fallback.
    let data_home = std::env::var("XDG_DATA_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".local/share")));
    if let Some(data_home) = data_home {
        candidates.push(data_home.join(EXTENSION_ID).join("database.log"));
    }

    candidates
}

/// Return the first path in `paths` that exists on disk.
fn first_existing(paths: Vec<PathBuf>) -> Option<PathBuf> {
    paths.into_iter().find(|p| p.exists())
}

/// Parse `gsettings get org.gnome.shell enabled-extensions` output for `ext_id`.
fn enabled_from_gsettings(output: &str, ext_id: &str) -> bool {
    output.contains(ext_id)
}

/// Query whether the extension id is listed in the shell's enabled extensions.
fn query_enabled(ext_id: &str) -> bool {
    std::process::Command::new("gsettings")
        .args(["get", "org.gnome.shell", "enabled-extensions"])
        .output()
        .map(|o| enabled_from_gsettings(&String::from_utf8_lossy(&o.stdout), ext_id))
        .unwrap_or(false)
}

/// Detect the installation and enabled state of the GNOME Clipboard History
/// extension across all known locations.
pub fn detect_gch() -> GchStatus {
    let extension_path = first_existing(extension_dirs());
    let installed = extension_path.is_some();
    let enabled = query_enabled(EXTENSION_ID);
    let db_path = first_existing(db_candidates());

    GchStatus {
        installed,
        enabled,
        db_path,
        extension_path,
    }
}

/// Tauri command exposing GCH status to the frontend.
#[tauri::command]
pub fn get_gch_status() -> GchStatus {
    detect_gch()
}

/// Clipboard watcher for GNOME via the Clipboard History extension.
///
/// Reads the append-only binary log written by the extension and emits a
/// `NewClip` for every Add operation.
pub struct GchWatcher {
    extension_id: String,
}

impl GchWatcher {
    /// Create a watcher with the default extension id.
    pub fn new() -> Self {
        Self {
            extension_id: EXTENSION_ID.to_string(),
        }
    }

    /// Resolve the database path, waiting for it to appear if the extension is
    /// enabled but has not written any data yet.
    async fn resolve_db_path(&self) -> Result<PathBuf, WatcherError> {
        let status = detect_gch();

        if !status.installed {
            return Err(WatcherError::NotFound(
                "Clipboard History GNOME extension not found. \
                 Install it from extensions.gnome.org"
                    .to_string(),
            ));
        }

        if status.installed && !status.enabled {
            return Err(WatcherError::NotFound(
                "Clipboard History extension is installed but not enabled. \
                 Enable it in GNOME Extensions."
                    .to_string(),
            ));
        }

        if let Some(path) = status.db_path {
            return Ok(path);
        }

        // Installed and enabled but no database yet: poll for it to appear
        // rather than failing immediately.
        let start = std::time::Instant::now();
        tracing::info!(
            "GCH enabled but database not yet present; polling for up to {}s",
            DB_WAIT_TIMEOUT.as_secs()
        );
        while start.elapsed() < DB_WAIT_TIMEOUT {
            tokio::time::sleep(DB_POLL_INTERVAL).await;
            if let Some(path) = first_existing(db_candidates()) {
                return Ok(path);
            }
        }

        Err(WatcherError::NotFound(
            "Clipboard History extension is enabled but has not recorded any \
             clipboard data yet. Copy something, then restart SynaptClip."
                .to_string(),
        ))
    }
}

impl Default for GchWatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ClipboardWatcher for GchWatcher {
    async fn watch(&self, tx: mpsc::Sender<NewClip>) -> Result<(), WatcherError> {
        // Touch extension_id so the field is always read (it documents intent).
        debug_assert_eq!(self.extension_id, EXTENSION_ID);

        let path = self.resolve_db_path().await?;

        let parent = path
            .parent()
            .ok_or_else(|| WatcherError::Io("GCH path has no parent".to_string()))?
            .to_path_buf();

        let mut current_offset: u64 = std::fs::metadata(&path)
            .map_err(|e| WatcherError::Io(format!("stat {path:?}: {e}")))?
            .len();

        let (evt_tx, mut evt_rx) = mpsc::unbounded_channel::<notify::Result<notify::Event>>();
        let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |res| {
            let _ = evt_tx.send(res);
        })
        .map_err(|e| WatcherError::Io(format!("notify init: {e}")))?;

        watcher
            .watch(&parent, RecursiveMode::NonRecursive)
            .map_err(|e| WatcherError::Io(format!("notify watch {parent:?}: {e}")))?;

        while let Some(res) = evt_rx.recv().await {
            let event = match res {
                Ok(e) => e,
                Err(e) => {
                    tracing::warn!("notify error: {e}");
                    continue;
                }
            };

            if !event.paths.iter().any(|p| p == &path) {
                continue;
            }

            match event.kind {
                EventKind::Remove(_) => {
                    tracing::warn!("GCH database removed, awaiting recreation");
                    current_offset = 0;
                    continue;
                }
                EventKind::Create(_) => {
                    current_offset = 0;
                }
                EventKind::Modify(_) => {}
                _ => continue,
            }

            match read_and_parse(&path, &mut current_offset, &tx).await {
                Ok(SendOutcome::Continue) => {}
                Ok(SendOutcome::Stopped) => return Ok(()),
                Err(e) => tracing::warn!("GCH parse error: {e}"),
            }
        }

        Ok(())
    }
}

enum SendOutcome {
    Continue,
    Stopped,
}

async fn read_and_parse(
    path: &Path,
    offset: &mut u64,
    tx: &mpsc::Sender<NewClip>,
) -> Result<SendOutcome, WatcherError> {
    let buf = read_tail(path, *offset)?;
    if buf.is_empty() {
        return Ok(SendOutcome::Continue);
    }

    let (entries, consumed) = parse_entries(&buf);
    *offset += consumed as u64;

    for content in entries {
        let clip = NewClip {
            content,
            content_type: "text".to_string(),
            source_app: None,
        };
        if tx.send(clip).await.is_err() {
            return Ok(SendOutcome::Stopped);
        }
    }

    Ok(SendOutcome::Continue)
}

fn read_tail(path: &Path, offset: u64) -> Result<Vec<u8>, WatcherError> {
    let mut file =
        File::open(path).map_err(|e| WatcherError::Io(format!("open {path:?}: {e}")))?;
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| WatcherError::Io(format!("seek: {e}")))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .map_err(|e| WatcherError::Io(format!("read: {e}")))?;
    Ok(buf)
}

fn parse_entries(buf: &[u8]) -> (Vec<String>, usize) {
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut last_complete = 0usize;

    while i < buf.len() {
        let op = buf[i];
        let body_start = i + 1;
        let terminator = buf[body_start..].iter().position(|&b| b == 0x00);
        let Some(term_rel) = terminator else {
            break;
        };
        let term_abs = body_start + term_rel;

        match op {
            OP_ADD => {
                if let Ok(s) = std::str::from_utf8(&buf[body_start..term_abs]) {
                    if !s.is_empty() {
                        out.push(s.to_string());
                    }
                }
            }
            OP_DELETE | OP_MOVE => {}
            _ => {}
        }

        i = term_abs + 1;
        last_complete = i;
    }

    (out, last_complete)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_add_entries() {
        let mut buf = Vec::new();
        buf.push(OP_ADD);
        buf.extend_from_slice(b"hello");
        buf.push(0x00);
        buf.push(OP_ADD);
        buf.extend_from_slice(b"world");
        buf.push(0x00);
        let (entries, consumed) = parse_entries(&buf);
        assert_eq!(entries, vec!["hello".to_string(), "world".to_string()]);
        assert_eq!(consumed, buf.len());
    }

    #[test]
    fn parse_skips_delete_and_move() {
        let mut buf = Vec::new();
        buf.push(OP_ADD);
        buf.extend_from_slice(b"a");
        buf.push(0x00);
        buf.push(OP_DELETE);
        buf.push(0x00);
        buf.push(OP_MOVE);
        buf.push(0x00);
        buf.push(OP_ADD);
        buf.extend_from_slice(b"b");
        buf.push(0x00);
        let (entries, consumed) = parse_entries(&buf);
        assert_eq!(entries, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(consumed, buf.len());
    }

    #[test]
    fn parse_stops_at_incomplete_entry() {
        let mut buf = Vec::new();
        buf.push(OP_ADD);
        buf.extend_from_slice(b"done");
        buf.push(0x00);
        buf.push(OP_ADD);
        buf.extend_from_slice(b"incomplete");
        let (entries, consumed) = parse_entries(&buf);
        assert_eq!(entries, vec!["done".to_string()]);
        assert_eq!(consumed, 6);
    }

    #[test]
    fn enabled_parsing_matches_known_string() {
        let listed = "['background-logo@fedorahosted.org', \
                       'clipboard-history@alexsaveau.dev', \
                       'dash-to-dock@micxgx.gmail.com']";
        assert!(enabled_from_gsettings(listed, EXTENSION_ID));

        let absent = "['background-logo@fedorahosted.org', \
                       'dash-to-dock@micxgx.gmail.com']";
        assert!(!enabled_from_gsettings(absent, EXTENSION_ID));
    }

    #[test]
    fn first_existing_returns_none_when_no_paths_exist() {
        let dir = std::env::temp_dir().join("synaptclip-gch-test-nonexistent");
        let candidates = vec![
            dir.join("a/database.log"),
            dir.join("b/database.log"),
        ];
        assert!(first_existing(candidates).is_none());
    }

    #[test]
    fn first_existing_finds_present_file() {
        let dir = std::env::temp_dir().join("synaptclip-gch-test-present");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("database.log");
        std::fs::write(&file, b"x").expect("write temp db");
        let candidates = vec![dir.join("missing.log"), file.clone()];
        assert_eq!(first_existing(candidates), Some(file.clone()));
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn gch_status_serialises_to_json() {
        let status = GchStatus {
            installed: true,
            enabled: false,
            db_path: Some(PathBuf::from("/tmp/database.log")),
            extension_path: None,
        };
        let json = serde_json::to_string(&status).expect("serialise GchStatus");
        assert!(json.contains("\"installed\":true"));
        assert!(json.contains("\"enabled\":false"));
    }
}
