use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

use super::watcher::{ClipboardWatcher, NewClip, WatcherError};

const EXTENSION_ID: &str = "clipboard-history@alexsaveau.dev";
const OP_ADD: u8 = 0x01;
const OP_DELETE: u8 = 0x02;
const OP_MOVE: u8 = 0x03;

/// Clipboard watcher for GNOME via the Clipboard History extension.
///
/// Reads the append-only binary log at
/// `$XDG_DATA_HOME/gnome-shell/extensions/<ext>/database.log` and emits
/// a `NewClip` for every Add operation.
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

    fn gch_file_path(&self) -> Option<PathBuf> {
        let base = dirs::data_dir()?
            .join("gnome-shell")
            .join("extensions")
            .join(&self.extension_id)
            .join("database.log");
        if base.exists() {
            Some(base)
        } else {
            None
        }
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
        let path = self.gch_file_path().ok_or_else(|| {
            WatcherError::NotFound(
                "Clipboard History GNOME extension not found. \
                 Install it from extensions.gnome.org"
                    .to_string(),
            )
        })?;

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
}
