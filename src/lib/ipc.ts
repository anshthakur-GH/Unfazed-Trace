import { invoke } from "@tauri-apps/api/core";
import type {
  DaySummary,
  NewNoteInput,
  NewTaskInput,
  Note,
  ReviewNotesInput,
  Task,
  UpdateTaskInput,
} from "./types";

/**
 * The only way the frontend touches data. Every call is a thin wrapper around a Rust command —
 * there is no SQL, and no business logic, on this side of the IPC boundary.
 */
export const api = {
  listTasks: () => invoke<Task[]>("list_tasks"),
  createTask: (task: NewTaskInput) => invoke<Task>("create_task", { task }),
  updateTask: (task: UpdateTaskInput) => invoke<Task>("update_task", { task }),
  deleteTask: (id: number) => invoke<void>("delete_task", { id }),
  startTask: (id: number) => invoke<Task>("start_task", { id }),
  pauseTask: (id: number) => invoke<Task>("pause_task", { id }),
  completeTask: (id: number, notes: ReviewNotesInput) =>
    invoke<Task>("complete_task", { id, notes }),
  addNote: (note: NewNoteInput) => invoke<Note>("add_note", { note }),
  getDaySummary: () => invoke<DaySummary>("get_day_summary"),
  getDayReview: (date: string) => invoke<DaySummary>("get_day_review", { date }),
  listHistoryDates: () => invoke<string[]>("list_history_dates"),
};

/** Rust's `AppError` shape, surfaced as the `invoke()` rejection payload. */
export function errorMessage(err: unknown): string {
  if (err && typeof err === "object" && "message" in err) {
    return String((err as { message: unknown }).message);
  }
  return String(err);
}
