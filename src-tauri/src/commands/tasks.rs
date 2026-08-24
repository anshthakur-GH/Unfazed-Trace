use crate::db::Db;
use crate::error::AppError;
use crate::models::{NewTask, Task, UpdateTask};
use crate::validate;
use chrono::Utc;
use rusqlite::{params, Connection};
use tauri::{AppHandle, State};

use super::notify_state_changed;
use crate::services::scheduler;

const TASK_COLUMNS: &str = "
    t.id, t.title, t.description, t.status, t.planned_minutes, t.remind_at,
    t.reminder_fired, t.total_seconds, t.sort_order, t.created_at, t.started_at,
    t.completed_at, ts.started_at AS running_started_at
";

fn row_to_task(row: &rusqlite::Row) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get(0)?,
        title: row.get(1)?,
        description: row.get(2)?,
        status: row.get(3)?,
        planned_minutes: row.get(4)?,
        remind_at: row.get(5)?,
        reminder_fired: row.get::<_, i64>(6)? != 0,
        total_seconds: row.get(7)?,
        sort_order: row.get(8)?,
        created_at: row.get(9)?,
        started_at: row.get(10)?,
        completed_at: row.get(11)?,
        running_started_at: row.get(12)?,
    })
}

/// Fetches a single task, joined against its currently-open session (if any). Shared by every
/// command module so the "running_started_at" projection stays in one place.
pub(crate) fn fetch_task(conn: &Connection, id: i64) -> Result<Task, AppError> {
    conn.query_row(
        &format!(
            "SELECT {TASK_COLUMNS} FROM tasks t
             LEFT JOIN time_sessions ts ON ts.task_id = t.id AND ts.ended_at IS NULL
             WHERE t.id = ?1"
        ),
        [id],
        row_to_task,
    )
    .map_err(|_| AppError::new("Task not found."))
}

#[tauri::command]
pub fn list_tasks(db: State<Db>) -> Result<Vec<Task>, AppError> {
    let conn = db.lock().map_err(|_| AppError::new("Internal lock error."))?;
    let mut stmt = conn.prepare(&format!(
        "SELECT {TASK_COLUMNS} FROM tasks t
         LEFT JOIN time_sessions ts ON ts.task_id = t.id AND ts.ended_at IS NULL
         WHERE t.status != 'done' OR date(t.completed_at) = date('now', 'localtime')
         ORDER BY
           CASE t.status WHEN 'active' THEN 0 WHEN 'paused' THEN 1 WHEN 'pending' THEN 2 ELSE 3 END,
           t.sort_order, t.created_at"
    ))?;
    let tasks = stmt
        .query_map([], row_to_task)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(tasks)
}

#[tauri::command]
pub fn create_task(app: AppHandle, db: State<Db>, task: NewTask) -> Result<Task, AppError> {
    let title = validate::title(&task.title)?;
    let description = validate::description(&task.description)?;
    let planned_minutes = validate::planned_minutes(&task.planned_minutes)?;
    let remind_at = validate::remind_at(&task.remind_at)?;
    let now = Utc::now().to_rfc3339();

    let conn = db.lock().map_err(|_| AppError::new("Internal lock error."))?;
    conn.execute(
        "INSERT INTO tasks (title, description, status, planned_minutes, remind_at, created_at)
         VALUES (?1, ?2, 'pending', ?3, ?4, ?5)",
        params![title, description, planned_minutes, remind_at, now],
    )?;
    let id = conn.last_insert_rowid();
    let result = fetch_task(&conn, id)?;
    drop(conn);
    notify_state_changed(&app);
    scheduler::wake(&app);
    Ok(result)
}

#[tauri::command]
pub fn update_task(app: AppHandle, db: State<Db>, task: UpdateTask) -> Result<Task, AppError> {
    let title = validate::title(&task.title)?;
    let description = validate::description(&task.description)?;
    let planned_minutes = validate::planned_minutes(&task.planned_minutes)?;
    let remind_at = validate::remind_at(&task.remind_at)?;

    let conn = db.lock().map_err(|_| AppError::new("Internal lock error."))?;
    // Changing the reminder time re-arms it, so a rescheduled reminder can fire again.
    conn.execute(
        "UPDATE tasks
         SET title = ?1, description = ?2, planned_minutes = ?3, remind_at = ?4,
             reminder_fired = CASE WHEN remind_at IS NOT ?4 THEN 0 ELSE reminder_fired END
         WHERE id = ?5",
        params![title, description, planned_minutes, remind_at, task.id],
    )?;
    if conn.changes() == 0 {
        return Err(AppError::new("Task not found."));
    }
    let result = fetch_task(&conn, task.id)?;
    drop(conn);
    notify_state_changed(&app);
    scheduler::wake(&app);
    Ok(result)
}

#[tauri::command]
pub fn delete_task(app: AppHandle, db: State<Db>, id: i64) -> Result<(), AppError> {
    let conn = db.lock().map_err(|_| AppError::new("Internal lock error."))?;
    // ON DELETE CASCADE (foreign_keys=ON) takes care of time_sessions and notes.
    conn.execute("DELETE FROM tasks WHERE id = ?1", [id])?;
    drop(conn);
    notify_state_changed(&app);
    scheduler::wake(&app);
    Ok(())
}
