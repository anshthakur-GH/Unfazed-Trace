export type TaskStatus = "pending" | "active" | "paused" | "done";

export interface Task {
  id: number;
  title: string;
  description: string | null;
  status: TaskStatus;
  planned_minutes: number | null;
  remind_at: string | null;
  reminder_fired: boolean;
  total_seconds: number;
  sort_order: number;
  created_at: string;
  started_at: string | null;
  completed_at: string | null;
  /** Start timestamp of the open session, present only on the one active task. */
  running_started_at: string | null;
}

export interface NewTaskInput {
  title: string;
  description: string | null;
  planned_minutes: number | null;
  remind_at: string | null;
}

export interface UpdateTaskInput extends NewTaskInput {
  id: number;
}

export type NoteKind = "review" | "meeting" | "blocker";

export interface Note {
  id: number;
  task_id: number;
  kind: NoteKind;
  body: string;
  created_at: string;
}

export interface NewNoteInput {
  task_id: number;
  kind: NoteKind;
  body: string;
}

export interface ReviewNotesInput {
  what_i_did: string | null;
  blocker: string | null;
  for_next_meeting: string | null;
}

export interface TaskWithNotes extends Task {
  notes: Note[];
}

export interface DaySummary {
  date: string;
  total_seconds: number;
  created_tasks: Task[];
  done_tasks: TaskWithNotes[];
}
