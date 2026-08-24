use crate::db::Db;
use crate::error::AppError;
use crate::models::{DaySummary, Note, Task, TaskWithNotes};
use crate::time_math::elapsed_seconds;
use chrono::{DateTime, Utc};
use rusqlite::OptionalExtension;
use tauri::State;

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
        running_started_at: None,
    })
}

#[tauri::command]
pub fn get_day_summary(db: State<Db>) -> Result<DaySummary, AppError> {
    let conn = db.lock().map_err(|_| AppError::new("Internal lock error."))?;
    let now = Utc::now();

    let date: String = conn.query_row("SELECT date('now', 'localtime')", [], |r| r.get(0))?;

    // Ambient "Today" total: every closed session that started today, plus the live elapsed
    // of an open session if it also started today (a session spanning midnight only counts
    // its today-portion via this same live-elapsed math on the next day's open session).
    let closed_seconds_today: i64 = conn.query_row(
        "SELECT COALESCE(SUM(seconds), 0) FROM time_sessions
         WHERE ended_at IS NOT NULL AND date(started_at, 'localtime') = date('now', 'localtime')",
        [],
        |r| r.get(0),
    )?;
    let open_session: Option<String> = conn
        .query_row(
            "SELECT started_at FROM time_sessions
             WHERE ended_at IS NULL AND date(started_at, 'localtime') = date('now', 'localtime')",
            [],
            |r| r.get(0),
        )
        .optional()?;
    let open_seconds = match open_session.and_then(|s| DateTime::parse_from_rfc3339(&s).ok()) {
        Some(started) => elapsed_seconds(0, Some(started.with_timezone(&Utc)), now),
        None => 0,
    };

    let mut stmt = conn.prepare(
        "SELECT id, title, description, status, planned_minutes, remind_at, reminder_fired,
                total_seconds, sort_order, created_at, started_at, completed_at
         FROM tasks
         WHERE status = 'done' AND date(completed_at, 'localtime') = date('now', 'localtime')
         ORDER BY completed_at",
    )?;
    let tasks = stmt
        .query_map([], row_to_task)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    drop(stmt);

    let mut done_tasks = Vec::with_capacity(tasks.len());
    for task in tasks {
        let mut note_stmt = conn.prepare(
            "SELECT id, task_id, kind, body, created_at FROM notes WHERE task_id = ?1 ORDER BY created_at",
        )?;
        let notes = note_stmt
            .query_map([task.id], |r| {
                Ok(Note {
                    id: r.get(0)?,
                    task_id: r.get(1)?,
                    kind: r.get(2)?,
                    body: r.get(3)?,
                    created_at: r.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        done_tasks.push(TaskWithNotes { task, notes });
    }

    Ok(DaySummary {
        date,
        total_seconds_today: closed_seconds_today + open_seconds,
        done_tasks,
    })
}
