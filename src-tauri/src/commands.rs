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
    let results = engine.search(&query, &all_clips, &recent_ids);

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
    state
        .db
        .toggle_pin(clip_id)
        .await
        .map_err(|e| e.to_string())
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
        _ => {}
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct PlatformInfo {
    pub backend: String,
    pub session_type: String,
    pub gch_installed: bool,
}

#[tauri::command]
pub async fn get_platform_info() -> Result<PlatformInfo, String> {
    let backend = detect_backend();
    let backend_str = match backend {
        ClipboardBackend::Arboard => "arboard",
        ClipboardBackend::WlrDataControl => "wlr",
        ClipboardBackend::GchFile => "gch",
    }
    .to_string();

    let session_type = if cfg!(target_os = "windows") {
        "windows".to_string()
    } else {
        std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "x11".to_string())
    };

    let gch_installed = dirs::data_dir()
        .map(|d| {
            d.join("gnome-shell")
                .join("extensions")
                .join("clipboard-history@alexsaveau.dev")
                .join("database.log")
                .exists()
        })
        .unwrap_or(false);

    Ok(PlatformInfo {
        backend: backend_str,
        session_type,
        gch_installed,
    })
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
