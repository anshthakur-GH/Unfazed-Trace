import { formatDuration } from "./format";
import type { DaySummary, NoteKind, TaskWithNotes } from "./types";

const NOTE_LABELS: Record<NoteKind, string> = {
  review: "What I did",
  blocker: "Blocker",
  meeting: "For next meeting",
};

function taskSection(task: TaskWithNotes): string {
  const lines = [`## ${task.title} (${formatDuration(task.total_seconds)})`];
  for (const note of task.notes) {
    lines.push(`**${NOTE_LABELS[note.kind]}:** ${note.body}`);
  }
  return lines.join("\n");
}

/** Renders a day's done tasks + notes as clean Markdown, ready to paste into standup or chat. */
export function buildDayReviewMarkdown(summary: DaySummary): string {
  const header = `# Today's review — ${summary.date}\n\nTotal time tracked: ${formatDuration(summary.total_seconds_today)}`;
  if (summary.done_tasks.length === 0) {
    return `${header}\n\nNo tasks completed yet today.`;
  }
  const sections = summary.done_tasks.map(taskSection);
  return [header, ...sections].join("\n\n");
}
