/// Clipboard backend to use at runtime.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardBackend {
    /// X11, XWayland, and Windows — uses arboard crate for polling.
    Arboard,
    /// Wayland with wlroots compositors (Sway, Hyprland) via wl-paste.
    WlrDataControl,
    /// Wayland with GNOME Mutter via the Clipboard History extension log.
    GchFile,
}

/// Detect the appropriate clipboard backend for the current session.
pub fn detect_backend() -> ClipboardBackend {
    #[cfg(target_os = "windows")]
    {
        return ClipboardBackend::Arboard;
    }

    #[cfg(target_os = "linux")]
    {
        if std::env::var("GDK_BACKEND").as_deref() == Ok("x11") {
            tracing::info!("GDK_BACKEND=x11 set — using Arboard via XWayland");
            return ClipboardBackend::Arboard;
        }

        let session = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
        if session != "wayland" || std::env::var("WAYLAND_DISPLAY").is_err() {
            return ClipboardBackend::Arboard;
        }

        let desktop = std::env::var("XDG_CURRENT_DESKTOP")
            .unwrap_or_default()
            .to_ascii_lowercase();
        if desktop.contains("gnome") || desktop.contains("unity") {
            tracing::info!("GNOME Wayland detected — wlr-data-control unsupported, using GCH");
            return ClipboardBackend::GchFile;
        }

        if probe_wlr_data_control() {
            ClipboardBackend::WlrDataControl
        } else {
            tracing::info!("wlr-data-control probe failed — falling back to GCH file watcher");
            ClipboardBackend::GchFile
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        ClipboardBackend::Arboard
    }
}

#[cfg(target_os = "linux")]
fn probe_wlr_data_control() -> bool {
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    let child = Command::new("wl-paste")
        .arg("--list-types")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    let mut child = match child {
        Ok(c) => c,
        Err(_) => return false,
    };

    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    return false;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => return false,
        }
    }
}
