//! Select a desktop portal on Wayland and the native shortcut plugin elsewhere.

use serde::Serialize;
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// Default shortcut used before a preference has been saved.
pub const DEFAULT_HOTKEY: &str = "Super+Shift+V";

/// Shortcut state shown in Settings, including the desktop's actual binding.
#[derive(Serialize)]
pub struct HotkeyInfo {
    /// Whether shortcuts are managed by the Wayland desktop.
    pub wayland: bool,
    /// Whether a live binding has been granted.
    pub registered: bool,
    /// The actual trigger description, when available.
    pub trigger: Option<String>,
    /// A registration or session error suitable for display.
    pub error: Option<String>,
}

fn registered_shortcut() -> &'static Mutex<Option<String>> {
    static REGISTERED: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    REGISTERED.get_or_init(|| Mutex::new(None))
}

fn registration_error() -> &'static Mutex<Option<String>> {
    static ERROR: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    ERROR.get_or_init(|| Mutex::new(None))
}

#[cfg(target_os = "linux")]
fn portal() -> &'static synapt_core::portal_hotkey::PortalHotkey {
    static PORTAL: OnceLock<synapt_core::portal_hotkey::PortalHotkey> = OnceLock::new();
    PORTAL.get_or_init(Default::default)
}

/// Detect Wayland independently of the presence of an XWayland DISPLAY.
pub fn uses_portal() -> bool {
    #[cfg(target_os = "linux")]
    return synapt_core::portal_hotkey::is_wayland_session(
        std::env::var("XDG_SESSION_TYPE").ok().as_deref(),
        std::env::var("WAYLAND_DISPLAY").ok().as_deref(),
    );
    #[cfg(not(target_os = "linux"))]
    false
}

fn activate(app: &AppHandle, token: Option<String>, timestamp: u32) {
    let handle = app.clone();
    if let Err(error) = app.run_on_main_thread(move || {
        if let Some(window) = handle.get_webview_window("main") {
            let _ = window.center();
            #[cfg(target_os = "linux")]
            if uses_portal() {
                use gtk::prelude::*;
                if let Ok(gtk_window) = window.gtk_window() {
                    if let Some(token) = token.as_deref() {
                        gtk_window.set_startup_id(token);
                    }
                    gtk_window.present_with_time(timestamp);
                    return;
                }
            }
            #[cfg(not(target_os = "linux"))]
            let _ = (token, timestamp);
            let _ = window.show();
            let _ = window.set_focus();
        }
    }) {
        tracing::warn!("hotkey: could not present window: {error}");
    }
}

/// Register the saved shortcut through the backend appropriate to this session.
pub async fn register_hotkey(app: &AppHandle, hotkey: &str) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    if uses_portal() {
        let handle = app.clone();
        let result = portal()
            .register(
                &app.config().identifier,
                "Open SynaptClip",
                hotkey,
                move |event| {
                    activate(&handle, event.token, event.timestamp);
                },
            )
            .await;
        if result.is_ok() {
            tracing::info!(
                "hotkey: Wayland portal binding active: {:?}",
                portal().status().trigger
            );
        }
        return result;
    }
    let result = register_native(app, hotkey);
    *registration_error()
        .lock()
        .unwrap_or_else(|p| p.into_inner()) = result.as_ref().err().cloned();
    result
}

fn register_native(app: &AppHandle, hotkey: &str) -> Result<(), String> {
    let trimmed = hotkey.trim();
    if trimmed.is_empty() {
        return Err("The shortcut cannot be empty".into());
    }
    // Validate before releasing the current binding.
    let shortcut: tauri_plugin_global_shortcut::Shortcut = trimmed
        .parse()
        .map_err(|e| format!("Invalid shortcut: {e}"))?;
    unregister_hotkey(app)?;
    app.global_shortcut()
        .on_shortcut(shortcut, |app, _, event| {
            if event.state == ShortcutState::Pressed {
                activate(app, None, 0);
            }
        })
        .map_err(|e| format!("Could not register shortcut: {e}"))?;
    *registered_shortcut()
        .lock()
        .unwrap_or_else(|p| p.into_inner()) = Some(trimmed.to_string());
    tracing::info!("hotkey: native binding active: {trimmed}");
    Ok(())
}

/// Release the native shortcut while the key recorder is active.
pub fn unregister_hotkey(app: &AppHandle) -> Result<(), String> {
    if uses_portal() {
        return Err("Use desktop shortcut settings to change the Wayland binding".into());
    }
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| e.to_string())?;
    *registered_shortcut()
        .lock()
        .unwrap_or_else(|p| p.into_inner()) = None;
    Ok(())
}

/// Return the state of the active backend without opening a consent dialog.
pub fn info(app: &AppHandle) -> HotkeyInfo {
    #[cfg(target_os = "linux")]
    if uses_portal() {
        let state = portal().status();
        return HotkeyInfo {
            wayland: true,
            registered: state.registered,
            trigger: state.trigger,
            error: state.error,
        };
    }
    let shortcut = registered_shortcut()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clone();
    let registered = shortcut
        .as_deref()
        .is_some_and(|key| app.global_shortcut().is_registered(key));
    HotkeyInfo {
        wayland: false,
        registered,
        trigger: shortcut,
        error: registration_error()
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone(),
    }
}

/// Report whether the active backend has a live shortcut binding.
pub fn is_hotkey_registered(app: &AppHandle) -> bool {
    info(app).registered
}

/// Open the compositor's editor or retry registration after a portal failure.
pub async fn configure(app: &AppHandle, hotkey: &str) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    if uses_portal() {
        if !portal().status().registered {
            register_hotkey(app, hotkey).await?;
        }
        return portal().configure().await;
    }
    let _ = (app, hotkey);
    Err("Desktop shortcut settings are only used on Wayland".into())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use tauri_plugin_global_shortcut::Shortcut;

    /// The recorder emits KeyboardEvent.code derived names. Every shape it can
    /// produce has to survive the plugin's parser.
    #[test]
    fn recorder_output_parses() {
        for combo in [
            "Super+Shift+V",
            "Ctrl+Alt+Shift+Super+V",
            "Super+Space",
            "Alt+1",
            "Ctrl+Up",
            "Super+Escape",
            "Ctrl+Alt+F12",
            "Super+BracketLeft",
            "Ctrl+Numpad0",
        ] {
            assert!(
                Shortcut::from_str(combo).is_ok(),
                "recorder produced an unparseable combination: {combo}"
            );
        }
    }

    /// Modifier spellings the recorder and stored settings may use are all
    /// accepted, so existing saved hotkeys keep working.
    #[test]
    fn modifier_aliases_are_equivalent() {
        for combo in ["Super+V", "super+v", "Cmd+V", "Command+V"] {
            assert!(Shortcut::from_str(combo).is_ok(), "rejected: {combo}");
        }
    }

    /// Regression guard: reading event.key instead of event.code on macOS turned
    /// Option+V into the composed character the OS cannot register.
    #[test]
    fn composed_macos_character_is_rejected() {
        assert!(Shortcut::from_str("Alt+\u{221a}").is_err());
    }
}
