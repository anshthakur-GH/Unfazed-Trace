import { useState } from "react";
import type { Task } from "../lib/types";
import { formatDuration } from "../lib/format";
import { TimerDigits } from "./TimerDigits";

interface ReviewDialogProps {
  task: Task;
  onClose: () => void;
  onSave: (notes: {
    what_i_did: string | null;
    blocker: string | null;
    for_next_meeting: string | null;
  }) => void;
}

const fieldClass =
  "mt-1 w-full resize-none rounded-lg px-3 py-2 text-sm text-white outline-none placeholder:text-[color:var(--text-muted)]";
const fieldStyle: React.CSSProperties = {
  background: "var(--surface-2)",
  border: "1px solid var(--border)",
};

/**
 * Opening this dialog performs no backend mutation — "Stop" is purely a client-side view
 * change. Only "Save & finish" calls `complete_task`, so Cancel always leaves the task exactly
 * as it was (still ticking if it was active, still paused if it was paused).
 */
export function ReviewDialog({ task, onClose, onSave }: ReviewDialogProps) {
  const [whatIDid, setWhatIDid] = useState("");
  const [blocker, setBlocker] = useState("");
  const [forNextMeeting, setForNextMeeting] = useState("");

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    onSave({
      what_i_did: whatIDid.trim() || null,
      blocker: blocker.trim() || null,
      for_next_meeting: forNextMeeting.trim() || null,
    });
  }

  return (
    <div className="fixed inset-0 z-50 flex items-end justify-center bg-black/60 p-0 sm:items-center sm:p-4">
      <form
        onSubmit={handleSubmit}
        className="w-full max-w-sm rounded-t-2xl p-5 sm:rounded-2xl"
        style={{ background: "var(--surface)", border: "1px solid var(--border)" }}
      >
        <h2 className="truncate text-base font-semibold">{task.title}</h2>
        <div className="mt-1 text-2xl font-semibold">
          {task.status === "active" ? (
            <TimerDigits totalSeconds={task.total_seconds} runningStartedAt={task.running_started_at} />
          ) : (
            <span className="tabular">{formatDuration(task.total_seconds)}</span>
          )}
        </div>

        <label className="mt-4 block text-xs" style={{ color: "var(--text-muted)" }}>
          What I did
          <textarea
            autoFocus
            value={whatIDid}
            onChange={(e) => setWhatIDid(e.target.value)}
            rows={2}
            className={fieldClass}
            style={fieldStyle}
          />
        </label>

        <label className="mt-3 block text-xs" style={{ color: "var(--text-muted)" }}>
          Blocker / problem (optional)
          <textarea
            value={blocker}
            onChange={(e) => setBlocker(e.target.value)}
            rows={2}
            className={fieldClass}
            style={fieldStyle}
          />
        </label>

        <label className="mt-3 block text-xs" style={{ color: "var(--text-muted)" }}>
          For next meeting (optional)
          <textarea
            value={forNextMeeting}
            onChange={(e) => setForNextMeeting(e.target.value)}
            rows={2}
            className={fieldClass}
            style={fieldStyle}
          />
        </label>

        <div className="mt-5 flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="rounded-lg px-3 py-2 text-sm"
            style={{ color: "var(--text-muted)" }}
          >
            Cancel
          </button>
          <button
            type="submit"
            className="rounded-lg px-4 py-2 text-sm font-medium"
            style={{ background: "var(--accent)", color: "var(--accent-ink)" }}
          >
            Save & finish
          </button>
        </div>
      </form>
    </div>
  );
}
