use chrono::{DateTime, Utc};

/// Live elapsed seconds for a task: the cached sum of closed sessions (`total_seconds`) plus
/// the currently-open session's duration, if any. This is the single source of truth for
/// "how long has this task run" — it is derived from timestamps, never from a counter that
/// would drift or reset when the window is closed (Architecture §5, §10).
pub fn elapsed_seconds(
    total_seconds: i64,
    running_started_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> i64 {
    match running_started_at {
        Some(started) => total_seconds + (now - started).num_seconds().max(0),
        None => total_seconds,
    }
}

/// Whole seconds elapsed between two RFC3339 timestamps, clamped to zero (never negative,
/// e.g. under clock adjustment).
pub fn seconds_between(start: &str, end: &str) -> Result<i64, chrono::ParseError> {
    let start = DateTime::parse_from_rfc3339(start)?;
    let end = DateTime::parse_from_rfc3339(end)?;
    Ok((end - start).num_seconds().max(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn elapsed_with_no_running_session_is_just_total() {
        let now = Utc::now();
        assert_eq!(elapsed_seconds(120, None, now), 120);
    }

    #[test]
    fn elapsed_adds_running_session_duration() {
        let now = Utc::now();
        let started = now - Duration::seconds(30);
        assert_eq!(elapsed_seconds(100, Some(started), now), 130);
    }

    #[test]
    fn elapsed_clamps_negative_drift_to_zero() {
        // A running session that appears to start in the future (clock skew) must never
        // subtract time.
        let now = Utc::now();
        let started = now + Duration::seconds(10);
        assert_eq!(elapsed_seconds(50, Some(started), now), 50);
    }

    #[test]
    fn seconds_between_computes_duration() {
        let start = "2026-01-01T10:00:00+00:00";
        let end = "2026-01-01T10:05:30+00:00";
        assert_eq!(seconds_between(start, end).unwrap(), 330);
    }

    #[test]
    fn seconds_between_clamps_when_end_precedes_start() {
        let start = "2026-01-01T10:05:00+00:00";
        let end = "2026-01-01T10:00:00+00:00";
        assert_eq!(seconds_between(start, end).unwrap(), 0);
    }

    #[test]
    fn seconds_between_rejects_malformed_timestamps() {
        assert!(seconds_between("not-a-date", "2026-01-01T10:00:00+00:00").is_err());
    }

    #[test]
    fn seconds_between_handles_mixed_timezone_offsets() {
        // Sessions are always written in UTC ("+00:00") in this app, but the math itself
        // must be correct regardless of offset, since RFC3339 permits any offset.
        // "2026-01-01T10:00:00+05:30" is 2026-01-01T04:30:00 UTC.
        let start = "2026-01-01T10:00:00+05:30";
        let end = "2026-01-01T04:31:00+00:00"; // same instant + 1 minute
        assert_eq!(seconds_between(start, end).unwrap(), 60);
    }
}
