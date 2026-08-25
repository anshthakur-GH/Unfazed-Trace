use std::sync::Mutex;
use tauri::{
    window::Color, AppHandle, LogicalSize, Manager, PhysicalPosition, PhysicalSize, State,
    WebviewWindow,
};

/// Remembers the main window's normal bounds while it's collapsed into the mini floating timer,
/// so exiting mini mode restores exactly where and how big it was. `Some` also doubles as the
/// "currently mini" flag, guarding against re-entering (and clobbering the remembered bounds)
/// if triggered twice in a row.
#[derive(Default)]
pub struct MiniState(pub Mutex<Option<(PhysicalSize<u32>, PhysicalPosition<i32>)>>);

const MINI_W: f64 = 150.0;
const MINI_H: f64 = 52.0;
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

/// Collapse the main window into a small, always-on-top floating timer parked in the top-right
/// corner. Called both from the frontend (10s idle while a task runs, or the OS minimize button
/// -- see `lib.rs`'s Resized handler) so the running time stays visible while you work elsewhere.
/// A no-op if already mini (see [`MiniState`]).
pub(crate) fn apply_mini_mode(window: &WebviewWindow, state: &MiniState) {
    let mut guard = state.0.lock().unwrap();
    if guard.is_some() {
        return; // already mini
    }

    // Remember where we were so exit restores it exactly.
    if let (Ok(size), Ok(pos)) = (window.outer_size(), window.outer_position()) {
        *guard = Some((size, pos));
    }
    drop(guard);

    // The main window's configured minimum is larger than the widget; relax it first.
    let _ = window.set_min_size(Some(LogicalSize::new(110.0, 42.0)));
    let _ = window.set_resizable(false);
    // Frameless so it reads as a floating pill.
    let _ = window.set_decorations(false);
    // Windows draws a 1px white border (and its own corner rounding) around undecorated windows
    // whenever the window shadow is enabled -- that's the "rectangular ghost border" fighting
    // our own CSS rounding. Disabling it leaves only our rounded div, with true transparency
    // (below) filling in everywhere outside it.
    let _ = window.set_shadow(false);
    // Float above everything else and stay out of the taskbar, like an on-screen clock overlay.
    let _ = window.set_always_on_top(true);
    let _ = window.set_skip_taskbar(true);
    let _ = window.set_size(LogicalSize::new(MINI_W, MINI_H));
    // WebView2 paints an opaque white backdrop by default; without this, anti-aliasing at the
    // CSS-rounded corners blends against that white and leaves a faint white fringe even though
    // the OS window itself is transparent there. Fully transparent removes it.
    let _ = window.set_background_color(Some(Color(0, 0, 0, 0)));

    // Park it in the top-right corner as an always-visible on-screen clock overlay.
    if let Ok(Some(monitor)) = window.current_monitor() {
        let scale = monitor.scale_factor();
        let mon_pos = monitor.position();
        let mon_size = monitor.size();
        let mini_w = (MINI_W * scale) as i32;
        let margin = (16.0 * scale) as i32;
        let x = mon_pos.x + mon_size.width as i32 - mini_w - margin;
        let y = mon_pos.y + margin;
        let _ = window.set_position(PhysicalPosition::new(x, y));
    }

    let _ = window.show();
}

#[tauri::command]
pub fn enter_mini_mode(app: AppHandle, state: State<MiniState>) {
    if let Some(window) = app.get_webview_window("main") {
        apply_mini_mode(&window, &state);
    }
}

/// Restore the main window from mini mode back to its previous size, position, and normal
/// (not-always-on-top, resizable) behavior.
#[tauri::command]
pub fn exit_mini_mode(app: AppHandle, state: State<MiniState>) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    let _ = window.set_always_on_top(false);
    let _ = window.set_skip_taskbar(false);
    let _ = window.set_decorations(true);
    let _ = window.set_shadow(true);
    let _ = window.set_background_color(None);
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
