use std::sync::{Mutex, OnceLock};

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// Shortcut string that was last registered successfully, so the frontend can
/// tell whether the global hotkey is actually live.
fn registered_shortcut() -> &'static Mutex<Option<String>> {
    static REGISTERED: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    REGISTERED.get_or_init(|| Mutex::new(None))
}

fn set_registered(value: Option<String>) {
    match registered_shortcut().lock() {
        Ok(mut guard) => *guard = value,
        Err(poisoned) => *poisoned.into_inner() = value,
    }
}

fn current_registered() -> Option<String> {
    match registered_shortcut().lock() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

/// Register `hotkey_str` as the global shortcut that shows the clip panel,
/// replacing any shortcut registered earlier.
///
/// Modifier names are parsed case-insensitively by the global shortcut plugin.
/// `Super`, `Cmd` and `Command` all mean the Command key on macOS and the
/// Windows key elsewhere; `Alt` and `Option` are interchangeable.
pub fn register_hotkey(app: &AppHandle, hotkey_str: &str) -> Result<(), String> {
    let trimmed = hotkey_str.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    if let Err(e) = app.global_shortcut().unregister_all() {
        tracing::warn!("Failed to unregister existing shortcuts: {e}");
    }
    set_registered(None);

    let app_clone = app.clone();
    let result = app
        .global_shortcut()
        .on_shortcut(trimmed, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                if let Some(window) = app_clone.get_webview_window("main") {
                    let _ = window.center();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        });

    match result {
        Ok(()) => {
            set_registered(Some(trimmed.to_string()));
            Ok(())
        }
        Err(e) => {
            // On macOS the OS refuses combinations already owned by the system
            // or another application (Command+Space belongs to Spotlight, for
            // example). On Windows the same shows up as ERROR_HOTKEY_ALREADY_REGISTERED.
            tracing::warn!(
                "hotkey: failed to register '{trimmed}': {e}. \
                 The combination is most likely already claimed by the system or another application"
            );
            Err(format!("Failed to register hotkey '{trimmed}': {e}"))
        }
    }
}

/// Report whether the configured global hotkey is currently registered with the OS.
pub fn is_hotkey_registered(app: &AppHandle) -> bool {
    match current_registered() {
        Some(shortcut) => app.global_shortcut().is_registered(shortcut.as_str()),
        None => false,
    }
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
