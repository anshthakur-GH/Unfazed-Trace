use tauri::{AppHandle, Manager};

/// Brings the main window to the foreground. Used when the app proactively surfaces something
/// the user should see now -- e.g. the once-a-day catch-up report on a fresh login, where the
/// window was started hidden (`--minimized`).
#[tauri::command]
pub fn reveal_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
