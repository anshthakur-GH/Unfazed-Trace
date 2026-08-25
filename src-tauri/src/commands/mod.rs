pub mod notes;
pub mod summary;
pub mod tasks;
pub mod timer;
pub mod window;

/// Emitted after any command that changes task/session state, so the frontend list can
/// refetch reactively instead of polling.
pub(crate) fn notify_state_changed(app: &tauri::AppHandle) {
    use tauri::Emitter;
    let _ = app.emit("state-changed", ());
}
