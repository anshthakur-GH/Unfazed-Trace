use crate::db::Db;
use crate::error::AppError;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension};
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tauri::{AppHandle, Manager};
use tokio::sync::Notify;

/// Wraps the notifier the scheduler sleeps on, managed as Tauri state so any command that
/// touches a `remind_at` can wake it to recompute its next sleep target immediately, rather
/// than waiting out whatever (possibly very long) sleep it's currently in.
pub struct SchedulerNotify(pub Arc<Notify>);

pub fn wake(app: &AppHandle) {
    if let Some(notify) = app.try_state::<SchedulerNotify>() {
        notify.0.notify_one();
    }
}

/// Finds the soonest-due, not-yet-fired reminder among pending tasks.
fn next_reminder(conn: &Connection) -> Option<(i64, String, String)> {
    conn.query_row(
        "SELECT id, title, remind_at FROM tasks
         WHERE status = 'pending' AND remind_at IS NOT NULL AND reminder_fired = 0
         ORDER BY remind_at LIMIT 1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )
    .optional()
    .unwrap_or(None)
}

/// Atomically claims the reminder: only proceeds if the task is still pending and unfired,
/// guarding against a race where the user started the task in the instant the timer elapsed.
fn mark_fired_if_still_eligible(conn: &Connection, task_id: i64) -> Result<bool, AppError> {
    let changed = conn.execute(
        "UPDATE tasks SET reminder_fired = 1
         WHERE id = ?1 AND status = 'pending' AND reminder_fired = 0",
        [task_id],
    )?;
    Ok(changed > 0)
}

/// Event-driven reminder scheduler: sleeps until the next `remind_at`, woken early by [`wake`]
/// whenever a task mutation could change that target. Never polls (Architecture §6.3, §10).
pub fn spawn(app: AppHandle, notify: Arc<Notify>) {
    tauri::async_runtime::spawn(async move {
        loop {
            let due = {
                let db = app.state::<Db>();
                let conn = match db.lock() {
                    Ok(c) => c,
                    Err(_) => return,
                };
                next_reminder(&conn)
            };

            match due {
                Some((task_id, title, remind_at)) => {
                    let target = DateTime::parse_from_rfc3339(&remind_at)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now());
                    let wait = (target - Utc::now()).to_std().unwrap_or(StdDuration::ZERO);

                    tokio::select! {
                        _ = tokio::time::sleep(wait) => {
                            fire(&app, task_id, &title);
                        }
                        _ = notify.notified() => {
                            // A task changed -- loop around and recompute the next target.
                        }
                    }
                }
                None => {
                    // Nothing pending: block until woken instead of polling.
                    notify.notified().await;
                }
            }
        }
    });
}

fn fire(app: &AppHandle, task_id: i64, title: &str) {
    let db = app.state::<Db>();
    let should_show = match db.lock() {
        Ok(conn) => mark_fired_if_still_eligible(&conn, task_id).unwrap_or(false),
        Err(_) => false,
    };
    if should_show {
        #[cfg(windows)]
        crate::services::toast::show_reminder(app, task_id, title);
        #[cfg(not(windows))]
        eprintln!("reminder due: {title} (native toasts are Windows-only)");

        crate::commands::notify_state_changed(app);
    }
}
