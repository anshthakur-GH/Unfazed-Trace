import { useEffect, useMemo, useState } from "react";
import { api, errorMessage } from "../lib/ipc";
import { formatDuration } from "../lib/format";
import { buildDayReviewMarkdown } from "../lib/markdown";
import type { DaySummary, NoteKind } from "../lib/types";

interface TodaysReviewProps {
  /** Live "today" summary from the main store — used as-is while browsing today. */
  todaySummary: DaySummary;
  onClose: () => void;
}

const NOTE_LABELS: Record<NoteKind, string> = {
  review: "What I did",
  blocker: "Blocker",
  meeting: "For next meeting",
};

function formatDateLabel(date: string, isToday: boolean): string {
  if (isToday) return "Today";
  return new Date(`${date}T00:00:00`).toLocaleDateString(undefined, {
    weekday: "short",
    month: "short",
    day: "numeric",
  });
}

/**
 * Lazy-loaded (Architecture §10): this view is only needed when the user asks for it, so it
 * shouldn't cost anything on the widget's default cold-start path.
 *
 * Doubles as the history browser: every completed task and its notes are already permanent in
 * SQLite (`tasks`/`notes`) -- this view just adds a way to step back through past dates instead
 * of only ever showing today.
 */
export default function TodaysReview({ todaySummary, onClose }: TodaysReviewProps) {
  const [historyDates, setHistoryDates] = useState<string[]>([]);
  // null = viewing today (the live summary from the parent); otherwise a "YYYY-MM-DD" string.
  const [selectedDate, setSelectedDate] = useState<string | null>(null);
  const [historicalSummary, setHistoricalSummary] = useState<DaySummary | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    api.listHistoryDates().then(setHistoryDates).catch((err) => setError(errorMessage(err)));
  }, []);

  useEffect(() => {
    if (selectedDate === null) return;
    setLoading(true);
    setError(null);
    api
      .getDayReview(selectedDate)
      .then(setHistoricalSummary)
      .catch((err) => setError(errorMessage(err)))
      .finally(() => setLoading(false));
  }, [selectedDate]);

  // Past dates only -- "today" is reached via `selectedDate === null`, not as a list entry,
  // so it always reflects the live, currently-ticking summary rather than a stale fetch.
  const pastDates = useMemo(
    () => historyDates.filter((d) => d !== todaySummary.date),
    [historyDates, todaySummary.date],
  );

  const currentIndex = selectedDate === null ? -1 : pastDates.indexOf(selectedDate);
  const canGoOlder = selectedDate === null ? pastDates.length > 0 : currentIndex < pastDates.length - 1;
  const canGoNewer = selectedDate !== null;

  function goOlder() {
    if (!canGoOlder) return;
    setSelectedDate(selectedDate === null ? pastDates[0] : pastDates[currentIndex + 1]);
  }
  function goNewer() {
    if (!canGoNewer) return;
    setSelectedDate(currentIndex <= 0 ? null : pastDates[currentIndex - 1]);
  }

  const summary = selectedDate === null ? todaySummary : historicalSummary;
  const isToday = selectedDate === null;

  async function handleCopy() {
    if (!summary) return;
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
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={goOlder}
            disabled={!canGoOlder}
            className="rounded-lg px-2 py-1 text-sm disabled:opacity-30"
            style={{ color: "var(--text-muted)" }}
            aria-label="Older day"
          >
            &larr;
          </button>
          <div>
            <h2 className="text-base font-semibold">{formatDateLabel(summary?.date ?? todaySummary.date, isToday)}</h2>
            <p className="text-xs" style={{ color: "var(--text-muted)" }}>
              {formatDuration(summary?.total_seconds_today ?? 0)} tracked
            </p>
          </div>
          <button
            type="button"
            onClick={goNewer}
            disabled={!canGoNewer}
            className="rounded-lg px-2 py-1 text-sm disabled:opacity-30"
            style={{ color: "var(--text-muted)" }}
            aria-label="Newer day"
          >
            &rarr;
          </button>
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
        {error && (
          <p className="mb-3 text-xs" style={{ color: "var(--danger)" }}>
            {error}
          </p>
        )}
        {loading ? null : summary && summary.done_tasks.length === 0 ? (
          <p className="text-sm" style={{ color: "var(--text-muted)" }}>
            No tasks completed on this day.
          </p>
        ) : (
          <div className="flex flex-col gap-4">
            {summary?.done_tasks.map((task) => (
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
          disabled={!summary}
          className="w-full rounded-xl py-3 font-medium transition-colors disabled:opacity-50"
          style={{ background: "var(--accent)", color: "var(--accent-ink)" }}
        >
          {copied ? "Copied!" : "Copy as Markdown"}
        </button>
      </footer>
    </div>
  );
}
