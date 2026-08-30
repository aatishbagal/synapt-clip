use std::collections::HashMap;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::dsa::PersistentList;
use crate::platform::{detect_backend, ClipboardBackend};
use crate::storage::Clip;
use crate::AppState;

#[tauri::command]
pub async fn get_clips(
    state: State<'_, AppState>,
    category: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<Clip>, String> {
    let cap = limit.unwrap_or(500);
    match category.as_deref() {
        Some(c) if !c.is_empty() => state
            .db
            .get_clips_by_category(c)
            .await
            .map(|mut v| {
                if (v.len() as i64) > cap {
                    v.truncate(cap as usize);
                }
                v
            })
            .map_err(|e| e.to_string()),
        _ => state
            .db
            .get_recent_clips(cap)
            .await
            .map_err(|e| e.to_string()),
    }
}

#[tauri::command]
pub async fn get_pinned_clips(state: State<'_, AppState>) -> Result<Vec<Clip>, String> {
    state
        .db
        .get_pinned_clips()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn copy_clip(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let clip = state
        .db
        .get_clip_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Clip not found".to_string())?;

    tokio::task::spawn_blocking(move || {
        let mut clipboard =
            arboard::Clipboard::new().map_err(|e| format!("Failed to open clipboard: {e}"))?;
        clipboard
            .set_text(&clip.content)
            .map_err(|e| format!("Failed to set clipboard: {e}"))?;
        Ok::<(), String>(())
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

#[tauri::command]
pub async fn delete_clip(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
) -> Result<(), String> {
    state
        .db
        .soft_delete_clip(id)
        .await
        .map_err(|e| e.to_string())?;
    state.search_engine.lock().await.remove_clip(id, "");
    state.clip_index.lock().await.remove(id);

    {
        let mut history = state.clip_history.lock().await;
        let index = history.current().iter().position(|c| c.id == id);
        if let Some(idx) = index {
            let new_version = history.current().remove_at(idx);
            history.push(new_version);
        }
    }

    let _ = app.emit("clip:deleted", id);
    Ok(())
}

#[tauri::command]
pub async fn undo_delete(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<Clip>, String> {
    let restored_id = {
        let mut history = state.clip_history.lock().await;
        let before: std::collections::HashSet<i64> =
            history.current().iter().map(|c| c.id).collect();
        match history.undo() {
            Some(restored) => {
                let after: std::collections::HashSet<i64> =
                    restored.iter().map(|c| c.id).collect();
                after.difference(&before).copied().next()
            }
            None => None,
        }
    };

    let Some(id) = restored_id else {
        return Ok(None);
    };

    let clip = state
        .db
        .restore_clip(id)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(ref c) = clip {
        state.search_engine.lock().await.index_clip(c.id, &c.content);
        let _ = app.emit("clip:restored", c);
    }
    Ok(clip)
}

#[tauri::command]
pub async fn get_clip_groups(state: State<'_, AppState>) -> Result<Vec<Vec<i64>>, String> {
    let mut mgr = state.group_manager.lock().await;
    Ok(mgr.all_groups())
}

#[tauri::command]
pub async fn get_group_for_clip(
    state: State<'_, AppState>,
    clip_id: i64,
) -> Result<Vec<i64>, String> {
    let mut mgr = state.group_manager.lock().await;
    Ok(mgr.get_group(clip_id))
}

#[tauri::command]
pub async fn clear_all_clips(state: State<'_, AppState>) -> Result<(), String> {
    state
        .db
        .clear_all_clips()
        .await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResultDto {
    pub clip_id: i64,
    pub score: f32,
    pub match_type: String,
    pub match_positions: Vec<(usize, usize)>,
}

#[tauri::command]
pub async fn search_clips(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<SearchResultDto>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let recent = state
        .db
        .get_recent_clips(500)
        .await
        .map_err(|e| e.to_string())?;

    let recent_ids: Vec<i64> = recent.iter().take(20).map(|c| c.id).collect();
    let all_clips: Vec<(i64, String, bool)> = recent
        .into_iter()
        .map(|c| (c.id, c.content, c.pinned))
        .collect();

    let engine = state.search_engine.lock().await;
    let mut results = engine.search(&query, &all_clips, &recent_ids);

    let ranked_ids = state.clip_index.lock().await.iter_ranked();
    let rank_map: std::collections::HashMap<i64, usize> = ranked_ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();

    results.sort_by(|a, b| {
        let ra = rank_map.get(&a.clip_id).copied().unwrap_or(usize::MAX);
        let rb = rank_map.get(&b.clip_id).copied().unwrap_or(usize::MAX);
        let score_ord = b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal);
        if a.score == b.score {
            ra.cmp(&rb)
        } else {
            score_ord
        }
    });

    Ok(results
        .into_iter()
        .take(50)
        .map(|r| SearchResultDto {
            clip_id: r.clip_id,
            score: r.score,
            match_type: r.match_type.as_str().to_string(),
            match_positions: r.match_positions,
        })
        .collect())
}

#[tauri::command]
pub async fn toggle_pin(state: State<'_, AppState>, clip_id: i64) -> Result<bool, String> {
    let new_pinned = state
        .db
        .toggle_pin(clip_id)
        .await
        .map_err(|e| e.to_string())?;

    if let Ok(Some(clip)) = state.db.get_clip_by_id(clip_id).await {
        let new_score = crate::clip_score(&clip.created_at, new_pinned);
        state.clip_index.lock().await.update_score(clip_id, new_score);
    }

    Ok(new_pinned)
}

#[tauri::command]
pub async fn assign_category(
    state: State<'_, AppState>,
    clip_id: i64,
    category: Option<String>,
) -> Result<(), String> {
    state
        .db
        .assign_category(clip_id, category.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_categories(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state
        .db
        .get_categories()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_category(state: State<'_, AppState>, name: String) -> Result<(), String> {
    state
        .db
        .delete_category(&name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn bulk_delete(
    app: AppHandle,
    state: State<'_, AppState>,
    clip_ids: Vec<i64>,
) -> Result<(), String> {
    state
        .db
        .bulk_delete(clip_ids.clone())
        .await
        .map_err(|e| e.to_string())?;

    {
        let mut engine = state.search_engine.lock().await;
        for id in &clip_ids {
            engine.remove_clip(*id, "");
        }
    }

    {
        let mut history = state.clip_history.lock().await;
        for id in &clip_ids {
            if let Some(idx) = history.current().iter().position(|c| c.id == *id) {
                let v = history.current().remove_at(idx);
                history.push(v);
            }
        }
    }

    for id in &clip_ids {
        let _ = app.emit("clip:deleted", id);
    }
    Ok(())
}

#[tauri::command]
pub async fn clear_history(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<u64, String> {
    let count = state
        .db
        .clear_history()
        .await
        .map_err(|e| e.to_string())?;

    let remaining = state
        .db
        .get_recent_clips(5000)
        .await
        .map_err(|e| e.to_string())?;

    {
        let mut engine = state.search_engine.lock().await;
        *engine = crate::search::engine::SearchEngine::new();
        for c in &remaining {
            engine.index_clip(c.id, &c.content);
        }
    }

    {
        let mut history = state.clip_history.lock().await;
        let mut rebuilt: PersistentList<Clip> = PersistentList::new();
        for c in remaining.iter().rev() {
            rebuilt = rebuilt.prepend(c.clone());
        }
        history.push(rebuilt);
    }

    let _ = app.emit("history:cleared", count);
    Ok(count)
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<HashMap<String, String>, String> {
    state
        .db
        .get_all_settings()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_setting(
    app: AppHandle,
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    state
        .db
        .set_setting(&key, &value)
        .await
        .map_err(|e| e.to_string())?;

    match key.as_str() {
        "history_limit" => {
            if let Ok(limit) = value.parse::<i64>() {
                if limit > 0 {
                    state
                        .db
                        .enforce_history_limit(limit)
                        .await
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        "expiry_days" => {
            if let Ok(days) = value.parse::<i64>() {
                if days > 0 {
                    state
                        .db
                        .expire_older_than_days(days)
                        .await
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        "theme" => {
            let _ = app.emit("settings:theme_changed", &value);
        }
        "hotkey" => {
            match crate::hotkey::register_hotkey(&app, &value) {
                Ok(()) => {
                    let _ = app.emit("settings:hotkey_changed", &value);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        _ => {}
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct PlatformInfo {
    pub backend: String,
    pub session_type: String,
    pub gch_installed: bool,
    pub os: String,
}

#[tauri::command]
pub async fn get_platform_info() -> Result<PlatformInfo, String> {
    let backend = detect_backend();
    let backend_str = match backend {
        ClipboardBackend::Arboard => "arboard",
        ClipboardBackend::WlrDataControl => "wlr",
        ClipboardBackend::GchFile
        | ClipboardBackend::GchNotEnabled
        | ClipboardBackend::GchNotInstalled => "gch",
        ClipboardBackend::Unsupported => "unsupported",
    }
    .to_string();

    let session_type = if cfg!(target_os = "windows") {
        "windows".to_string()
    } else {
        std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "x11".to_string())
    };

    let gch_installed = crate::clipboard::gch::detect_gch().installed;

    Ok(PlatformInfo {
        backend: backend_str,
        session_type,
        gch_installed,
        os: std::env::consts::OS.to_string(),
    })
}

/// Release the global hotkey while the settings recorder is capturing keys.
///
/// Without this the OS consumes the current combination before the webview sees
/// it, so pressing it in the recorder triggers the panel instead of being
/// recorded. Always paired with `resume_hotkey`.
#[tauri::command]
pub async fn pause_hotkey(app: AppHandle) -> Result<(), String> {
    crate::hotkey::unregister_hotkey(&app)
}

/// Re-register the stored hotkey after the recorder closes.
///
/// Reads the preference back from the database rather than trusting the caller,
/// so a cancelled recording restores exactly what was in effect before.
#[tauri::command]
pub async fn resume_hotkey(
    app: AppHandle,
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    let hotkey = state
        .db
        .get_setting("hotkey")
        .await
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| crate::hotkey::DEFAULT_HOTKEY.to_string());
    crate::hotkey::register_hotkey(&app, &hotkey)
}

/// Report whether the configured global hotkey is currently registered with the OS.
#[tauri::command]
pub async fn get_hotkey_status(app: AppHandle) -> Result<bool, String> {
    Ok(crate::hotkey::is_hotkey_registered(&app))
}

/// Show and focus the dedicated settings window.
#[tauri::command]
pub async fn open_settings(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("settings")
        .ok_or_else(|| "settings window not found".to_string())?;
    window.center().map_err(|e| e.to_string())?;
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

/// Hide the dedicated settings window.
#[tauri::command]
pub async fn close_settings(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Enable or disable launching SynaptClip automatically on login.
#[tauri::command]
pub async fn set_autostart(enabled: bool, _app: AppHandle) -> Result<(), String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .to_string_lossy()
        .to_string();
    if enabled {
        crate::platform::autostart::enable(&exe_path).map_err(|e| e.to_string())
    } else {
        crate::platform::autostart::disable().map_err(|e| e.to_string())
    }
}

/// Report whether SynaptClip is configured to launch on login.
#[tauri::command]
pub async fn get_autostart() -> Result<bool, String> {
    crate::platform::autostart::is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_ranked_clips(
    state: State<'_, AppState>,
    limit: usize,
) -> Result<Vec<i64>, String> {
    Ok(state.clip_index.lock().await.top_n(limit))
}

#[tauri::command]
pub async fn get_log_path() -> Result<String, String> {
    Ok(crate::log_file_path()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unavailable".to_string()))
}

/// Path of the crash log, or None when no crash has been recorded.
///
/// Distinct from [`get_log_path`], which points at the ordinary tracing log.
/// Returning None for a missing file lets the UI offer the crash log only when
/// there is actually something in it.
#[tauri::command]
pub fn get_crash_log_path() -> Option<String> {
    crate::crash::log_path()
        .filter(|p| p.is_file())
        .map(|p| p.to_string_lossy().to_string())
}

/// Version of the running build.
///
/// Served from the binary so the UI never carries a copy that can drift from
/// Cargo.toml and tauri.conf.json.
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Whether automatic update checks are enabled. Defaults to true when unset.
pub async fn auto_update_enabled(db: &crate::storage::Db) -> bool {
    db.get_setting("auto_update_check").await.ok().flatten().as_deref() != Some("false")
}

/// Run the startup update check, when the setting allows it.
///
/// Emits `update-available` so an open Settings window can show the result
/// without the user asking.
pub async fn run_auto_update_check(app: &AppHandle, db: &crate::storage::Db) {
    if !auto_update_enabled(db).await {
        tracing::debug!("automatic update check is disabled");
        return;
    }
    match check_for_update(app.clone()).await {
        Ok(Some(info)) => {
            tracing::info!("update available: v{}", info.version);
            if let Err(e) = app.emit("update-available", &info) {
                tracing::warn!("could not emit update-available: {e}");
            }
        }
        Ok(None) => tracing::info!("update check: already up to date"),
        Err(e) => tracing::warn!("update check failed: {e}"),
    }
}

/// An available update, as shown in the Settings Updates section.
#[derive(Clone, serde::Serialize)]
pub struct UpdateInfo {
    /// Version offered by the update endpoint.
    pub version: String,
    /// Version currently running.
    pub current: String,
    /// Release notes from the endpoint, empty when it supplies none.
    pub notes: String,
}

/// Check the update endpoint, returning None when already up to date.
#[tauri::command]
pub async fn check_for_update(app: tauri::AppHandle) -> Result<Option<UpdateInfo>, String> {
    use tauri_plugin_updater::UpdaterExt;

    let update = app
        .updater()
        .map_err(|e| e.to_string())?
        .check()
        .await
        .map_err(|e| e.to_string())?;

    Ok(update.map(|u| UpdateInfo {
        version: u.version.clone(),
        current: u.current_version.clone(),
        notes: u.body.clone().unwrap_or_default(),
    }))
}

/// Download and install the available update, then restart into it.
///
/// Re-checks rather than taking a handle from [`check_for_update`], since the
/// plugin's update handle is not `Send` across the command boundary.
#[tauri::command]
pub async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_updater::UpdaterExt;

    let update = app
        .updater()
        .map_err(|e| e.to_string())?
        .check()
        .await
        .map_err(|e| e.to_string())?;

    match update {
        Some(update) => {
            update
                .download_and_install(|_chunk, _total| {}, || {})
                .await
                .map_err(|e| e.to_string())?;
            app.restart();
        }
        None => Err("no update available".to_string()),
    }
}

/// Result of queuing a clip transfer through Synapt.
#[derive(Debug, serde::Serialize)]
pub struct SendResult {
    pub transfer_id: String,
    pub status: String,
}

/// Verify the bridge is active and the target peer is currently online.
/// Pure over the bridge snapshot so it can be unit-tested without a live app.
fn ensure_peer_sendable(
    bridge: &crate::synapt::bridge::BridgeState,
    peer_id: &str,
) -> Result<(), String> {
    if !bridge.active {
        return Err("Synapt is not running".to_string());
    }
    if !bridge.peers.iter().any(|p| p.id == peer_id && p.online) {
        return Err("Peer not found or is offline".to_string());
    }
    Ok(())
}

/// POST a clip to Synapt's transfer API and map the response to a result.
async fn post_clip_to_synapt(peer_id: &str, content: &str) -> Result<SendResult, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .map_err(|e| e.to_string())?;

    let body = serde_json::json!({
        "peer_id": peer_id,
        "content": content,
        "content_type": "text",
    });

    let resp = client
        .post("http://127.0.0.1:57321/v1/clips/send")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    match resp.status().as_u16() {
        202 => {
            #[derive(serde::Deserialize)]
            struct Resp {
                transfer_id: String,
                status: String,
            }
            let r: Resp = resp
                .json()
                .await
                .map_err(|e| format!("Response parse error: {e}"))?;
            Ok(SendResult {
                transfer_id: r.transfer_id,
                status: r.status,
            })
        }
        404 => Err("Peer is no longer available".to_string()),
        422 => Err("Invalid request".to_string()),
        503 => Err("Synapt transfer service is not available".to_string()),
        s => Err(format!("Unexpected status: {s}")),
    }
}

/// Send the given clip content to a peer device via Synapt's P2P transfer layer.
#[tauri::command]
pub async fn send_clip_to_peer(
    peer_id: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<SendResult, String> {
    {
        let guard = state
            .bridge_state
            .read()
            .map_err(|_| "Bridge state unavailable".to_string())?;
        ensure_peer_sendable(&guard, &peer_id)?;
    }
    post_clip_to_synapt(&peer_id, &content).await
}

/// Send the most recent locally captured clip to a peer device.
#[tauri::command]
pub async fn send_latest_clip_to_peer(
    peer_id: String,
    state: State<'_, AppState>,
) -> Result<SendResult, String> {
    {
        let guard = state
            .bridge_state
            .read()
            .map_err(|_| "Bridge state unavailable".to_string())?;
        ensure_peer_sendable(&guard, &peer_id)?;
    }
    let content = state
        .db
        .get_latest_clip_content()
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No clips to send".to_string())?;
    post_clip_to_synapt(&peer_id, &content).await
}

/// Return the current Synapt bridge state for the frontend.
#[tauri::command]
pub fn get_bridge_state(
    state: State<'_, AppState>,
) -> crate::synapt::bridge::BridgeState {
    match state.bridge_state.read() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

/// Force an immediate peer fetch from Synapt, outside the polling loop.
#[tauri::command]
pub async fn refresh_bridge_peers(
    _state: State<'_, AppState>,
) -> Result<Vec<crate::synapt::bridge::SynaptPeer>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get("http://127.0.0.1:57321/v1/peers")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    #[derive(serde::Deserialize)]
    struct R {
        peers: Vec<crate::synapt::bridge::SynaptPeer>,
    }
    let body: R = resp.json().await.map_err(|e| e.to_string())?;
    Ok(body.peers)
}

#[tauri::command]
pub async fn get_auto_categories() -> Result<Vec<String>, String> {
    Ok(vec![
        "Link".to_string(),
        "File Path".to_string(),
        "Code".to_string(),
        "Email".to_string(),
        "Color".to_string(),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synapt::bridge::{BridgeState, SynaptPeer};

    fn online_peer(id: &str) -> SynaptPeer {
        SynaptPeer {
            id: id.to_string(),
            name: "device".to_string(),
            ip: "192.168.1.10".to_string(),
            port: 54321,
            online: true,
            last_seen: "2025-03-14T10:22:00Z".to_string(),
        }
    }

    #[test]
    fn send_rejected_when_bridge_inactive() {
        let bridge = BridgeState::inactive();
        let err = ensure_peer_sendable(&bridge, "peer-1").expect_err("should reject");
        assert_eq!(err, "Synapt is not running");
    }

    #[test]
    fn send_rejected_when_peer_not_present() {
        let bridge = BridgeState {
            active: true,
            peers: vec![online_peer("peer-1")],
            api_version: Some("1".to_string()),
        };
        let err = ensure_peer_sendable(&bridge, "peer-unknown").expect_err("should reject");
        assert_eq!(err, "Peer not found or is offline");
    }

    #[test]
    fn send_rejected_when_peer_offline() {
        let mut peer = online_peer("peer-1");
        peer.online = false;
        let bridge = BridgeState {
            active: true,
            peers: vec![peer],
            api_version: Some("1".to_string()),
        };
        let err = ensure_peer_sendable(&bridge, "peer-1").expect_err("should reject");
        assert_eq!(err, "Peer not found or is offline");
    }

    #[test]
    fn send_allowed_for_online_peer() {
        let bridge = BridgeState {
            active: true,
            peers: vec![online_peer("peer-1")],
            api_version: Some("1".to_string()),
        };
        assert!(ensure_peer_sendable(&bridge, "peer-1").is_ok());
    }

    #[test]
    fn send_result_serialises() {
        let r = SendResult {
            transfer_id: "transfer-9".to_string(),
            status: "queued".to_string(),
        };
        let json = serde_json::to_value(&r).expect("serialise");
        assert_eq!(json["transfer_id"], "transfer-9");
        assert_eq!(json["status"], "queued");
    }
}
