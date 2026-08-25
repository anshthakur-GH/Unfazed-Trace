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
 * The compact, always-on-top floating-timer pill the main window collapses into after ~10s idle
 * while a task runs. Matches the reference: near-black rounded card, big white tabular clock, and
 * a bold-Poppins "Stay Unfazed" caption (amber accent). Content hugs the edges (~2% padding).
 * Clicking anywhere expands back to the full app, where pause/stop live.
 */
export function MiniTimer({ task, onExpand }: MiniTimerProps) {
  return (
    <div
      onClick={onExpand}
      title="Click to expand"
      className="flex h-screen w-screen cursor-pointer select-none flex-col items-center justify-center"
      style={{
        background: "#101014",
        border: "1px solid #333336",
        borderRadius: "18px",
        boxSizing: "border-box",
        padding: "2%",
      }}
    >
      <div className="tabular font-bold leading-none tracking-tight text-white" style={{ fontSize: "30px" }}>
        <TimerDigits totalSeconds={task.total_seconds} runningStartedAt={task.running_started_at} />
      </div>
      <div
        className="leading-none"
        style={{ fontFamily: "Poppins, sans-serif", fontWeight: 700, fontSize: "12px", marginTop: "3px" }}
      >
        <span style={{ color: "#A2A2A3" }}>Stay </span>
        <span style={{ color: "#F5A623" }}>Unfazed</span>
      </div>
    </div>
  );
}
