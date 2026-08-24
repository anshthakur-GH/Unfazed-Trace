import { useState } from "react";
import { formatDuration } from "../lib/format";
import { buildDayReviewMarkdown } from "../lib/markdown";
import type { DaySummary, NoteKind } from "../lib/types";

interface TodaysReviewProps {
  summary: DaySummary;
  onClose: () => void;
}

const NOTE_LABELS: Record<NoteKind, string> = {
  review: "What I did",
  blocker: "Blocker",
  meeting: "For next meeting",
};

/**
 * Lazy-loaded (Architecture §10): this view is only needed when the user asks for it, so it
 * shouldn't cost anything on the widget's default cold-start path.
 */
export default function TodaysReview({ summary, onClose }: TodaysReviewProps) {
  const [copied, setCopied] = useState(false);

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(buildDayReviewMarkdown(summary));
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      // Clipboard access denied or unavailable -- non-fatal, the user can still read the list.
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex flex-col" style={{ background: "var(--bg)" }}>
      <header
        className="flex items-center justify-between border-b px-5 py-4"
        style={{ borderColor: "var(--border)" }}
      >
        <div>
          <h2 className="text-base font-semibold">Today's review</h2>
          <p className="text-xs" style={{ color: "var(--text-muted)" }}>
            {summary.date} &middot; {formatDuration(summary.total_seconds_today)} tracked
          </p>
        </div>
        <button
          type="button"
          onClick={onClose}
          className="rounded-lg px-3 py-1.5 text-sm"
          style={{ color: "var(--text-muted)" }}
        >
          Close
        </button>
      </header>

      <div className="flex-1 overflow-y-auto px-5 py-4">
        {summary.done_tasks.length === 0 ? (
          <p className="text-sm" style={{ color: "var(--text-muted)" }}>
            No tasks completed yet today.
          </p>
        ) : (
          <div className="flex flex-col gap-4">
            {summary.done_tasks.map((task) => (
              <div
                key={task.id}
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
          </div>
        )}
      </div>

      <footer className="px-5 py-4">
        <button
          type="button"
          onClick={handleCopy}
          className="w-full rounded-xl py-3 font-medium transition-colors"
          style={{ background: "var(--accent)", color: "var(--accent-ink)" }}
        >
          {copied ? "Copied!" : "Copy as Markdown"}
        </button>
      </footer>
    </div>
  );
}
