use crate::db::Db;
use crate::error::AppError;
use crate::models::{DaySummary, Note, Task, TaskWithNotes};
use crate::time_math::elapsed_seconds;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension};
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

/// Tasks completed on `date` (a "YYYY-MM-DD" local-calendar-date string), each with its notes.
/// Shared by [`get_day_summary`] (today, live) and [`get_day_review`] (any date, historical).
fn done_tasks_for_date(conn: &Connection, date: &str) -> Result<Vec<TaskWithNotes>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, title, description, status, planned_minutes, remind_at, reminder_fired,
                total_seconds, sort_order, created_at, started_at, completed_at
         FROM tasks
         WHERE status = 'done' AND date(completed_at, 'localtime') = ?1
         ORDER BY completed_at",
    )?;
    let tasks = stmt
        .query_map([date], row_to_task)?
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
    Ok(done_tasks)
}

/// Total closed-session seconds that started on `date`.
fn closed_seconds_for_date(conn: &Connection, date: &str) -> Result<i64, AppError> {
    conn.query_row(
        "SELECT COALESCE(SUM(seconds), 0) FROM time_sessions
         WHERE ended_at IS NOT NULL AND date(started_at, 'localtime') = ?1",
        [date],
        |r| r.get(0),
    )
    .map_err(AppError::from)
}

/// Ambient "Today" summary: today's tracked total (including the live elapsed of any
/// currently-open session) plus today's completed tasks. Powers the header counter, which
/// re-derives a live tick from this snapshot (Architecture §6.1).
#[tauri::command]
pub fn get_day_summary(db: State<Db>) -> Result<DaySummary, AppError> {
    let conn = db.lock().map_err(|_| AppError::new("Internal lock error."))?;
    let now = Utc::now();

    let date: String = conn.query_row("SELECT date('now', 'localtime')", [], |r| r.get(0))?;
    let closed_seconds = closed_seconds_for_date(&conn, &date)?;

    let open_session: Option<String> = conn
        .query_row(
            "SELECT started_at FROM time_sessions WHERE ended_at IS NULL AND date(started_at, 'localtime') = ?1",
            [&date],
            |r| r.get(0),
        )
        .optional()?;
    let open_seconds = match open_session.and_then(|s| DateTime::parse_from_rfc3339(&s).ok()) {
        Some(started) => elapsed_seconds(0, Some(started.with_timezone(&Utc)), now),
        None => 0,
    };

    let done_tasks = done_tasks_for_date(&conn, &date)?;

    Ok(DaySummary {
        date,
        total_seconds_today: closed_seconds + open_seconds,
        done_tasks,
    })
}

/// Historical review for any past (or the current) date, by "YYYY-MM-DD". Purely from closed
/// sessions/completed tasks -- no live-session component, since this is the browse-history
/// view rather than the ambient "right now" counter.
#[tauri::command]
pub fn get_day_review(db: State<Db>, date: String) -> Result<DaySummary, AppError> {
    if chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").is_err() {
        return Err(AppError::new("Invalid date."));
    }
    let conn = db.lock().map_err(|_| AppError::new("Internal lock error."))?;
    let closed_seconds = closed_seconds_for_date(&conn, &date)?;
    let done_tasks = done_tasks_for_date(&conn, &date)?;

    Ok(DaySummary {
        date,
        total_seconds_today: closed_seconds,
        done_tasks,
    })
}

/// Distinct local-calendar dates that have at least one completed task, newest first --
/// lets the frontend browse history without an open-ended calendar full of empty days.
#[tauri::command]
pub fn list_history_dates(db: State<Db>) -> Result<Vec<String>, AppError> {
    let conn = db.lock().map_err(|_| AppError::new("Internal lock error."))?;
    let mut stmt = conn.prepare(
        "SELECT DISTINCT date(completed_at, 'localtime') AS d
         FROM tasks
         WHERE status = 'done' AND completed_at IS NOT NULL
         ORDER BY d DESC",
    )?;
    let dates = stmt
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(dates)
}
