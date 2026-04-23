use tokio::sync::mpsc;

/// A new clipboard entry captured by a watcher backend.
#[derive(Debug, Clone)]
pub struct NewClip {
    /// The clipboard text content.
    pub content: String,
    /// Content type — always "text" in v0.1.
    pub content_type: String,
    /// Source application name, if detectable.
    pub source_app: Option<String>,
}

/// Errors that can occur in clipboard watchers.
#[derive(Debug, thiserror::Error)]
pub enum WatcherError {
    /// Error from the arboard clipboard backend.
    #[error("arboard error: {0}")]
    Arboard(String),
    /// The watcher was stopped (receiver dropped).
    #[error("watcher stopped")]
    WatcherStopped,
    /// A required binary, extension, or file was not found. The message is
    /// surfaced to the frontend as a setup prompt.
    #[error("setup required: {0}")]
    NotFound(String),
    /// An I/O error occurred while watching the clipboard source.
    #[error("io error: {0}")]
    Io(String),
}

/// Trait implemented by all clipboard backends.
///
/// Each backend watches the system clipboard and sends new entries
/// through the provided channel. The method runs indefinitely until
/// the receiver is dropped.
#[async_trait::async_trait]
pub trait ClipboardWatcher: Send + Sync {
    /// Start watching the clipboard. Send each new clip through `tx`.
    /// This method runs indefinitely until the receiver is dropped.
    async fn watch(&self, tx: mpsc::Sender<NewClip>) -> Result<(), WatcherError>;
}
