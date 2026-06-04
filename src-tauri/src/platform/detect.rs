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
    /// GNOME Wayland where the extension is installed but not enabled.
    GchNotEnabled,
    /// GNOME Wayland where the extension is not installed.
    GchNotInstalled,
    /// No supported clipboard backend for this session.
    Unsupported,
}

/// Detect the appropriate clipboard backend for the current session.
pub fn detect_backend() -> ClipboardBackend {
    #[cfg(target_os = "windows")]
    {
        tracing::info!("platform: Windows — using arboard backend");
        return ClipboardBackend::Arboard;
    }

    #[cfg(target_os = "macos")]
    {
        tracing::info!("platform: macOS — using arboard backend");
        return ClipboardBackend::Arboard;
    }

    #[cfg(target_os = "linux")]
    {
        // Explicit developer override: force XWayland/X11 arboard backend.
        if std::env::var("GDK_BACKEND").as_deref() == Ok("x11") {
            tracing::info!("platform: GDK_BACKEND=x11 set — using arboard via XWayland");
            return ClipboardBackend::Arboard;
        }

        // Check both session type and display server presence independently.
        let session_type = std::env::var("XDG_SESSION_TYPE")
            .unwrap_or_default()
            .to_lowercase();
        let wayland_display = std::env::var("WAYLAND_DISPLAY").ok();
        let x11_display = std::env::var("DISPLAY").ok();

        let is_wayland = session_type == "wayland" || wayland_display.is_some();
        let is_x11 = session_type == "x11" || (x11_display.is_some() && !is_wayland);

        if is_wayland {
            let desktop = std::env::var("XDG_CURRENT_DESKTOP")
                .unwrap_or_default()
                .to_ascii_lowercase();
            let is_gnome = desktop.contains("gnome") || desktop.contains("unity");

            // wlroots compositors (Sway, Hyprland) expose wlr/ext-data-control.
            // GNOME Mutter is handled via the Clipboard History extension: recent
            // Mutter answers wl-paste, so probing it would wrongly pick the wlr
            // backend on GNOME — hence GNOME is routed to GCH explicitly.
            if !is_gnome && probe_wlr_data_control(wayland_display.as_deref()) {
                tracing::info!("platform: Wayland wlroots detected — using wl-paste backend");
                return ClipboardBackend::WlrDataControl;
            }

            if is_gnome {
                // GNOME Wayland: rely on the Clipboard History extension.
                let gch = crate::clipboard::gch::detect_gch();
                if gch.installed && gch.enabled {
                    // Use the file backend when enabled; the watcher polls for
                    // the database if it has not been written yet.
                    tracing::info!("platform: GNOME Wayland with GCH extension — using GCH file watcher");
                    return ClipboardBackend::GchFile;
                }
                if gch.installed && !gch.enabled {
                    tracing::warn!("platform: GCH extension installed but not enabled");
                    return ClipboardBackend::GchNotEnabled;
                }
                // Not installed: fall back to XWayland arboard if available.
                if x11_display.is_some() {
                    tracing::warn!(
                        "platform: GNOME Wayland, GCH not installed — falling back to XWayland arboard"
                    );
                    return ClipboardBackend::Arboard;
                }
                tracing::warn!("platform: GNOME Wayland, GCH extension not found");
                return ClipboardBackend::GchNotInstalled;
            }

            // Non-GNOME Wayland without wlr support: try XWayland arboard.
            if x11_display.is_some() {
                tracing::warn!("platform: Wayland with no wlr support — falling back to XWayland arboard");
                return ClipboardBackend::Arboard;
            }

            tracing::error!("platform: Wayland with no supported clipboard backend");
            return ClipboardBackend::Unsupported;
        }

        if is_x11 {
            tracing::info!("platform: X11 detected — using arboard backend");
            return ClipboardBackend::Arboard;
        }

        tracing::warn!("platform: no display server detected — using arboard backend");
        ClipboardBackend::Arboard
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        ClipboardBackend::Arboard
    }
}

#[cfg(target_os = "linux")]
fn probe_wlr_data_control(wayland_display: Option<&str>) -> bool {
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    let child = Command::new("wl-paste")
        .arg("--list-types")
        .env("WAYLAND_DISPLAY", wayland_display.unwrap_or("wayland-0"))
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

/// Backend status surfaced to the frontend.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BackendStatusInfo {
    /// Active backend: `arboard` | `wlr` | `gch` | `unsupported`.
    pub backend: String,
    /// Session type: `x11` | `wayland` | `windows` | `macos` | `unknown`.
    pub session: String,
    /// Human-readable description of the detected backend.
    pub detail: String,
}

/// Compute the session string for the current process.
fn session_string() -> String {
    if cfg!(target_os = "windows") {
        return "windows".to_string();
    }
    if cfg!(target_os = "macos") {
        return "macos".to_string();
    }
    let session = std::env::var("XDG_SESSION_TYPE")
        .unwrap_or_default()
        .to_lowercase();
    match session.as_str() {
        "wayland" => "wayland".to_string(),
        "x11" => "x11".to_string(),
        _ => {
            if std::env::var("WAYLAND_DISPLAY").is_ok() {
                "wayland".to_string()
            } else if std::env::var("DISPLAY").is_ok() {
                "x11".to_string()
            } else {
                "unknown".to_string()
            }
        }
    }
}

/// Tauri command exposing backend status to the frontend.
#[tauri::command]
pub fn get_backend_status() -> BackendStatusInfo {
    let backend = detect_backend();
    let (backend_str, detail) = match backend {
        ClipboardBackend::Arboard => (
            "arboard",
            "Using the polling clipboard backend (X11/XWayland/Windows/macOS).".to_string(),
        ),
        ClipboardBackend::WlrDataControl => (
            "wlr",
            "wlroots Wayland — native wl-paste backend active.".to_string(),
        ),
        ClipboardBackend::GchFile => (
            "gch",
            "GNOME Wayland — using the Clipboard History extension.".to_string(),
        ),
        ClipboardBackend::GchNotEnabled => (
            "gch",
            "Clipboard History extension is installed but not enabled. Enable it in GNOME Extensions.".to_string(),
        ),
        ClipboardBackend::GchNotInstalled => (
            "gch",
            "Install the Clipboard History GNOME extension to use SynaptClip on GNOME Wayland.".to_string(),
        ),
        ClipboardBackend::Unsupported => (
            "unsupported",
            "No supported clipboard backend detected for this session.".to_string(),
        ),
    };

    BackendStatusInfo {
        backend: backend_str.to_string(),
        session: session_string(),
        detail,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wayland_detected_from_wayland_display_only() {
        // WAYLAND_DISPLAY set, XDG_SESSION_TYPE absent.
        let session_type = String::new();
        let wayland_display = Some("wayland-0".to_string());
        let is_wayland = session_type == "wayland" || wayland_display.is_some();
        assert!(is_wayland);
    }

    #[test]
    fn x11_detected_from_display_only() {
        // DISPLAY set, WAYLAND_DISPLAY absent.
        let session_type = String::new();
        let wayland_display: Option<String> = None;
        let x11_display = Some(":0".to_string());
        let is_wayland = session_type == "wayland" || wayland_display.is_some();
        let is_x11 = session_type == "x11" || (x11_display.is_some() && !is_wayland);
        assert!(!is_wayland);
        assert!(is_x11);
    }

    #[test]
    fn backend_status_serialises() {
        let info = get_backend_status();
        let json = serde_json::to_string(&info).expect("serialise BackendStatusInfo");
        assert!(json.contains("\"backend\""));
        assert!(json.contains("\"session\""));
        assert!(json.contains("\"detail\""));
    }
}
