import type { Task } from "../lib/types";
import { TimerDigits } from "./TimerDigits";

interface MiniTimerProps {
  task: Task;
  onExpand: () => void;
  onPause: (id: number) => void;
  onStop: (task: Task) => void;
}

/**
 * The compact floating-timer view the main window collapses into after ~10s idle while a task
 * runs. Fills its (tiny, always-on-top) window; clicking anywhere expands back to the full app.
 * Reuses TimerDigits so the running time keeps ticking here exactly as it does in the full view.
 */
export function MiniTimer({ task, onExpand, onPause, onStop }: MiniTimerProps) {
  return (
    <div
      onClick={onExpand}
      title="Click to expand"
      className="flex h-screen w-screen cursor-pointer flex-col justify-center gap-1 px-3"
      style={{ background: "var(--bg)" }}
    >
      <div className="truncate text-xs" style={{ color: "var(--text-muted)" }}>
        {task.title}
      </div>
      <div className="tabular text-3xl font-semibold" style={{ color: "var(--text)" }}>
        <TimerDigits totalSeconds={task.total_seconds} runningStartedAt={task.running_started_at} />
      </div>
      <div className="mt-0.5 flex gap-1.5">
        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation();
            onPause(task.id);
          }}
          className="rounded px-2 py-0.5 text-[11px] font-medium"
          style={{ background: "var(--surface-2)", color: "var(--text)" }}
        >
          Pause
        </button>
        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation();
            onStop(task);
          }}
          className="rounded px-2 py-0.5 text-[11px] font-medium"
          style={{ background: "var(--accent)", color: "var(--accent-ink)" }}
        >
          Stop
        </button>
      </div>
    </div>
  );
}
