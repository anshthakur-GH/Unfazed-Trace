import type { Task } from "../lib/types";
import { elapsedSeconds } from "../lib/format";
import { TimerDigits } from "./TimerDigits";

interface TodayCounterProps {
  totalSecondsToday: number;
  fetchedAtMs: number;
  activeTask: Task | null;
}

/**
 * Ambient, read-only "Today" total (Architecture §6.1) — the spiritual successor to the
 * always-on timer this app replaces. Ticks live while a task is active by reusing TimerDigits:
 * we back the active session's already-counted contribution out of the last fetched snapshot,
 * then let TimerDigits add its own fresh live elapsed back in every second.
 */
export function TodayCounter({ totalSecondsToday, fetchedAtMs, activeTask }: TodayCounterProps) {
  const baseline = activeTask
    ? totalSecondsToday - elapsedSeconds(0, activeTask.running_started_at, fetchedAtMs)
    : totalSecondsToday;

  return (
    <div>
      <div className="text-xs uppercase tracking-wider" style={{ color: "var(--text-muted)" }}>
        Today
      </div>
      <div className="mt-1 text-3xl font-semibold">
        <TimerDigits
          totalSeconds={baseline}
          runningStartedAt={activeTask?.running_started_at ?? null}
        />
      </div>
    </div>
  );
}
