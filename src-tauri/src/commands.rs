use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::storage::Clip;
use crate::AppState;

#[tauri::command]
pub async fn get_clips(state: State<'_, AppState>) -> Result<Vec<Clip>, String> {
    state
        .db
        .get_recent_clips(500)
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
