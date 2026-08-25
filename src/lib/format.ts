/** `hh:mm:ss` with tabular digits — used by the live-ticking timer displays. */
export function formatClock(totalSeconds: number): string {
  const s = Math.max(0, Math.floor(totalSeconds));
  const hh = Math.floor(s / 3600);
  const mm = Math.floor((s % 3600) / 60);
  const ss = s % 60;
  const pad = (n: number) => n.toString().padStart(2, "0");
  return `${pad(hh)}:${pad(mm)}:${pad(ss)}`;
}

/** Compact "1h 23m" / "23m" / "45s" — used for static (non-ticking) durations. */
export function formatDuration(totalSeconds: number): string {
  const s = Math.max(0, Math.floor(totalSeconds));
  const hh = Math.floor(s / 3600);
  const mm = Math.floor((s % 3600) / 60);
  if (hh > 0) return `${hh}h ${mm}m`;
  if (mm > 0) return `${mm}m`;
  return `${s}s`;
}

/** "Today" / "Yesterday" / "Mon, Aug 24" for a local "YYYY-MM-DD" date string. */
export function formatDayLabel(date: string): string {
  const d = new Date(`${date}T00:00:00`);
  const now = new Date();
  const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const diffDays = Math.round((startOfToday.getTime() - d.getTime()) / 86_400_000);
  if (diffDays === 0) return "Today";
  if (diffDays === 1) return "Yesterday";
  return d.toLocaleDateString(undefined, { weekday: "short", month: "short", day: "numeric" });
}

/**
 * Live elapsed seconds, derived from timestamps exactly like the Rust `time_math::elapsed_seconds`
 * it mirrors: the cached total plus the running session's duration, if any. Kept in the frontend
 * so the timer can tick every second locally without a round trip to Rust (Architecture §10).
 */
export function elapsedSeconds(
  totalSeconds: number,
  runningStartedAt: string | null,
  nowMs: number,
): number {
  if (!runningStartedAt) return totalSeconds;
  const startedMs = new Date(runningStartedAt).getTime();
  const delta = Math.max(0, Math.floor((nowMs - startedMs) / 1000));
  return totalSeconds + delta;
}
