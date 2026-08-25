import { useState } from "react";
import { DayReportBody } from "../components/DayReportBody";
import { formatDayLabel, formatDuration } from "../lib/format";
import { buildDayReviewMarkdown } from "../lib/markdown";
import type { DaySummary } from "../lib/types";

interface DailyReportProps {
  summary: DaySummary;
  onClose: () => void;
}

/**
 * The once-a-day catch-up shown automatically on the first launch of a new day: "here's what
 * you did last time" (the most recent prior day with completed work). Same rendering as the
 * history browser, but framed as a dismissible morning briefing with no date navigation.
 */
export function DailyReport({ summary, onClose }: DailyReportProps) {
  const [copied, setCopied] = useState(false);

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(buildDayReviewMarkdown(summary));
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      // Clipboard access denied or unavailable -- non-fatal.
    }
  }

  return (
    <div className="fixed inset-0 z-[60] flex flex-col" style={{ background: "var(--bg)" }}>
      <header
        className="flex items-center justify-between border-b px-5 py-4"
        style={{ borderColor: "var(--border)" }}
      >
        <div>
          <div className="text-xs uppercase tracking-wider" style={{ color: "var(--text-muted)" }}>
            Last time you worked
          </div>
          <h2 className="text-base font-semibold">{formatDayLabel(summary.date)}</h2>
          <p className="text-xs" style={{ color: "var(--text-muted)" }}>
            {formatDuration(summary.total_seconds)} tracked
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
        <DayReportBody summary={summary} />
      </div>

      <footer className="flex gap-2 px-5 py-4">
        <button
          type="button"
          onClick={handleCopy}
          className="flex-1 rounded-xl py-3 font-medium transition-colors"
          style={{ background: "var(--surface-2)", color: "var(--text)" }}
        >
          {copied ? "Copied!" : "Copy as Markdown"}
        </button>
        <button
          type="button"
          onClick={onClose}
          className="flex-1 rounded-xl py-3 font-medium transition-colors"
          style={{ background: "var(--accent)", color: "var(--accent-ink)" }}
        >
          Start my day
        </button>
      </footer>
    </div>
  );
}
