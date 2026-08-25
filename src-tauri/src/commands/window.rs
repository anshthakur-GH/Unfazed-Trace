use std::sync::Mutex;
use tauri::{AppHandle, LogicalSize, Manager, PhysicalPosition, PhysicalSize, State};

/// Remembers the main window's normal bounds while it's collapsed into the mini floating timer,
/// so exiting mini mode restores exactly where and how big it was.
#[derive(Default)]
pub struct MiniState(pub Mutex<Option<(PhysicalSize<u32>, PhysicalPosition<i32>)>>);

const MINI_W: f64 = 260.0;
const MINI_H: f64 = 132.0;
const NORMAL_MIN_W: f64 = 360.0;
const NORMAL_MIN_H: f64 = 480.0;

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

/// Collapse the main window into a small, always-on-top floating timer parked in the
/// bottom-right corner. Triggered by the frontend after ~10s of no interaction while a task is
/// actively running, so the running time stays visible while you work in other apps.
#[tauri::command]
pub fn enter_mini_mode(app: AppHandle, state: State<MiniState>) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    // Remember where we were so exit restores it exactly.
    if let (Ok(size), Ok(pos)) = (window.outer_size(), window.outer_position()) {
        *state.0.lock().unwrap() = Some((size, pos));
    }

    // The main window's configured minimum is larger than the widget; relax it first.
    let _ = window.set_min_size(Some(LogicalSize::new(200.0, 110.0)));
    let _ = window.set_resizable(false);
    let _ = window.set_always_on_top(true);
    let _ = window.set_size(LogicalSize::new(MINI_W, MINI_H));

    if let Ok(Some(monitor)) = window.current_monitor() {
        let scale = monitor.scale_factor();
        let mon_pos = monitor.position();
        let mon_size = monitor.size();
        let mini_w = (MINI_W * scale) as i32;
        let mini_h = (MINI_H * scale) as i32;
        let margin = (16.0 * scale) as i32;
        let taskbar = (56.0 * scale) as i32; // rough clearance for the Windows taskbar
        let x = mon_pos.x + mon_size.width as i32 - mini_w - margin;
        let y = mon_pos.y + mon_size.height as i32 - mini_h - taskbar;
        let _ = window.set_position(PhysicalPosition::new(x, y));
    }

    let _ = window.show();
}

/// Restore the main window from mini mode back to its previous size, position, and normal
/// (not-always-on-top, resizable) behavior.
#[tauri::command]
pub fn exit_mini_mode(app: AppHandle, state: State<MiniState>) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    let _ = window.set_always_on_top(false);
    let _ = window.set_min_size(Some(LogicalSize::new(NORMAL_MIN_W, NORMAL_MIN_H)));
    let _ = window.set_resizable(true);

    let prev = state.0.lock().unwrap().take();
    if let Some((size, pos)) = prev {
        let _ = window.set_size(size);
        let _ = window.set_position(pos);
    } else {
        let _ = window.set_size(LogicalSize::new(420.0, 640.0));
        let _ = window.center();
    }
    let _ = window.show();
    let _ = window.set_focus();
}
