use tauri::State;

use crate::storage::Clip;
use crate::AppState;

/// Fetch the most recent clips from the database.
#[tauri::command]
pub async fn get_clips(state: State<'_, AppState>) -> Result<Vec<Clip>, String> {
    state
        .db
        .get_recent_clips(100)
        .await
        .map_err(|e| e.to_string())
}

/// Copy a clip's content back to the system clipboard.
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

/// Soft-delete a clip by ID.
#[tauri::command]
pub async fn delete_clip(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    state
        .db
        .soft_delete_clip(id)
        .await
        .map_err(|e| e.to_string())
}
