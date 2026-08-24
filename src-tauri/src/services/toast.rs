use crate::commands::notify_state_changed;
use crate::commands::timer::start_task_tx;
use crate::db::Db;
use chrono::{Duration, Utc};
use tauri::{AppHandle, Manager};
use tauri_winrt_notification::{Duration as ToastDuration, Scenario, Toast};

/// Must match `identifier` in tauri.conf.json — used as the toast's Application User Model ID.
pub const APP_ID: &str = "com.unfazed.trace";
const APP_NAME: &str = "Unfazed Trace";

/// Registers this app's AUMID in the registry so Windows toasts show the correct name/icon
/// for an unpackaged (non-MSIX) desktop app, per the pattern in `tauri-winrt-notification`'s
/// own `unpackaged_app` example. Idempotent — cheap to call on every launch.
pub fn register_aumid() {
    use windows_registry::CURRENT_USER;
    let Ok(key) = CURRENT_USER.create(format!(r"SOFTWARE\Classes\AppUserModelId\{APP_ID}")) else {
        return;
    };
    let _ = key.set_string("DisplayName", APP_NAME);
    let _ = key.set_string("IconBackgroundColor", "0");
    // Best-effort: point at the running exe's own icon. Cosmetic only — if this doesn't
    // resolve, Windows just falls back to a generic icon rather than failing the toast.
    if let Ok(exe) = std::env::current_exe() {
        let _ = key.set_string("IconUri", &format!("{},0", exe.display()));
    }
}

/// Shows the "task is due" reminder toast with Start now / Snooze 10 min / Dismiss actions.
/// Button clicks are a native OS callback (not IPC), so they call directly into the same
/// transaction helpers the Tauri commands use, then emit `state-changed` for the frontend.
pub fn show_reminder(app: &AppHandle, task_id: i64, title: &str) {
    let app_for_activation = app.clone();
    let result = Toast::new(APP_ID)
        .title("Task due")
        .text1(title)
        .scenario(Scenario::Reminder)
        .duration(ToastDuration::Long)
        .add_button("Start now", &format!("start:{task_id}"))
        .add_button("Snooze 10 min", &format!("snooze:{task_id}"))
        .add_button("Dismiss", "dismiss")
        .on_activated(move |action| {
            handle_activation(&app_for_activation, action);
            Ok(())
        })
        .show();

    if let Err(err) = result {
        eprintln!("failed to show reminder toast: {err}");
    }
}

fn handle_activation(app: &AppHandle, action: Option<String>) {
    let Some(action) = action else {
        // Toast body (not a button) was clicked -- bring the window forward.
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_focus();
        }
        return;
    };

    let db = app.state::<Db>();
    if let Some(id_str) = action.strip_prefix("start:") {
        if let Ok(id) = id_str.parse::<i64>() {
            if let Ok(mut conn) = db.lock() {
                if let Err(err) = start_task_tx(&mut conn, id) {
                    eprintln!("toast 'Start now' action failed: {err}");
                }
            }
            notify_state_changed(app);
        }
    } else if let Some(id_str) = action.strip_prefix("snooze:") {
        if let Ok(id) = id_str.parse::<i64>() {
            let new_time = (Utc::now() + Duration::minutes(10)).to_rfc3339();
            if let Ok(conn) = db.lock() {
                let _ = conn.execute(
                    "UPDATE tasks SET remind_at = ?1, reminder_fired = 0 WHERE id = ?2",
                    rusqlite::params![new_time, id],
                );
            }
            notify_state_changed(app);
            crate::services::scheduler::wake(app);
        }
    }
    // "dismiss" -- reminder_fired is already 1 from when the toast fired; nothing else to do.
}
