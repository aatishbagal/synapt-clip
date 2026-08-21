/// Platform-specific setup hook for macOS.
///
/// Hides the app from the Dock (tray-only app) by setting the activation policy
/// to accessory. The system tray uses Tauri's built-in tray (NSStatusBar) and
/// does not depend on AppIndicator/libayatana.
// The `objc` crate's `msg_send!`/`class!` macros expand a `cfg(feature = "cargo-clippy")`
// check that clippy's stable check-cfg lint doesn't recognize; this is upstream, not ours.
#[allow(unexpected_cfgs)]
pub fn setup() {
    // NSApplicationActivationPolicyAccessory = 1 (no Dock icon).
    unsafe {
        use objc::runtime::Object;
        use objc::{class, msg_send, sel, sel_impl};
        let ns_app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
        let _: () = msg_send![ns_app, setActivationPolicy: 1_isize];
    }
    tracing::info!("macOS: set activation policy to accessory (no dock icon)");
}
