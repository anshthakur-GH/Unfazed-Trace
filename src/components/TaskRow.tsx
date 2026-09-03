import type { Task } from "../lib/types";
import { formatDuration } from "../lib/format";
import { TimerDigits } from "./TimerDigits";

interface TaskRowProps {
  task: Task;
  onStart: (id: number) => void;
  onPause: (id: number) => void;
  onStop: (task: Task) => void;
  onEdit: (task: Task) => void;
  onDelete: (id: number) => void;
  onNote: (task: Task) => void;
}

function ActionButton({
  children,
  onClick,
  variant = "ghost",
}: {
  children: React.ReactNode;
  onClick: () => void;
  variant?: "primary" | "neutral" | "ghost" | "danger";
}) {
  const styles: Record<string, React.CSSProperties> = {
    primary: { background: "var(--accent)", color: "var(--accent-ink)" },
    neutral: { background: "var(--surface-2)", color: "var(--text)" },
    ghost: { color: "var(--text-muted)" },
    danger: { color: "var(--danger)" },
  };
  return (
    <button
      type="button"
      onClick={onClick}
      className="rounded-lg px-3 py-1.5 text-xs font-medium transition-colors"
      style={styles[variant]}
    >
      {children}
    </button>
  );
}

export function TaskRow({ task, onStart, onPause, onStop, onEdit, onDelete, onNote }: TaskRowProps) {
  const isActive = task.status === "active";
  const isPaused = task.status === "paused";
  const isPending = task.status === "pending";
  const isDone = task.status === "done";

  return (
    <div
      className="flex items-center gap-3 rounded-xl border px-3 py-2.5"
      style={{
        borderColor: "var(--border)",
        background: isActive ? "var(--surface-2)" : "var(--surface)",
      }}
    >
      <div className="min-w-0 flex-1">
        <div className="truncate text-sm font-medium">{task.title}</div>
        {task.description && (
          <div
            className="mt-0.5 line-clamp-3 whitespace-pre-wrap text-xs"
            style={{ color: "var(--text-muted)" }}
          >
            {task.description}
          </div>
        )}
        <div
          className="mt-0.5 flex flex-wrap items-center gap-x-2 gap-y-0.5 text-xs"
          style={{ color: "var(--text-muted)" }}
        >
          {task.planned_minutes != null && <span>{task.planned_minutes}m planned</span>}
          {isPending && task.remind_at && (
            <span>
              reminds{" "}
              {new Date(task.remind_at).toLocaleTimeString([], {
                hour: "2-digit",
                minute: "2-digit",
              })}
            </span>
          )}
          {(isActive || isPaused || isDone) &&
            (isActive ? (
              <TimerDigits totalSeconds={task.total_seconds} runningStartedAt={task.running_started_at} />
            ) : (
              <span className="tabular">{formatDuration(task.total_seconds)}</span>
            ))}
        </div>
      </div>

      <div className="flex shrink-0 items-center gap-1.5">
        {isPending && (
          <>
            <ActionButton variant="primary" onClick={() => onStart(task.id)}>
              Start
            </ActionButton>
            <ActionButton onClick={() => onEdit(task)}>Edit</ActionButton>
            <ActionButton variant="danger" onClick={() => onDelete(task.id)}>
              Delete
            </ActionButton>
          </>
        )}
        {isActive && (
          <>
            <ActionButton onClick={() => onEdit(task)}>Edit</ActionButton>
            <ActionButton onClick={() => onNote(task)}>Note</ActionButton>
            <ActionButton variant="neutral" onClick={() => onPause(task.id)}>
              Pause
            </ActionButton>
            <ActionButton variant="primary" onClick={() => onStop(task)}>
              Stop
            </ActionButton>
          </>
        )}
        {isPaused && (
          <>
            <ActionButton onClick={() => onEdit(task)}>Edit</ActionButton>
            <ActionButton onClick={() => onNote(task)}>Note</ActionButton>
            <ActionButton variant="primary" onClick={() => onStart(task.id)}>
              Resume
            </ActionButton>
            <ActionButton variant="neutral" onClick={() => onStop(task)}>
              Stop
            </ActionButton>
          </>
        )}
        {isDone && (
          <>
            <ActionButton onClick={() => onEdit(task)}>Edit</ActionButton>
            <ActionButton variant="danger" onClick={() => onDelete(task.id)}>
              Delete
            </ActionButton>
          </>
        )}
      </div>
    </div>
  );
}
