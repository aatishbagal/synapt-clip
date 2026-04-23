mod arboard;
mod watcher;

pub use self::arboard::ArboardWatcher;
#[allow(unused_imports)]
pub use watcher::{ClipboardWatcher, NewClip, WatcherError};

use crate::platform::ClipboardBackend;

/// Create a clipboard watcher for the given backend.
///
/// In v0.1, all backends fall back to ArboardWatcher. The WlrDataControl
/// and GchFile arms will be replaced with real implementations in v0.3.
pub fn create_watcher(backend: &ClipboardBackend) -> Box<dyn ClipboardWatcher> {
    match backend {
        ClipboardBackend::Arboard => Box::new(ArboardWatcher::new(500)),
        ClipboardBackend::WlrDataControl => {
            tracing::warn!(
                "WlrDataControl backend not implemented in v0.1, falling back to arboard"
            );
            Box::new(ArboardWatcher::new(500))
        }
        ClipboardBackend::GchFile => {
            tracing::warn!(
                "GchFile backend not implemented in v0.1, falling back to arboard"
            );
            Box::new(ArboardWatcher::new(500))
        }
    }
}
