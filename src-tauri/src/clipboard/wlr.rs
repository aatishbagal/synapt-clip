use std::process::Stdio;
use std::time::{Duration, Instant};

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

use super::watcher::{ClipboardWatcher, NewClip, WatcherError};

const MAX_CONSECUTIVE_FAILURES: u32 = 5;
const LONG_RUN_RESET_SECS: u64 = 60;
const MAX_BACKOFF_SECS: u64 = 30;

/// Clipboard watcher for wlroots-based Wayland compositors.
///
/// Spawns `wl-paste --watch cat` as a subprocess and forwards each
/// line of output as a new clip. Restarts on subprocess exit with
/// exponential backoff up to 30 seconds.
pub struct WlrWatcher {
    wl_paste_path: String,
}

impl WlrWatcher {
    /// Create a watcher that resolves `wl-paste` from `PATH`.
    pub fn new() -> Self {
        Self {
            wl_paste_path: "wl-paste".to_string(),
        }
    }

    async fn run_once(&self, tx: &mpsc::Sender<NewClip>) -> Result<(), WatcherError> {
        let mut child = Command::new(&self.wl_paste_path)
            .arg("--watch")
            .arg("cat")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| WatcherError::Io(format!("failed to spawn wl-paste: {e}")))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| WatcherError::Io("wl-paste stdout unavailable".to_string()))?;

        let mut reader = BufReader::new(stdout).lines();

        loop {
            tokio::select! {
                line = reader.next_line() => {
                    match line {
                        Ok(Some(raw)) => {
                            let trimmed = raw.trim();
                            if trimmed.is_empty() {
                                continue;
                            }
                            let clip = NewClip {
                                content: trimmed.to_string(),
                                content_type: "text".to_string(),
                                source_app: None,
                            };
                            if tx.send(clip).await.is_err() {
                                let _ = child.kill().await;
                                return Err(WatcherError::WatcherStopped);
                            }
                        }
                        Ok(None) => break,
                        Err(e) => {
                            let _ = child.kill().await;
                            return Err(WatcherError::Io(format!("wl-paste read error: {e}")));
                        }
                    }
                }
            }
        }

        let status = child
            .wait()
            .await
            .map_err(|e| WatcherError::Io(format!("wl-paste wait error: {e}")))?;

        tracing::warn!("wl-paste exited: {status}");
        Ok(())
    }
}

impl Default for WlrWatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ClipboardWatcher for WlrWatcher {
    async fn watch(&self, tx: mpsc::Sender<NewClip>) -> Result<(), WatcherError> {
        if !binary_available(&self.wl_paste_path).await {
            return Err(WatcherError::NotFound(
                "wl-paste not found — install wl-clipboard package".to_string(),
            ));
        }

        let mut failures: u32 = 0;

        loop {
            let started = Instant::now();
            match self.run_once(&tx).await {
                Ok(()) => {}
                Err(WatcherError::WatcherStopped) => return Ok(()),
                Err(e) => {
                    tracing::warn!("wl-paste run failed: {e}");
                }
            }

            if started.elapsed().as_secs() >= LONG_RUN_RESET_SECS {
                failures = 0;
            }

            failures += 1;
            if failures > MAX_CONSECUTIVE_FAILURES {
                tracing::error!("wl-paste failed {failures} times, giving up");
                return Err(WatcherError::Io(
                    "wl-paste exited repeatedly, giving up".to_string(),
                ));
            }

            let backoff = (1u64 << (failures - 1)).min(MAX_BACKOFF_SECS);
            tracing::warn!("Restarting wl-paste in {backoff}s (attempt {failures})");
            tokio::time::sleep(Duration::from_secs(backoff)).await;
        }
    }
}

async fn binary_available(path: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {path}"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}
