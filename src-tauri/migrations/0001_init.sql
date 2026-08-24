CREATE TABLE tasks (
  id             INTEGER PRIMARY KEY AUTOINCREMENT,
  title          TEXT NOT NULL,
  description    TEXT,
  status         TEXT NOT NULL DEFAULT 'pending',   -- pending | active | paused | done
  planned_minutes INTEGER,                          -- optional estimate ("time assignment")
  remind_at      TEXT,                              -- optional ISO 8601 datetime for reminder
  reminder_fired INTEGER NOT NULL DEFAULT 0,        -- 0/1, so it only fires once
  total_seconds  INTEGER NOT NULL DEFAULT 0,        -- cached sum of time_sessions
  sort_order     INTEGER NOT NULL DEFAULT 0,
  created_at     TEXT NOT NULL,
  started_at     TEXT,                              -- first time it went active
  completed_at   TEXT
);

CREATE TABLE time_sessions (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  task_id    INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  started_at TEXT NOT NULL,                         -- ISO 8601
  ended_at   TEXT,                                  -- null while running
  seconds    INTEGER                                -- filled on pause/stop; kept fresh by the safety flush while running
);

CREATE TABLE notes (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  task_id    INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  kind       TEXT NOT NULL DEFAULT 'review',        -- review | meeting | blocker
  body       TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE app_state (
  key   TEXT PRIMARY KEY,
  value TEXT
);

CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_sessions_task ON time_sessions(task_id);
CREATE INDEX idx_notes_task ON notes(task_id);
