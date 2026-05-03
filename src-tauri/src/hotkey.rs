use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub fn register_hotkey(app: &AppHandle, hotkey_str: &str) -> Result<(), String> {
    let trimmed = hotkey_str.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    if let Err(e) = app.global_shortcut().unregister_all() {
        tracing::warn!("Failed to unregister existing shortcuts: {e}");
    }

    let app_clone = app.clone();
    app.global_shortcut()
        .on_shortcut(trimmed, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                if let Some(window) = app_clone.get_webview_window("main") {
                    let _ = window.center();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .map_err(|e| format!("Failed to register hotkey '{trimmed}': {e}"))
}
