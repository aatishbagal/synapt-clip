mod clipboard;
mod commands;
mod dsa;
mod platform;
mod search;
mod storage;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};
use tokio::sync::Mutex;
use tracing_subscriber::EnvFilter;

use crate::dsa::{should_group, ClipGroupManager, PersistentList, VersionHistory};
use crate::search::depq::Depq;
use crate::search::engine::SearchEngine;
use crate::storage::Clip;

const HISTORY_LIMIT: usize = 500;
const TRAY_ID: &str = "main";

pub struct AppState {
    pub db: Arc<storage::Db>,
    pub search_engine: Arc<Mutex<SearchEngine>>,
    pub expiry_depq: Arc<Mutex<Depq<(String, i64)>>>,
    pub clip_history: Arc<Mutex<VersionHistory<Clip>>>,
    pub group_manager: Arc<Mutex<ClipGroupManager>>,
    pub clip_count: Arc<AtomicUsize>,
}

fn update_tray_tooltip(app: &tauri::AppHandle, count: usize) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(format!("SynaptClip — {count} clips")));
    }
}

fn set_tray_warning(app: &tauri::AppHandle) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some("SynaptClip — setup required"));
    }
}

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting SynaptClip v0.3.0");

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

    let (db, initial_clips) = rt.block_on(async {
        let app_data_dir = dirs::data_dir()
            .expect("failed to resolve app data directory")
            .join("dev.synapt.clip");
        let db = storage::Db::new(&app_data_dir)
            .await
            .expect("failed to initialize database");
        let clips = db
            .get_recent_clips(5000)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to load initial clips: {e}");
                Vec::new()
            });
        (db, clips)
    });

    let db = Arc::new(db);

    let mut engine = SearchEngine::new();
    let mut depq: Depq<(String, i64)> = Depq::new();
    let mut group_manager = ClipGroupManager::new();
    for c in &initial_clips {
        engine.index_clip(c.id, &c.content);
        if !c.pinned {
            depq.push((c.created_at.clone(), c.id));
        }
        group_manager.add_clip(c.id);
    }
    for i in 0..initial_clips.len() {
        for j in (i + 1)..initial_clips.len() {
            if should_group(&initial_clips[i], &initial_clips[j]) {
                group_manager.group_clips(initial_clips[i].id, initial_clips[j].id);
            }
        }
    }

    let mut initial_version: PersistentList<Clip> = PersistentList::new();
    for c in initial_clips.iter().rev() {
        initial_version = initial_version.prepend(c.clone());
    }

    let search_engine = Arc::new(Mutex::new(engine));
    let expiry_depq = Arc::new(Mutex::new(depq));
    let clip_history = Arc::new(Mutex::new(VersionHistory::new(initial_version)));
    let group_manager = Arc::new(Mutex::new(group_manager));
    let clip_count = Arc::new(AtomicUsize::new(initial_clips.len()));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            db: db.clone(),
            search_engine: search_engine.clone(),
            expiry_depq: expiry_depq.clone(),
            clip_history: clip_history.clone(),
            group_manager: group_manager.clone(),
            clip_count: clip_count.clone(),
        })
        .setup(move |app| {
            let show = MenuItem::with_id(app, "show", "Show SynaptClip", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            TrayIconBuilder::with_id(TRAY_ID)
                .tooltip(format!(
                    "SynaptClip — {} clips",
                    clip_count.load(Ordering::Relaxed)
                ))
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.center();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let visible = window.is_visible().unwrap_or(false);
                            if visible {
                                let _ = window.hide();
                            } else {
                                let _ = window.center();
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            if let Some(window) = app.get_webview_window("main") {
                let w = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Focused(false) = event {
                        let _ = w.hide();
                    }
                });
            }

            if let Some(settings_window) = app.get_webview_window("settings") {
                let s = settings_window.clone();
                settings_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = s.hide();
                    }
                });
            }

            let backend = platform::detect_backend();
            tracing::info!("Detected clipboard backend: {:?}", backend);

            let watcher = clipboard::create_watcher(&backend);
            let (tx, mut rx) = tokio::sync::mpsc::channel::<clipboard::NewClip>(32);

            let app_handle = app.handle().clone();
            let db_writer = db.clone();
            let engine_writer = search_engine.clone();
            let depq_writer = expiry_depq.clone();
            let history_writer = clip_history.clone();
            let group_writer = group_manager.clone();
            let count_writer = clip_count.clone();

            let setup_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match watcher.watch(tx).await {
                    Ok(()) => {}
                    Err(clipboard::WatcherError::NotFound(msg)) => {
                        tracing::warn!("Clipboard watcher setup required: {msg}");
                        let _ = setup_handle.emit(
                            "watcher:setup_required",
                            serde_json::json!({ "message": msg }),
                        );
                        set_tray_warning(&setup_handle);
                    }
                    Err(e) => {
                        tracing::error!("Clipboard watcher error: {e}");
                        set_tray_warning(&setup_handle);
                    }
                }
            });

            tauri::async_runtime::spawn(async move {
                while let Some(new_clip) = rx.recv().await {
                    let is_probable_dup = engine_writer
                        .lock()
                        .await
                        .is_probable_duplicate(&new_clip.content);

                    if is_probable_dup {
                        match db_writer.get_last_clip_content().await {
                            Ok(Some(ref last)) if last == &new_clip.content => continue,
                            Err(e) => {
                                tracing::warn!("Failed to check last clip: {e}");
                            }
                            _ => {}
                        }
                    }

                    match db_writer
                        .insert_clip(
                            &new_clip.content,
                            &new_clip.content_type,
                            new_clip.source_app.as_deref(),
                        )
                        .await
                    {
                        Ok(clip) => {
                            engine_writer.lock().await.index_clip(clip.id, &clip.content);
                            depq_writer
                                .lock()
                                .await
                                .push((clip.created_at.clone(), clip.id));

                            {
                                let mut history = history_writer.lock().await;
                                let new_version = history.current().prepend(clip.clone());
                                history.push(new_version);
                            }

                            let recent = match db_writer.get_recent_clips(21).await {
                                Ok(r) => r,
                                Err(e) => {
                                    tracing::warn!("Failed to fetch recent for grouping: {e}");
                                    Vec::new()
                                }
                            };
                            {
                                let mut mgr = group_writer.lock().await;
                                mgr.add_clip(clip.id);
                                for other in recent.iter() {
                                    if other.id == clip.id {
                                        continue;
                                    }
                                    if should_group(&clip, other) {
                                        mgr.add_clip(other.id);
                                        mgr.group_clips(clip.id, other.id);
                                    }
                                }
                            }

                            if let Err(e) = app_handle.emit("clip:new", &clip) {
                                tracing::warn!("Failed to emit clip:new event: {e}");
                            }

                            let new_count = count_writer.fetch_add(1, Ordering::Relaxed) + 1;
                            update_tray_tooltip(&app_handle, new_count);

                            let mut depq_guard = depq_writer.lock().await;
                            while depq_guard.len() > HISTORY_LIMIT {
                                if let Some((_, evict_id)) = depq_guard.pop_min() {
                                    drop(depq_guard);
                                    if let Err(e) = db_writer.hard_delete_clip(evict_id).await {
                                        tracing::warn!("Failed to evict clip {evict_id}: {e}");
                                    } else {
                                        engine_writer.lock().await.remove_clip(evict_id, "");
                                        group_writer.lock().await.remove_clip(evict_id);
                                    }
                                    depq_guard = depq_writer.lock().await;
                                } else {
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to insert clip: {e}");
                        }
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_clips,
            commands::get_pinned_clips,
            commands::copy_clip,
            commands::delete_clip,
            commands::clear_all_clips,
            commands::search_clips,
            commands::undo_delete,
            commands::get_clip_groups,
            commands::get_group_for_clip,
            commands::toggle_pin,
            commands::assign_category,
            commands::get_categories,
            commands::delete_category,
            commands::bulk_delete,
            commands::clear_history,
            commands::get_settings,
            commands::set_setting,
            commands::get_platform_info,
            commands::open_settings,
            commands::close_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
