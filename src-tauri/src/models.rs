use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub planned_minutes: Option<i64>,
    pub remind_at: Option<String>,
    pub reminder_fired: bool,
    pub total_seconds: i64,
    pub sort_order: i64,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    /// Start timestamp of the currently-open `time_sessions` row, if this task is the one
    /// active task. The frontend derives live elapsed as `now - running_started_at + total_seconds`.
    pub running_started_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewTask {
    pub title: String,
    pub description: Option<String>,
    pub planned_minutes: Option<i64>,
    pub remind_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTask {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub planned_minutes: Option<i64>,
    pub remind_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Note {
    pub id: i64,
    pub task_id: i64,
    pub kind: String,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewNote {
    pub task_id: i64,
    pub kind: String,
    pub body: String,
}

/// The three optional review fields captured when a task is completed (Architecture §6.5).
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ReviewNotes {
    pub what_i_did: Option<String>,
    pub blocker: Option<String>,
    pub for_next_meeting: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskWithNotes {
    #[serde(flatten)]
    pub task: Task,
    pub notes: Vec<Note>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DaySummary {
    /// Local calendar date (YYYY-MM-DD) this summary covers.
    pub date: String,
    /// Time tracked on this day (closed sessions started this day; for the live "today"
    /// summary this also folds in the currently-open session's elapsed-so-far).
    pub total_seconds: i64,
    /// Tasks created on this day, in any status — "what was planned/created that day".
    pub created_tasks: Vec<Task>,
    /// Tasks completed on this day, each with its review notes — "what was done that day".
    pub done_tasks: Vec<TaskWithNotes>,
}
