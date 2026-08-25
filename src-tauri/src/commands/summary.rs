use crate::db::Db;
use crate::error::AppError;
use crate::models::{DaySummary, Note, Task, TaskWithNotes};
use crate::time_math::elapsed_seconds;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use tauri::State;

/// Column list matching [`row_to_task`]'s positional gets. Kept in one place so the several
/// task queries below stay in sync.
const TASK_COLS: &str = "id, title, description, status, planned_minutes, remind_at, \
    reminder_fired, total_seconds, sort_order, created_at, started_at, completed_at";

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

/// Tasks completed on `date` (a local "YYYY-MM-DD"), each with its notes.
fn done_tasks_for_date(conn: &Connection, date: &str) -> Result<Vec<TaskWithNotes>, AppError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {TASK_COLS} FROM tasks
         WHERE status = 'done' AND date(completed_at, 'localtime') = ?1
         ORDER BY completed_at"
    ))?;
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

/// Tasks created on `date` (any status).
fn created_tasks_for_date(conn: &Connection, date: &str) -> Result<Vec<Task>, AppError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {TASK_COLS} FROM tasks
         WHERE date(created_at, 'localtime') = ?1
         ORDER BY created_at"
    ))?;
    let tasks = stmt
        .query_map([date], row_to_task)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(tasks)
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

fn get_app_state(conn: &Connection, key: &str) -> Result<Option<String>, AppError> {
    conn.query_row("SELECT value FROM app_state WHERE key = ?1", [key], |r| r.get(0))
        .optional()
        .map_err(AppError::from)
}

fn set_app_state(conn: &Connection, key: &str, value: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO app_state (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

/// Ambient "Today" summary: today's tracked total (including the live elapsed of any
/// currently-open session), today's created tasks, and today's completed tasks. Powers the
/// header counter (which re-derives a live tick from this snapshot) and the "Today" tab of the
/// history view (Architecture §6.1).
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

    Ok(DaySummary {
        total_seconds: closed_seconds + open_seconds,
        created_tasks: created_tasks_for_date(&conn, &date)?,
        done_tasks: done_tasks_for_date(&conn, &date)?,
        date,
    })
}

/// Historical review for any date ("YYYY-MM-DD"): tasks created that day and tasks completed
/// that day, plus that day's tracked total. Purely from persisted rows -- no live component.
#[tauri::command]
pub fn get_day_review(db: State<Db>, date: String) -> Result<DaySummary, AppError> {
    if chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").is_err() {
        return Err(AppError::new("Invalid date."));
    }
    let conn = db.lock().map_err(|_| AppError::new("Internal lock error."))?;

    Ok(DaySummary {
        total_seconds: closed_seconds_for_date(&conn, &date)?,
        created_tasks: created_tasks_for_date(&conn, &date)?,
        done_tasks: done_tasks_for_date(&conn, &date)?,
        date,
    })
}

/// Distinct local dates that have any activity -- a task created OR completed that day, newest
/// first. Lets the frontend browse history and only stop on days that actually have data.
#[tauri::command]
pub fn list_history_dates(db: State<Db>) -> Result<Vec<String>, AppError> {
    let conn = db.lock().map_err(|_| AppError::new("Internal lock error."))?;
    let mut stmt = conn.prepare(
        "SELECT d FROM (
            SELECT DISTINCT date(created_at, 'localtime') AS d FROM tasks WHERE created_at IS NOT NULL
            UNION
            SELECT DISTINCT date(completed_at, 'localtime') AS d
              FROM tasks WHERE status = 'done' AND completed_at IS NOT NULL
         )
         WHERE d IS NOT NULL
         ORDER BY d DESC",
    )?;
    let dates = stmt
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(dates)
}

/// The once-a-day catch-up report, fetched on launch. Returns the most recent *prior* day that
/// had completed tasks ("what was done"), but at most once per calendar day -- gated by an
/// `app_state` marker so reopening the window later the same day won't pop it again. Returns
/// `None` when it has already been shown today, or when there is no prior day with completed work.
#[tauri::command]
pub fn get_pending_daily_report(db: State<Db>) -> Result<Option<DaySummary>, AppError> {
    let conn = db.lock().map_err(|_| AppError::new("Internal lock error."))?;
    let today: String = conn.query_row("SELECT date('now', 'localtime')", [], |r| r.get(0))?;

    if get_app_state(&conn, "last_report_shown_date")?.as_deref() == Some(today.as_str()) {
        return Ok(None);
    }
    // Mark today handled up front so reopening the window later today never re-triggers it,
    // whether or not there turns out to be anything to show.
    set_app_state(&conn, "last_report_shown_date", &today)?;

    let prior: Option<String> = conn
        .query_row(
            "SELECT date(completed_at, 'localtime') AS d FROM tasks
             WHERE status = 'done' AND completed_at IS NOT NULL
               AND date(completed_at, 'localtime') < ?1
             ORDER BY d DESC LIMIT 1",
            [&today],
            |r| r.get(0),
        )
        .optional()?;

    match prior {
        Some(date) => Ok(Some(DaySummary {
            total_seconds: closed_seconds_for_date(&conn, &date)?,
            created_tasks: created_tasks_for_date(&conn, &date)?,
            done_tasks: done_tasks_for_date(&conn, &date)?,
            date,
        })),
        None => Ok(None),
    }
}
