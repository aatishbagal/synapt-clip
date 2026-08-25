pub mod autostart;
pub mod detect;
#[cfg(target_os = "macos")]
pub mod macos;

pub use detect::{detect_backend, ClipboardBackend};

/// Ask the user to confirm quitting, running `on_confirm` only if they agree.
///
/// macOS shows a native NSAlert, dispatched to the main thread because
/// `runModal` drives a nested AppKit run loop. Other platforms quit straight
/// away, matching the behaviour they had before the dialog existed.
pub fn confirm_quit<F>(handle: &tauri::AppHandle, on_confirm: F)
where
    F: FnOnce() + Send + 'static,
{
    #[cfg(target_os = "macos")]
    {
        if let Err(e) = handle.run_on_main_thread(move || {
            if macos::confirm_quit_dialog() {
                on_confirm();
            } else {
                tracing::info!("quit cancelled from the confirmation dialog");
            }
        }) {
            tracing::warn!("could not show quit confirmation on the main thread: {e}");
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = handle;
        on_confirm();
    }
}
