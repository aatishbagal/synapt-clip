mod arboard;
pub mod gch;
mod watcher;
mod wlr;

pub use self::arboard::ArboardWatcher;
pub use self::gch::GchWatcher;
pub use self::wlr::WlrWatcher;
#[allow(unused_imports)]
pub use watcher::{ClipboardWatcher, NewClip, WatcherError};

use crate::platform::ClipboardBackend;
use tokio::sync::mpsc;

/// A watcher that does nothing — used for backends that require user setup
/// (GCH not installed/enabled) or unsupported sessions. The setup state is
/// surfaced to the frontend separately via `backend-status` events.
struct NoopWatcher;

#[async_trait::async_trait]
impl ClipboardWatcher for NoopWatcher {
    async fn watch(&self, _tx: mpsc::Sender<NewClip>) -> Result<(), WatcherError> {
        Ok(())
    }
}

/// Create a clipboard watcher for the given backend.
pub fn create_watcher(backend: &ClipboardBackend) -> Box<dyn ClipboardWatcher> {
    match backend {
        ClipboardBackend::Arboard => Box::new(ArboardWatcher::new(500)),
        ClipboardBackend::WlrDataControl => Box::new(WlrWatcher::new()),
        ClipboardBackend::GchFile => Box::new(GchWatcher::new()),
        ClipboardBackend::GchNotEnabled
        | ClipboardBackend::GchNotInstalled
        | ClipboardBackend::Unsupported => Box::new(NoopWatcher),
    }
}
