import { useEffect, useState } from "react";
import type { Task } from "../lib/types";

interface TaskEditorModalProps {
  /** null when creating a new task. */
  task: Task | null;
  onClose: () => void;
  onSave: (data: {
    title: string;
    description: string | null;
    planned_minutes: number | null;
    remind_at: string | null;
  }) => void;
}

const fieldClass =
  "mt-1 w-full rounded-lg px-3 py-2 text-sm text-white outline-none placeholder:text-[color:var(--text-muted)]";
const fieldStyle: React.CSSProperties = {
  background: "var(--surface-2)",
  border: "1px solid var(--border)",
};

function toDatetimeLocal(iso: string | null): string {
  if (!iso) return "";
  const d = new Date(iso);
  const pad = (n: number) => n.toString().padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

export function TaskEditorModal({ task, onClose, onSave }: TaskEditorModalProps) {
  const [title, setTitle] = useState(task?.title ?? "");
  const [description, setDescription] = useState(task?.description ?? "");
  const [plannedMinutes, setPlannedMinutes] = useState(task?.planned_minutes?.toString() ?? "");
  const [remindAt, setRemindAt] = useState(toDatetimeLocal(task?.remind_at ?? null));

  useEffect(() => {
    setTitle(task?.title ?? "");
    setDescription(task?.description ?? "");
    setPlannedMinutes(task?.planned_minutes?.toString() ?? "");
    setRemindAt(toDatetimeLocal(task?.remind_at ?? null));
  }, [task]);

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!title.trim()) return;
    onSave({
      title: title.trim(),
      description: description.trim() || null,
      planned_minutes: plannedMinutes.trim() ? Number(plannedMinutes) : null,
      remind_at: remindAt ? new Date(remindAt).toISOString() : null,
    });
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-end justify-center bg-black/60 p-0 sm:items-center sm:p-4"
      onClick={onClose}
    >
      <form
        onClick={(e) => e.stopPropagation()}
        onSubmit={handleSubmit}
        className="w-full max-w-sm rounded-t-2xl p-5 sm:rounded-2xl"
        style={{ background: "var(--surface)", border: "1px solid var(--border)" }}
      >
        <h2 className="text-base font-semibold">{task ? "Edit task" : "New task"}</h2>

        <label className="mt-4 block text-xs" style={{ color: "var(--text-muted)" }}>
          Task
          <input
            autoFocus
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="What are you working on?"
            className={fieldClass}
            style={fieldStyle}
          />
        </label>

        <label className="mt-3 block text-xs" style={{ color: "var(--text-muted)" }}>
          Description (optional)
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            rows={2}
            className={`${fieldClass} resize-none`}
            style={fieldStyle}
          />
        </label>

        <div className="mt-3 flex gap-3">
          <label className="block flex-1 text-xs" style={{ color: "var(--text-muted)" }}>
            Planned time (min)
            <input
              type="number"
              min={0}
              value={plannedMinutes}
              onChange={(e) => setPlannedMinutes(e.target.value)}
              placeholder="Optional"
              className={fieldClass}
              style={fieldStyle}
            />
          </label>
          <label className="block flex-1 text-xs" style={{ color: "var(--text-muted)" }}>
            Remind me at
            <input
              type="datetime-local"
              value={remindAt}
              onChange={(e) => setRemindAt(e.target.value)}
              className={fieldClass}
              style={fieldStyle}
            />
          </label>
        </div>

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
            Save
          </button>
        </div>
      </form>
    </div>
  );
}
