pub mod autostart;
pub mod detect;
#[cfg(target_os = "macos")]
pub mod macos;

pub use detect::{detect_backend, ClipboardBackend};
