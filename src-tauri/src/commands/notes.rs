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
