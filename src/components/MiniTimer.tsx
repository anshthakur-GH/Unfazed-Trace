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
 *
 * The whole pill is a native drag region (`data-tauri-drag-region`) so it can be moved anywhere
 * on screen. Tauri's drag handling swallows the click that would normally follow a plain
 * mousedown+mouseup on a drag region, so "expand" lives on its own small button in the corner
 * instead of "click anywhere" -- buttons are excluded from drag by Tauri itself, so it stays
 * reliably clickable no matter how the rest of the pill is dragged.
 */
export function MiniTimer({ task, onExpand }: MiniTimerProps) {
  return (
    <div
      data-tauri-drag-region="true"
      title="Drag to move"
      className="relative flex h-screen w-screen cursor-move select-none flex-col items-center justify-center"
      style={{
        background: "#101014",
        border: "1px solid #333336",
        borderRadius: "18px",
        boxSizing: "border-box",
        padding: "2%",
      }}
    >
      <button
        type="button"
        onClick={onExpand}
        title="Expand"
        className="absolute right-1.5 top-1.5 flex h-4 w-4 cursor-pointer items-center justify-center rounded text-[10px] leading-none opacity-50 transition-opacity hover:opacity-100"
        style={{ color: "#A2A2A3", background: "transparent" }}
      >
        &#x2922;
      </button>

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
