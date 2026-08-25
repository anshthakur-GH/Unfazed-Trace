import { formatDuration } from "../lib/format";
import type { DaySummary, NoteKind, Task, TaskStatus } from "../lib/types";

const NOTE_LABELS: Record<NoteKind, string> = {
  review: "What I did",
  blocker: "Blocker",
  meeting: "For next meeting",
};

const STATUS_LABELS: Record<TaskStatus, string> = {
  pending: "Pending",
  active: "Active",
  paused: "Paused",
  done: "Done",
};

function SectionHeading({ children }: { children: React.ReactNode }) {
  return (
    <div
      className="text-xs font-medium uppercase tracking-wider"
      style={{ color: "var(--text-muted)" }}
    >
      {children}
    </div>
  );
}

function CreatedRow({ task }: { task: Task }) {
  return (
    <div
      className="rounded-xl border p-3"
      style={{ borderColor: "var(--border)", background: "var(--surface)" }}
    >
      <div className="flex items-baseline justify-between gap-2">
        <span className="truncate text-sm font-medium">{task.title}</span>
        <span className="shrink-0 text-xs" style={{ color: "var(--text-muted)" }}>
          {STATUS_LABELS[task.status]}
        </span>
      </div>
      {task.planned_minutes != null && (
        <p className="mt-1 text-xs" style={{ color: "var(--text-muted)" }}>
          {task.planned_minutes}m planned
        </p>
      )}
    </div>
  );
}

/**
 * The shared body of a single day's report: "Done" (completed tasks + their review notes) and
 * "Created" (tasks created that day, any status). Used by both the history browser and the
 * once-a-day launch report so the two always render identically.
 */
export function DayReportBody({ summary }: { summary: DaySummary }) {
  if (summary.created_tasks.length === 0 && summary.done_tasks.length === 0) {
    return (
      <p className="text-sm" style={{ color: "var(--text-muted)" }}>
        Nothing recorded on this day.
      </p>
    );
  }

  return (
    <div className="flex flex-col gap-5">
      {summary.done_tasks.length > 0 && (
        <section className="flex flex-col gap-2">
          <SectionHeading>Done</SectionHeading>
          {summary.done_tasks.map((task) => (
            <div
              key={`done-${task.id}`}
              className="rounded-xl border p-3"
              style={{ borderColor: "var(--border)", background: "var(--surface)" }}
            >
              <div className="flex items-baseline justify-between gap-2">
                <span className="truncate text-sm font-medium">{task.title}</span>
                <span className="tabular shrink-0 text-xs" style={{ color: "var(--text-muted)" }}>
                  {formatDuration(task.total_seconds)}
                </span>
              </div>
              {task.notes.length === 0 ? (
                <p className="mt-1 text-xs" style={{ color: "var(--text-muted)" }}>
                  No notes recorded.
                </p>
              ) : (
                <div className="mt-2 flex flex-col gap-1.5">
                  {task.notes.map((note) => (
                    <div key={note.id} className="text-xs">
                      <span style={{ color: "var(--text-muted)" }}>{NOTE_LABELS[note.kind]}: </span>
                      <span>{note.body}</span>
                    </div>
                  ))}
                </div>
              )}
            </div>
          ))}
        </section>
      )}

      {summary.created_tasks.length > 0 && (
        <section className="flex flex-col gap-2">
          <SectionHeading>Created</SectionHeading>
          {summary.created_tasks.map((task) => (
            <CreatedRow key={`created-${task.id}`} task={task} />
          ))}
        </section>
      )}
    </div>
  );
}
