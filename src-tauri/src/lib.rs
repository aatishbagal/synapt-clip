mod clipboard;
mod commands;
mod crash;
mod dsa;
mod hotkey;
mod platform;
mod search;
mod share;
mod storage;
mod synapt;
mod tray;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

use tauri::{Emitter, Manager};
use tokio::sync::Mutex;
use tracing_subscriber::EnvFilter;

use crate::dsa::{should_group, ClipGroupManager, PersistentList, SkipList, VersionHistory};
use crate::search::classifier::ClipClassifier;
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
    pub clip_index: Arc<Mutex<SkipList>>,
    pub classifier: Arc<ClipClassifier>,
    pub clip_count: Arc<AtomicUsize>,
    pub bridge_state: crate::synapt::bridge::SharedBridgeState,
}

pub fn clip_score(created_at: &str, pinned: bool) -> f64 {
    let seconds = chrono::NaiveDateTime::parse_from_str(created_at, "%Y-%m-%d %H:%M:%S")
        .map(|dt| {
            let now = chrono::Utc::now().naive_utc();
            (now - dt).num_seconds().max(0) as f64
        })
        .unwrap_or(0.0);
    1.0 / (1.0 + seconds) + if pinned { 0.5 } else { 0.0 }
}

/// Read the current bridge active/peer-count snapshot, tolerating lock poison.
fn bridge_snapshot(state: &crate::synapt::bridge::SharedBridgeState) -> (bool, usize) {
    let read = match state.read() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    (read.active, read.peers.len())
}

fn set_tray_warning(app: &tauri::AppHandle) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some("SynaptClip — setup required"));
    }
}

pub fn log_file_path() -> Option<std::path::PathBuf> {
    dirs::data_dir().map(|d| d.join("synaptclip").join("synaptclip.log"))
}

struct DualWriter {
    file: Option<Arc<std::sync::Mutex<std::fs::File>>>,
}

impl std::io::Write for DualWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Some(ref f) = self.file {
            if let Ok(mut guard) = f.lock() {
                let _ = guard.write_all(buf);
            }
        }
        std::io::stderr().write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(ref f) = self.file {
            if let Ok(mut guard) = f.lock() {
                let _ = guard.flush();
            }
        }
        std::io::stderr().flush()
    }
}

impl<'a> tracing_subscriber::fmt::writer::MakeWriter<'a> for DualWriter {
    type Writer = DualWriter;
    fn make_writer(&'a self) -> Self::Writer {
        DualWriter {
            file: self.file.as_ref().map(Arc::clone),
        }
    }
}

fn setup_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let file_arc: Option<Arc<std::sync::Mutex<std::fs::File>>> =
        log_file_path().and_then(|path| {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(meta) = std::fs::metadata(&path) {
                if meta.len() > 5 * 1024 * 1024 {
                    let _ = std::fs::rename(&path, path.with_extension("log.old"));
                }
            }
            match std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
            {
                Ok(file) => Some(Arc::new(std::sync::Mutex::new(file))),
                Err(e) => {
                    eprintln!("Failed to open log file: {e}");
                    None
                }
            }
        });

    let writer = DualWriter { file: file_arc };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(writer)
        .with_ansi(false)
        .init();
}

pub fn run() {
    // Installed before anything else so a panic during startup is still logged.
    crash::install_panic_hook();

    setup_logging();

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "SynaptClip starting");

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

    let (db, initial_clips, startup_hotkey) = rt.block_on(async {
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
        let hotkey_val = db
            .get_setting("hotkey")
            .await
            .unwrap_or(None)
            .unwrap_or_else(|| hotkey::DEFAULT_HOTKEY.to_string());
        (db, clips, hotkey_val)
    });

    let db = Arc::new(db);

    let mut engine = SearchEngine::new();
    let mut depq: Depq<(String, i64)> = Depq::new();
    let mut group_manager = ClipGroupManager::new();
    let mut skip_list = SkipList::new();
    for c in &initial_clips {
        engine.index_clip(c.id, &c.content);
        if !c.pinned {
            depq.push((c.created_at.clone(), c.id));
        }
        group_manager.add_clip(c.id);
        skip_list.insert(clip_score(&c.created_at, c.pinned), c.id);
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
    let clip_index = Arc::new(Mutex::new(skip_list));
    let classifier = Arc::new(ClipClassifier::new());
    let clip_count = Arc::new(AtomicUsize::new(initial_clips.len()));
    let bridge_state: crate::synapt::bridge::SharedBridgeState =
        Arc::new(RwLock::new(crate::synapt::bridge::BridgeState::inactive()));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState {
            db: db.clone(),
            search_engine: search_engine.clone(),
            expiry_depq: expiry_depq.clone(),
            clip_history: clip_history.clone(),
            group_manager: group_manager.clone(),
            clip_index: clip_index.clone(),
            classifier: classifier.clone(),
            clip_count: clip_count.clone(),
            bridge_state: bridge_state.clone(),
        })
        .setup(move |app| {
            // macOS: hide the Dock icon (tray-only app).
            #[cfg(target_os = "macos")]
            platform::macos::setup();

            // Shared system tray: become the host (owns the single icon) or attach
            // as a client to an already-running Synapt/SynaptClip host.
            share::start(app);

            // Background Synapt bridge: detect Synapt and track its peer list.
            tauri::async_runtime::spawn(crate::synapt::bridge::start(
                Arc::clone(&bridge_state),
                clip_count.clone(),
                app.handle().clone(),
            ));

            // Incoming-clip listener on port 57322: always runs, even when Synapt
            // is not detected, since Synapt may start later.
            tauri::async_runtime::spawn(crate::synapt::listener::start(
                db.clone(),
                app.handle().clone(),
            ));

            // Automatic update check. Delayed so it does not compete with the
            // rest of startup for the first seconds.
            let db_for_updates = db.clone();
            let handle_updates = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(15)).await;
                commands::run_auto_update_check(&handle_updates, &db_for_updates).await;
            });

            if let Some(window) = app.get_webview_window("main") {
                let w = window.clone();
                window.on_window_event(move |event| match event {
                    tauri::WindowEvent::Focused(false) => {
                        let _ = w.hide();
                    }
                    // The panel is undecorated so it has no close button, but a
                    // close can still be requested (Cmd+W, or the OS tearing the
                    // window down). Hide it instead: quitting is tray-only.
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        let _ = w.hide();
                    }
                    _ => {}
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

            match hotkey::register_hotkey(app.handle(), &startup_hotkey) {
                Ok(()) => tracing::info!("Hotkey registered: {startup_hotkey}"),
                Err(e) => tracing::warn!("Hotkey registration failed: {e}"),
            }

            let backend = platform::detect_backend();
            tracing::info!("Detected clipboard backend: {:?}", backend);

            // Surface setup-required backends to the frontend up front.
            match backend {
                platform::ClipboardBackend::GchNotEnabled => {
                    let _ = app.handle().emit(
                        "backend-status",
                        serde_json::json!({
                            "status": "gch_not_enabled",
                            "message": "Enable Clipboard History in GNOME Extensions",
                        }),
                    );
                    set_tray_warning(app.handle());
                }
                platform::ClipboardBackend::GchNotInstalled => {
                    let _ = app.handle().emit(
                        "backend-status",
                        serde_json::json!({
                            "status": "gch_not_installed",
                            "message": "Install the Clipboard History GNOME extension to use SynaptClip on GNOME Wayland",
                        }),
                    );
                    set_tray_warning(app.handle());
                }
                platform::ClipboardBackend::Unsupported => {
                    let _ = app.handle().emit(
                        "backend-status",
                        serde_json::json!({
                            "status": "unsupported",
                            "message": "No supported clipboard backend detected for this session",
                        }),
                    );
                    set_tray_warning(app.handle());
                }
                _ => {}
            }

            let watcher = clipboard::create_watcher(&backend);
            let (tx, mut rx) = tokio::sync::mpsc::channel::<clipboard::NewClip>(32);

            let app_handle = app.handle().clone();
            let db_writer = db.clone();
            let engine_writer = search_engine.clone();
            let depq_writer = expiry_depq.clone();
            let history_writer = clip_history.clone();
            let group_writer = group_manager.clone();
            let index_writer = clip_index.clone();
            let classifier_ref = classifier.clone();
            let count_writer = clip_count.clone();
            let bridge_reader = bridge_state.clone();

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
                                let score = clip_score(&clip.created_at, clip.pinned);
                                index_writer.lock().await.insert(score, clip.id);
                            }

                            {
                                let category = classifier_ref.classify(&clip.content);
                                if !matches!(category, crate::search::classifier::ClipCategory::PlainText) {
                                    if let Err(e) = db_writer.assign_category(clip.id, Some(category.label())).await {
                                        tracing::warn!("Failed to auto-assign category: {e}");
                                    }
                                }
                            }

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
                            let (bridge_active, peer_count) = bridge_snapshot(&bridge_reader);
                            crate::tray::update_tray_tooltip(
                                &app_handle,
                                new_count,
                                bridge_active,
                                peer_count,
                            );

                            let mut depq_guard = depq_writer.lock().await;
                            while depq_guard.len() > HISTORY_LIMIT {
                                if let Some((_, evict_id)) = depq_guard.pop_min() {
                                    drop(depq_guard);
                                    if let Err(e) = db_writer.hard_delete_clip(evict_id).await {
                                        tracing::warn!("Failed to evict clip {evict_id}: {e}");
                                    } else {
                                        engine_writer.lock().await.remove_clip(evict_id, "");
                                        group_writer.lock().await.remove_clip(evict_id);
                                        index_writer.lock().await.remove(evict_id);
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
            commands::get_hotkey_status,
            commands::pause_hotkey,
            commands::resume_hotkey,
            clipboard::gch::get_gch_status,
            platform::detect::get_backend_status,
            commands::open_settings,
            commands::close_settings,
            commands::get_ranked_clips,
            commands::get_auto_categories,
            commands::set_autostart,
            commands::get_autostart,
            commands::get_log_path,
            commands::get_crash_log_path,
            commands::check_for_update,
            commands::install_update,
            commands::get_app_version,
            commands::get_bridge_state,
            commands::refresh_bridge_peers,
            commands::send_clip_to_peer,
            commands::send_latest_clip_to_peer,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app, event| {
            // Suppress Cmd+Q, the app menu's Quit and the Dock's Quit. `code` is
            // None only when the OS asked on the user's behalf; the tray menu
            // quits via AppHandle::exit, which arrives with Some(code) and is
            // allowed through so the confirmed quit still works.
            if let tauri::RunEvent::ExitRequested { code: None, api, .. } = event {
                api.prevent_exit();
                tracing::debug!("ignored an OS quit request; quit from the tray menu instead");
            }
        });
}
