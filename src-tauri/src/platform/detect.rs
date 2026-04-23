//! Runtime clipboard backend detection.
//!
//! Determines which clipboard backend to use based on the current
//! platform and session type. In v0.1, always returns Arboard.
//! Wayland backends are added in v0.3.

/// Clipboard backend to use at runtime.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardBackend {
    /// X11, XWayland, and Windows — uses arboard crate for polling.
    Arboard,
    /// Wayland with wlroots compositors (Sway, Hyprland) — not implemented until v0.3.
    WlrDataControl,
    /// Wayland with GNOME Mutter via GCH extension file watcher — not implemented until v0.3.
    GchFile,
}

/// Detect the appropriate clipboard backend for the current session.
///
/// On Windows, always returns `Arboard`. On Linux, checks `XDG_SESSION_TYPE`
/// to determine if the session is Wayland or X11. In v0.1, Wayland sessions
/// fall back to Arboard via XWayland.
pub fn detect_backend() -> ClipboardBackend {
    #[cfg(target_os = "windows")]
    {
        return ClipboardBackend::Arboard;
    }

    #[cfg(target_os = "linux")]
    {
        match std::env::var("XDG_SESSION_TYPE").as_deref() {
            Ok("wayland") => {
                if std::env::var("WAYLAND_DISPLAY").is_ok() {
                    tracing::warn!(
                        "Wayland detected — falling back to arboard via XWayland in v0.1. \
                         Full Wayland support added in v0.3."
                    );
                }
                ClipboardBackend::Arboard
            }
            _ => ClipboardBackend::Arboard,
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        ClipboardBackend::Arboard
    }
}
