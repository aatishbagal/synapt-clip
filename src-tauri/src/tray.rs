//! Tray icon tooltip helpers driven by Synapt bridge state.

const TRAY_ID: &str = "main";

/// Compose the tray tooltip text for the given bridge state.
pub fn tray_tooltip(bridge_active: bool, peer_count: usize) -> String {
    if bridge_active && peer_count > 0 {
        format!(
            "SynaptClip — Synapt connected ({peer_count} device{})",
            if peer_count == 1 { "" } else { "s" }
        )
    } else if bridge_active {
        "SynaptClip — Synapt connected".to_string()
    } else {
        "SynaptClip".to_string()
    }
}

/// Update the tray icon tooltip to reflect the current bridge state.
pub fn update_tray_tooltip(app: &tauri::AppHandle, bridge_active: bool, peer_count: usize) {
    let tooltip = tray_tooltip(bridge_active, peer_count);
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(&tooltip));
    }
}

#[cfg(test)]
mod tests {
    use super::tray_tooltip;

    #[test]
    fn inactive_is_plain_name() {
        assert_eq!(tray_tooltip(false, 0), "SynaptClip");
        assert_eq!(tray_tooltip(false, 3), "SynaptClip");
    }

    #[test]
    fn active_no_peers_omits_count() {
        assert_eq!(tray_tooltip(true, 0), "SynaptClip — Synapt connected");
    }

    #[test]
    fn active_one_peer_is_singular() {
        assert_eq!(
            tray_tooltip(true, 1),
            "SynaptClip — Synapt connected (1 device)"
        );
    }

    #[test]
    fn active_multiple_peers_is_plural() {
        assert_eq!(
            tray_tooltip(true, 2),
            "SynaptClip — Synapt connected (2 devices)"
        );
    }
}
