//! Tray icon tooltip helpers. The tooltip is composed from a single source of
//! truth so the clip count and the Synapt bridge status never overwrite each
//! other: both are rendered in one string by [`tray_tooltip`].

const TRAY_ID: &str = "main";

/// Compose the tray tooltip from the clip count and the current bridge state.
/// The bridge portion is appended only when Synapt is connected.
pub fn tray_tooltip(clip_count: usize, bridge_active: bool, peer_count: usize) -> String {
    let clips = if clip_count == 1 { "clip" } else { "clips" };
    let mut tooltip = format!("SynaptClip — {clip_count} {clips}");

    if bridge_active {
        if peer_count > 0 {
            let devices = if peer_count == 1 { "device" } else { "devices" };
            tooltip.push_str(&format!(", Synapt connected ({peer_count} {devices})"));
        } else {
            tooltip.push_str(", Synapt connected");
        }
    }

    tooltip
}

/// Update the tray icon tooltip to reflect the current clip count and bridge state.
pub fn update_tray_tooltip(
    app: &tauri::AppHandle,
    clip_count: usize,
    bridge_active: bool,
    peer_count: usize,
) {
    let tooltip = tray_tooltip(clip_count, bridge_active, peer_count);
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(&tooltip));
    }
}

#[cfg(test)]
mod tests {
    use super::tray_tooltip;

    #[test]
    fn inactive_shows_only_clip_count() {
        assert_eq!(tray_tooltip(0, false, 0), "SynaptClip — 0 clips");
        assert_eq!(tray_tooltip(1, false, 3), "SynaptClip — 1 clip");
        assert_eq!(tray_tooltip(12, false, 0), "SynaptClip — 12 clips");
    }

    #[test]
    fn active_no_peers_appends_connected() {
        assert_eq!(
            tray_tooltip(5, true, 0),
            "SynaptClip — 5 clips, Synapt connected"
        );
    }

    #[test]
    fn active_one_peer_is_singular() {
        assert_eq!(
            tray_tooltip(5, true, 1),
            "SynaptClip — 5 clips, Synapt connected (1 device)"
        );
    }

    #[test]
    fn active_multiple_peers_is_plural() {
        assert_eq!(
            tray_tooltip(5, true, 2),
            "SynaptClip — 5 clips, Synapt connected (2 devices)"
        );
    }
}
