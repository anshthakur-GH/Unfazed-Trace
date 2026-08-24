use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::sync::Mutex;
use tauri::Manager;

/// Shared connection guarded by a mutex. A single local writer is all this app ever needs —
/// a connection pool would be unjustified complexity for a single-user desktop app.
pub type Db = Mutex<Connection>;

const MIGRATIONS: &[(&str, &str)] = &[("0001_init", include_str!("../migrations/0001_init.sql"))];

/// Opens (creating if needed) `%APPDATA%\UnfazedTrace\unfazed.db`, applies pragmas, and runs
/// any pending migrations. Also reconciles a session left open by a crash (see
/// [`reconcile_orphaned_session`]).
pub fn open(app: &tauri::AppHandle) -> rusqlite::Result<Connection> {
    let identifier_dir = app
        .path()
        .app_data_dir()
        .expect("app data dir must be resolvable");
    // `app_data_dir()` is `%APPDATA%\<bundle identifier>`; the architecture spec calls for the
    // friendlier `%APPDATA%\UnfazedTrace\unfazed.db`, so we go up one level and rename.
    let base = identifier_dir
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or(identifier_dir);
    let dir = base.join("UnfazedTrace");
    std::fs::create_dir_all(&dir).expect("failed to create app data directory");

    let conn = Connection::open(dir.join("unfazed.db"))?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", true)?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;

    run_migrations(&conn)?;
    reconcile_orphaned_session(&conn)?;

    Ok(conn)
}

fn run_migrations(conn: &Connection) -> rusqlite::Result<()> {
    let current: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    let target = MIGRATIONS.len() as i64;
    if current < target {
        for (i, (_name, sql)) in MIGRATIONS.iter().enumerate() {
            let version = (i + 1) as i64;
            if version > current {
                conn.execute_batch(sql)?;
            }
        }
        conn.pragma_update(None, "user_version", target)?;
    }
    Ok(())
}

/// If the app crashed (or was force-killed) while a task was active, a `time_sessions` row is
/// left with `ended_at IS NULL` forever. On the next launch we close it using the *last
/// safety-flushed* `seconds` value (never "now") so a long-dead gap — e.g. the laptop was off
/// overnight — isn't silently counted as active work. The task is surfaced as `paused` so the
/// user consciously resumes it rather than it appearing to still be running.
fn reconcile_orphaned_session(conn: &Connection) -> rusqlite::Result<()> {
    let open: Option<(i64, i64, String, Option<i64>)> = conn
        .query_row(
            "SELECT id, task_id, started_at, seconds FROM time_sessions WHERE ended_at IS NULL",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;

    if let Some((session_id, task_id, started_at, flushed_seconds)) = open {
        let secs = flushed_seconds.unwrap_or(0).max(0);
        let started: DateTime<Utc> = DateTime::parse_from_rfc3339(&started_at)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let ended = started + chrono::Duration::seconds(secs);

        conn.execute(
            "UPDATE time_sessions SET ended_at = ?1, seconds = ?2 WHERE id = ?3",
            params![ended.to_rfc3339(), secs, session_id],
        )?;
        conn.execute(
            "UPDATE tasks SET total_seconds = total_seconds + ?1, status = 'paused' WHERE id = ?2",
            params![secs, task_id],
        )?;
    }
    Ok(())
}

/// Writes the currently-open session's elapsed-so-far into its `seconds` column without
/// closing it, so a hard crash loses at most the interval between flushes (Architecture §10).
/// Called on a background timer; a no-op when no task is active.
pub fn flush_open_session(conn: &Connection) -> rusqlite::Result<()> {
    let open: Option<(i64, String)> = conn
        .query_row(
            "SELECT id, started_at FROM time_sessions WHERE ended_at IS NULL",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;

    if let Some((session_id, started_at)) = open {
        if let Ok(started) = DateTime::parse_from_rfc3339(&started_at) {
            let secs = (Utc::now() - started.with_timezone(&Utc)).num_seconds().max(0);
            conn.execute(
                "UPDATE time_sessions SET seconds = ?1 WHERE id = ?2",
                params![secs, session_id],
            )?;
        }
    }
    Ok(())
}
