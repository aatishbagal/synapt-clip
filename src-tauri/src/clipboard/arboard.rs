use tokio::sync::mpsc;

use super::watcher::{ClipboardWatcher, NewClip, WatcherError};

/// Clipboard watcher using the arboard crate.
///
/// Polls the system clipboard at a fixed interval and sends new
/// text entries through the channel. Works on X11, XWayland, and Windows.
pub struct ArboardWatcher {
    poll_interval_ms: u64,
}

impl ArboardWatcher {
    /// Create a new arboard watcher with the given poll interval in milliseconds.
    pub fn new(poll_interval_ms: u64) -> Self {
        Self { poll_interval_ms }
    }
}

#[async_trait::async_trait]
impl ClipboardWatcher for ArboardWatcher {
    async fn watch(&self, tx: mpsc::Sender<NewClip>) -> Result<(), WatcherError> {
        let interval_ms = self.poll_interval_ms;

        let result = tokio::task::spawn_blocking(move || {
            let mut clipboard = arboard::Clipboard::new().map_err(|e| {
                WatcherError::Arboard(format!("failed to initialize clipboard: {e}"))
            })?;

            let mut last_seen = String::new();

            loop {
                std::thread::sleep(std::time::Duration::from_millis(interval_ms));

                let text = match clipboard.get_text() {
                    Ok(t) => t,
                    Err(arboard::Error::ContentNotAvailable) => continue,
                    Err(e) => {
                        tracing::warn!("Clipboard read error: {e}");
                        continue;
                    }
                };

                if text.is_empty() || text == last_seen {
                    continue;
                }

                last_seen = text.clone();

                let clip = NewClip {
                    content: text,
                    content_type: "text".to_string(),
                    source_app: None,
                };

                if tx.blocking_send(clip).is_err() {
                    tracing::info!("Clipboard watcher channel closed, stopping");
                    return Ok::<(), WatcherError>(());
                }
            }
        })
        .await;

        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(WatcherError::WatcherStopped),
        }
    }
}
