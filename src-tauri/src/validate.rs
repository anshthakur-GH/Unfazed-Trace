use crate::error::AppError;

const MAX_TITLE_LEN: usize = 200;
const MAX_DESCRIPTION_LEN: usize = 2000;
const MAX_NOTE_LEN: usize = 4000;
/// Sanity bound on planned time (30 days worth of minutes) — not a real product constraint,
/// just a guard against garbage input reaching the database.
const MAX_PLANNED_MINUTES: i64 = 60 * 24 * 30;

pub fn title(raw: &str) -> Result<String, AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::new("Title is required."));
    }
    if trimmed.chars().count() > MAX_TITLE_LEN {
        return Err(AppError::new("Title is too long."));
    }
    Ok(trimmed.to_string())
}

pub fn description(raw: &Option<String>) -> Result<Option<String>, AppError> {
    match raw {
        None => Ok(None),
        Some(d) => {
            let trimmed = d.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            if trimmed.chars().count() > MAX_DESCRIPTION_LEN {
                return Err(AppError::new("Description is too long."));
            }
            Ok(Some(trimmed.to_string()))
        }
    }
}

pub fn planned_minutes(raw: &Option<i64>) -> Result<Option<i64>, AppError> {
    match raw {
        None => Ok(None),
        Some(v) if *v < 0 => Err(AppError::new("Planned time cannot be negative.")),
        Some(v) if *v > MAX_PLANNED_MINUTES => {
            Err(AppError::new("Planned time is unreasonably large."))
        }
        Some(v) => Ok(Some(*v)),
    }
}

pub fn remind_at(raw: &Option<String>) -> Result<Option<String>, AppError> {
    match raw {
        None => Ok(None),
        Some(v) if v.trim().is_empty() => Ok(None),
        Some(v) => {
            chrono::DateTime::parse_from_rfc3339(v.trim())
                .map_err(|_| AppError::new("Reminder time must be a valid date/time."))?;
            Ok(Some(v.trim().to_string()))
        }
    }
}

pub fn note_kind(raw: &str) -> Result<String, AppError> {
    match raw {
        "review" | "meeting" | "blocker" => Ok(raw.to_string()),
        _ => Err(AppError::new("Unknown note kind.")),
    }
}

pub fn note_body(raw: &str) -> Result<String, AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::new("Note body cannot be empty."));
    }
    if trimmed.chars().count() > MAX_NOTE_LEN {
        return Err(AppError::new("Note is too long."));
    }
    Ok(trimmed.to_string())
}

/// Like [`note_body`], but an empty/whitespace-only value is treated as "not provided" rather
/// than an error — used for the three optional review fields.
pub fn note_body_optional(raw: &Option<String>) -> Result<Option<String>, AppError> {
    match raw {
        None => Ok(None),
        Some(v) => {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            if trimmed.chars().count() > MAX_NOTE_LEN {
                return Err(AppError::new("Note is too long."));
            }
            Ok(Some(trimmed.to_string()))
        }
    }
}
