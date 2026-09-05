use crate::db::Db;
use crate::error::AppError;
use crate::models::{NewNote, Note};
use crate::validate;
use chrono::Utc;
use rusqlite::params;
use tauri::State;

/// Saves a task's note of a given kind -- there is at most one row per (task_id, kind)
/// (enforced by a unique index), so re-saving the same kind updates it in place rather than
/// piling up a history. `created_at` is left untouched on update, so it still reflects when the
/// note was first started.
#[tauri::command]
pub fn add_note(db: State<Db>, note: NewNote) -> Result<Note, AppError> {
    let kind = validate::note_kind(&note.kind)?;
    let body = validate::note_body(&note.body)?;
    let now = Utc::now().to_rfc3339();

    let conn = db.lock().map_err(|_| AppError::new("Internal lock error."))?;
    conn.execute(
        "INSERT INTO notes (task_id, kind, body, created_at) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(task_id, kind) DO UPDATE SET body = excluded.body",
        params![note.task_id, kind, body, now],
    )?;
    conn.query_row(
        "SELECT id, task_id, kind, body, created_at FROM notes WHERE task_id = ?1 AND kind = ?2",
        params![note.task_id, kind],
        |r| {
            Ok(Note {
                id: r.get(0)?,
                task_id: r.get(1)?,
                kind: r.get(2)?,
                body: r.get(3)?,
                created_at: r.get(4)?,
            })
        },
    )
    .map_err(AppError::from)
}

/// Clears a task's note of the given kind -- used when a previously-filled field is emptied
/// out and saved. A no-op if there was nothing to delete.
#[tauri::command]
pub fn delete_task_note(db: State<Db>, id: i64, kind: String) -> Result<(), AppError> {
    let kind = validate::note_kind(&kind)?;
    let conn = db.lock().map_err(|_| AppError::new("Internal lock error."))?;
    conn.execute(
        "DELETE FROM notes WHERE task_id = ?1 AND kind = ?2",
        params![id, kind],
    )?;
    Ok(())
}

/// All notes currently recorded for a task (at most one per kind), oldest-started first -- lets
/// the note dialog pre-fill its fields with whatever was already saved, so reopening it continues
/// from where you left off instead of starting blank.
#[tauri::command]
pub fn list_task_notes(db: State<Db>, id: i64) -> Result<Vec<Note>, AppError> {
    let conn = db.lock().map_err(|_| AppError::new("Internal lock error."))?;
    let mut stmt = conn.prepare(
        "SELECT id, task_id, kind, body, created_at FROM notes WHERE task_id = ?1 ORDER BY created_at",
    )?;
    let notes = stmt
        .query_map([id], |r| {
            Ok(Note {
                id: r.get(0)?,
                task_id: r.get(1)?,
                kind: r.get(2)?,
                body: r.get(3)?,
                created_at: r.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(notes)
}
