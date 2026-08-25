import type { Task } from "../lib/types";
import { TimerDigits } from "./TimerDigits";

interface MiniTimerProps {
  task: Task;
  onExpand: () => void;
  /** Retained for callers; the compact pill itself has no controls — click to expand for those. */
  onPause: (id: number) => void;
  onStop: (task: Task) => void;
}

/**
 * The compact floating-timer pill the main window collapses into after ~10s idle while a task
 * runs. Matches the reference design: near-black rounded card, big white tabular clock, and a
 * "Stay Unfazed" caption (amber accent). Clicking anywhere expands back to the full app, where
 * pause/stop live. Reuses TimerDigits so the clock keeps ticking identically to the full view.
 */
export function MiniTimer({ task, onExpand }: MiniTimerProps) {
  return (
    <div
      onClick={onExpand}
      title="Click to expand"
      className="flex h-screen w-screen cursor-pointer select-none flex-col items-center justify-center gap-0.5"
      style={{
        background: "#101014",
        border: "1px solid #333336",
        boxSizing: "border-box",
      }}
    >
      <div className="tabular text-[30px] font-bold leading-none tracking-tight text-white">
        <TimerDigits totalSeconds={task.total_seconds} runningStartedAt={task.running_started_at} />
      </div>
      <div className="text-xs font-medium leading-none">
        <span style={{ color: "#A2A2A3" }}>Stay </span>
        <span style={{ color: "#F5A623" }}>Unfazed</span>
      </div>
    </div>
  );
}
