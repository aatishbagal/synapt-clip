mod arboard;
mod gch;
mod watcher;
mod wlr;

pub use self::arboard::ArboardWatcher;
pub use self::gch::GchWatcher;
pub use self::wlr::WlrWatcher;
#[allow(unused_imports)]
pub use watcher::{ClipboardWatcher, NewClip, WatcherError};

use crate::platform::ClipboardBackend;

/// Create a clipboard watcher for the given backend.
pub fn create_watcher(backend: &ClipboardBackend) -> Box<dyn ClipboardWatcher> {
    match backend {
        ClipboardBackend::Arboard => Box::new(ArboardWatcher::new(500)),
        ClipboardBackend::WlrDataControl => Box::new(WlrWatcher::new()),
        ClipboardBackend::GchFile => Box::new(GchWatcher::new()),
    }
}
