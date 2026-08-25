mod commands;
mod db;
mod error;
mod models;
mod services;
mod time_math;
mod validate;

use commands::notify_state_changed;
use commands::timer::{pause_task_tx, start_task_tx};
use rusqlite::OptionalExtension;
use services::scheduler::SchedulerNotify;
use std::sync::Arc;
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};
use tokio::sync::Notify;

/// Show, unminimize and focus the main window.
fn show_main_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// "Start last task": resumes the most recently paused task, or failing that, the most
/// recently created pending task. A no-op if a task is already active.
fn tray_start_last(app: &tauri::AppHandle) {
    let db = app.state::<db::Db>();
    let Ok(mut conn) = db.lock() else { return };

    let already_active: bool = conn
        .query_row(
            "SELECT 1 FROM tasks WHERE status = 'active' LIMIT 1",
            [],
            |_| Ok(()),
        )
        .optional()
        .unwrap_or(None)
        .is_some();
    if already_active {
        return;
    }

    let candidate: Option<i64> = conn
        .query_row(
            "SELECT id FROM tasks WHERE status = 'paused' ORDER BY started_at DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()
        .unwrap_or(None)
        .or_else(|| {
            conn.query_row(
                "SELECT id FROM tasks WHERE status = 'pending' ORDER BY created_at DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .optional()
            .unwrap_or(None)
        });

    if let Some(id) = candidate {
        if start_task_tx(&mut conn, id).is_ok() {
            drop(conn);
            notify_state_changed(app);
        }
    }
}

/// "Pause current": pauses whichever task is active, if any.
fn tray_pause_current(app: &tauri::AppHandle) {
    let db = app.state::<db::Db>();
    let Ok(mut conn) = db.lock() else { return };

    let active: Option<i64> = conn
        .query_row("SELECT id FROM tasks WHERE status = 'active' LIMIT 1", [], |r| {
            r.get(0)
        })
        .optional()
        .unwrap_or(None);

    if let Some(id) = active {
        if pause_task_tx(&mut conn, id).is_ok() {
            drop(conn);
            notify_state_changed(app);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    // Desktop-only plugins. `single-instance` MUST be the first plugin registered so a
    // second launch focuses the existing window instead of spawning another process.
    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                show_main_window(app);
            }))
            .plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                Some(vec!["--minimized"]),
            ));
    }

    builder
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            commands::tasks::list_tasks,
            commands::tasks::create_task,
            commands::tasks::update_task,
            commands::tasks::delete_task,
            commands::timer::start_task,
            commands::timer::pause_task,
            commands::timer::complete_task,
            commands::notes::add_note,
            commands::summary::get_day_summary,
            commands::summary::get_day_review,
            commands::summary::list_history_dates,
            commands::summary::get_pending_daily_report,
            commands::window::reveal_window,
        ])
        .setup(|app| {
            #[cfg(windows)]
            services::toast::register_aumid();

            // `init()` above only registers the plugin -- it does not itself register a login
            // item. Enabling here (idempotent; a harmless no-op if already enabled) is what
            // actually makes the app launch on login, silently, per Architecture §9.1.
            #[cfg(desktop)]
            {
                use tauri_plugin_autostart::ManagerExt;
                let _ = app.autolaunch().enable();
            }

            let handle = app.handle().clone();
            let conn = db::open(&handle).expect("failed to open database");
            app.manage(std::sync::Mutex::new(conn));

            let reminder_notify = Arc::new(Notify::new());
            app.manage(SchedulerNotify(reminder_notify.clone()));
            services::scheduler::spawn(handle.clone(), reminder_notify);

            // Safety flush: while a task is running, persist its elapsed-so-far every 45s so
            // a hard crash loses at most that interval (Architecture §10).
            let flush_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                let mut ticker = tokio::time::interval(Duration::from_secs(45));
                ticker.tick().await; // skip the immediate first tick
                loop {
                    ticker.tick().await;
                    let db_state = flush_handle.state::<db::Db>();
                    if let Ok(conn) = db_state.lock() {
                        let _ = db::flush_open_session(&conn);
                    };
                }
            });

            // The window starts hidden (see tauri.conf.json) so an autostart-minimized launch
            // never flashes on screen; show it now unless we were launched by Windows login
            // autostart, which passes --minimized (Architecture §9.1).
            let launched_minimized = std::env::args().any(|a| a == "--minimized");
            if let Some(window) = app.get_webview_window("main") {
                if !launched_minimized {
                    let _ = window.show();
                }
                // Closing the window minimizes to tray instead of quitting the app
                // (Architecture §8.3) -- the process (and its tray icon) stays alive.
                let window_to_hide = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_to_hide.hide();
                    }
                });
            }

            // System tray with the quick-actions menu (Architecture §8.2 #9).
            let open_i = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            let start_last_i =
                MenuItem::with_id(app, "start_last", "Start last task", true, None::<&str>)?;
            let pause_i =
                MenuItem::with_id(app, "pause_current", "Pause current", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_i, &start_last_i, &pause_i, &quit_i])?;

            TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Unfazed Trace")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show_main_window(app),
                    "start_last" => tray_start_last(app),
                    "pause_current" => tray_pause_current(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main_window(tray.app_handle());
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
