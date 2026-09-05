use crate::db::Db;
use crate::error::AppError;
use crate::models::{NewNote, Note};
use crate::validate;
use chrono::Utc;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn add_note(db: State<Db>, note: NewNote) -> Result<Note, AppError> {
    let kind = validate::note_kind(&note.kind)?;
    let body = validate::note_body(&note.body)?;
    let now = Utc::now().to_rfc3339();

    let conn = db.lock().map_err(|_| AppError::new("Internal lock error."))?;
    conn.execute(
        "INSERT INTO notes (task_id, kind, body, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![note.task_id, kind, body, now],
    )?;
    let id = conn.last_insert_rowid();
    conn.query_row(
        "SELECT id, task_id, kind, body, created_at FROM notes WHERE id = ?1",
        [id],
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

/// All notes recorded for a task so far, oldest first -- lets the note-taking dialog show what
/// was already saved (mid-task notes accumulate as separate rows, one per Save) instead of
/// looking like each save replaced the last.
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
