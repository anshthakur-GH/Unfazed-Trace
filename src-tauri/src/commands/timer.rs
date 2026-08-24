use super::notify_state_changed;
use super::tasks::fetch_task;
use crate::db::Db;
use crate::error::AppError;
use crate::models::{ReviewNotes, Task};
use crate::time_math::seconds_between;
use crate::validate;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use tauri::{AppHandle, State};

/// Closes whichever `time_sessions` row is currently open and folds its duration into the
/// owning task's `total_seconds`. Returns that task's id, if one was open.
///
/// Safe to call unconditionally because the app maintains a strict invariant: **at most one
/// task is ever `active`, and at most one `time_sessions` row is ever open, and it always
/// belongs to that active task.** Every call site below first checks task status so it never
/// closes a session it doesn't intend to.
fn close_open_session(tx: &Transaction, now: &str) -> Result<Option<i64>, AppError> {
    let open: Option<(i64, i64, String)> = tx
        .query_row(
            "SELECT id, task_id, started_at FROM time_sessions WHERE ended_at IS NULL",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;

    if let Some((session_id, task_id, started_at)) = open {
        let secs = seconds_between(&started_at, now)
            .map_err(|_| AppError::new("Corrupt session timestamp."))?;
        tx.execute(
            "UPDATE time_sessions SET ended_at = ?1, seconds = ?2 WHERE id = ?3",
            params![now, secs, session_id],
        )?;
        tx.execute(
            "UPDATE tasks SET total_seconds = total_seconds + ?1 WHERE id = ?2",
            params![secs, task_id],
        )?;
        Ok(Some(task_id))
    } else {
        Ok(None)
    }
}

/// Transaction-only body of "start a task". Shared by the `start_task` IPC command and the
/// toast's "Start now" button, which acts on a native OS callback with no IPC involved.
pub(crate) fn start_task_tx(conn: &mut Connection, id: i64) -> Result<(), AppError> {
    let now = Utc::now().to_rfc3339();
    let tx = conn.transaction()?;

    let status: String = tx
        .query_row("SELECT status FROM tasks WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .map_err(|_| AppError::new("Task not found."))?;
    if status == "done" {
        return Err(AppError::new("Cannot start a task that is already done."));
    }

    if status != "active" {
        // Auto-pause whatever else is running. `id` is guaranteed not to own the open
        // session here, since we just confirmed `id`'s own status isn't "active".
        if let Some(other_task_id) = close_open_session(&tx, &now)? {
            tx.execute(
                "UPDATE tasks SET status = 'paused' WHERE id = ?1",
                [other_task_id],
            )?;
        }
        tx.execute(
            "INSERT INTO time_sessions (task_id, started_at) VALUES (?1, ?2)",
            params![id, now],
        )?;
        tx.execute(
            "UPDATE tasks SET status = 'active', started_at = COALESCE(started_at, ?2) WHERE id = ?1",
            params![id, now],
        )?;
    }

    tx.commit()?;
    Ok(())
}

#[tauri::command]
pub fn start_task(app: AppHandle, db: State<Db>, id: i64) -> Result<Task, AppError> {
    let mut conn = db.lock().map_err(|_| AppError::new("Internal lock error."))?;
    start_task_tx(&mut conn, id)?;
    let result = fetch_task(&conn, id)?;
    drop(conn);
    notify_state_changed(&app);
    Ok(result)
}

/// Transaction-only body of "pause the active task". Shared by the `pause_task` IPC command
/// and the tray's "Pause current" menu item, which has no IPC/window involved.
pub(crate) fn pause_task_tx(conn: &mut Connection, id: i64) -> Result<(), AppError> {
    let now = Utc::now().to_rfc3339();
    let tx = conn.transaction()?;

    let status: String = tx
        .query_row("SELECT status FROM tasks WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .map_err(|_| AppError::new("Task not found."))?;
    if status != "active" {
        return Err(AppError::new("Task is not active."));
    }

    // Safe: `id` is confirmed active, so it is the sole owner of the open session.
    close_open_session(&tx, &now)?;
    tx.execute("UPDATE tasks SET status = 'paused' WHERE id = ?1", [id])?;

    tx.commit()?;
    Ok(())
}

#[tauri::command]
pub fn pause_task(app: AppHandle, db: State<Db>, id: i64) -> Result<Task, AppError> {
    let mut conn = db.lock().map_err(|_| AppError::new("Internal lock error."))?;
    pause_task_tx(&mut conn, id)?;
    let result = fetch_task(&conn, id)?;
    drop(conn);
    notify_state_changed(&app);
    Ok(result)
}

/// Stop + review, combined into one atomic transaction (Architecture §6.5, §7): closes any
/// open session, marks the task `done`, and inserts up to three review notes — so a crash
/// mid-flow can never leave a task "stopped" with no record of why.
#[tauri::command]
pub fn complete_task(
    app: AppHandle,
    db: State<Db>,
    id: i64,
    notes: ReviewNotes,
) -> Result<Task, AppError> {
    let what_i_did = validate::note_body_optional(&notes.what_i_did)?;
    let blocker = validate::note_body_optional(&notes.blocker)?;
    let for_next_meeting = validate::note_body_optional(&notes.for_next_meeting)?;

    let mut conn = db.lock().map_err(|_| AppError::new("Internal lock error."))?;
    let now = Utc::now().to_rfc3339();
    let tx = conn.transaction()?;

    let status: String = tx
        .query_row("SELECT status FROM tasks WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .map_err(|_| AppError::new("Task not found."))?;
    if status == "done" {
        return Err(AppError::new("Task is already done."));
    }
    if status == "active" {
        // Safe: `id` is confirmed active, so it is the sole owner of the open session.
        close_open_session(&tx, &now)?;
    }

    tx.execute(
        "UPDATE tasks SET status = 'done', completed_at = ?2 WHERE id = ?1",
        params![id, now],
    )?;

    for (kind, body) in [
        ("review", what_i_did),
        ("blocker", blocker),
        ("meeting", for_next_meeting),
    ] {
        if let Some(body) = body {
            tx.execute(
                "INSERT INTO notes (task_id, kind, body, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![id, kind, body, now],
            )?;
        }
    }

    tx.commit()?;
    let result = fetch_task(&conn, id)?;
    drop(conn);
    notify_state_changed(&app);
    Ok(result)
}
