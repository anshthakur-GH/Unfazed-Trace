import { useEffect, useState } from "react";
import type { NoteKind, Task } from "../lib/types";
import { formatDuration } from "../lib/format";
import { api } from "../lib/ipc";
import { TimerDigits } from "./TimerDigits";

interface ReviewDialogProps {
  task: Task;
  /** "finish" (default) closes out the task via Stop; "note" jots progress while it keeps running. */
  mode?: "finish" | "note";
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
 * Opening this dialog performs no backend mutation on its own — the caller decides what happens
 * on save. In "finish" mode (default), the parent calls `complete_task`, so Cancel always leaves
 * the task exactly as it was (still ticking if it was active, still paused if it was paused). In
 * "note" mode, the parent calls `add_note` per filled field and the task's status/timer is
 * untouched either way — this is how progress notes get saved while a task is still running.
 *
 * There is at most one saved note per kind per task (an upsert on the backend), so the fields
 * here are pre-filled with whatever was saved last time this opened — editing and saving again
 * continues that same note in place rather than starting over or piling up separate entries.
 */
export function ReviewDialog({ task, mode = "finish", onClose, onSave }: ReviewDialogProps) {
  const [whatIDid, setWhatIDid] = useState("");
  const [blocker, setBlocker] = useState("");
  const [forNextMeeting, setForNextMeeting] = useState("");

  useEffect(() => {
    let cancelled = false;
    void api.listTaskNotes(task.id).then(
      (notes) => {
        if (cancelled) return;
        const bodyFor = (kind: NoteKind) => notes.find((n) => n.kind === kind)?.body ?? "";
        setWhatIDid(bodyFor("review"));
        setBlocker(bodyFor("blocker"));
        setForNextMeeting(bodyFor("meeting"));
      },
      () => {
        // A failed fetch here must never block writing a fresh note.
      },
    );
    return () => {
      cancelled = true;
    };
  }, [task.id]);

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
        {mode === "note" && (
          <div className="text-xs font-medium uppercase tracking-wider" style={{ color: "var(--text-muted)" }}>
            Add note
          </div>
        )}
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
            {mode === "finish" ? "Save & finish" : "Save note"}
          </button>
        </div>
      </form>
    </div>
  );
}
