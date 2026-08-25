import type { Task } from "../lib/types";
import { TimerDigits } from "./TimerDigits";

interface MiniTimerProps {
  task: Task;
  onExpand: () => void;
  /** Retained for callers; the compact pill itself has no controls — double-click to expand. */
  onPause: (id: number) => void;
  onStop: (task: Task) => void;
}

/**
 * The compact, always-on-top floating-timer pill the main window collapses into after ~10s idle
 * while a task runs. Matches the reference: near-black rounded card, big white tabular clock, and
 * a bold-Poppins "Stay Unfazed" caption (amber accent). Content hugs the edges (~2% padding).
 *
 * The whole pill is a native drag region. `data-tauri-drag-region="deep"` is required (not
 * "true") -- Tauri only treats a drag-region element as active when it is the *exact* element
 * under the cursor; "deep" extends that to the whole subtree, so dragging works no matter which
 * child (digits, caption) you actually click on.
 *
 * Double-click expands back to the full window. Tauri's own drag-region handling treats a
 * double-click on a drag region as "toggle maximize" (Windows/Linux) and swallows the event
 * before it can bubble to a normal onDoubleClick -- so this intercepts it in the CAPTURE phase
 * (which runs before Tauri's document-level bubble-phase listener) and stops it from
 * propagating any further, substituting our own expand action for the native toggle-maximize.
 */
export function MiniTimer({ task, onExpand }: MiniTimerProps) {
  return (
    <div
      data-tauri-drag-region="deep"
      title="Drag to move · double-click to expand"
      onMouseDownCapture={(e) => {
        if (e.button === 0 && e.detail === 2) {
          e.preventDefault();
          e.stopPropagation();
          onExpand();
        }
      }}
      className="relative flex h-screen w-screen cursor-move select-none flex-col items-center justify-center"
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
