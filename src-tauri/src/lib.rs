mod commands;
mod db;
mod error;
mod models;
mod services;
mod time_math;
mod validate;

use services::scheduler::SchedulerNotify;
use std::sync::Arc;
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
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
        ])
        .setup(|app| {
            #[cfg(windows)]
            services::toast::register_aumid();

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

            // System tray with a minimal quick-actions menu (expanded in Phase 6).
            let open_i = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_i, &quit_i])?;

            TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Unfazed Trace")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show_main_window(app),
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
