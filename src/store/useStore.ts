import { listen } from "@tauri-apps/api/event";
import { create } from "zustand";
import { api, errorMessage } from "../lib/ipc";
import type {
  DaySummary,
  NewTaskInput,
  NoteKind,
  ReviewNotesInput,
  Task,
  UpdateTaskInput,
} from "../lib/types";

interface StoreState {
  tasks: Task[];
  daySummary: DaySummary | null;
  /** When `daySummary` was last fetched — lets the UI back out its live-session
   *  contribution and re-derive a fresh tick without re-fetching every second. */
  daySummaryFetchedAtMs: number;
  /** The once-a-day catch-up report, if one is pending on this launch (null otherwise). */
  dailyReport: DaySummary | null;
  loading: boolean;
  error: string | null;

  init: () => Promise<void>;
  refresh: () => Promise<void>;
  dismissError: () => void;
  dismissDailyReport: () => void;
  createTask: (task: NewTaskInput) => Promise<void>;
  updateTask: (task: UpdateTaskInput) => Promise<void>;
  deleteTask: (id: number) => Promise<void>;
  startTask: (id: number) => Promise<void>;
  pauseTask: (id: number) => Promise<void>;
  completeTask: (id: number, notes: ReviewNotesInput) => Promise<void>;
  /** Jot progress notes on a task without touching its status or timer — used mid-task, unlike
   *  `completeTask` which stops the task. Saves one note per non-empty field. */
  addTaskNotes: (id: number, notes: ReviewNotesInput) => Promise<void>;
}

async function guarded(set: (partial: Partial<StoreState>) => void, fn: () => Promise<void>) {
  try {
    await fn();
  } catch (err) {
    set({ error: errorMessage(err) });
  }
}

export const useStore = create<StoreState>((set, get) => ({
  tasks: [],
  daySummary: null,
  daySummaryFetchedAtMs: Date.now(),
  dailyReport: null,
  loading: true,
  error: null,

  init: async () => {
    await get().refresh();
    set({ loading: false });
    // Reminder toasts (Start now / Snooze) and any other window mutate state outside this
    // window's own action calls; this event is how we learn about those without polling.
    await listen("state-changed", () => {
      void get().refresh();
    });
    // Once-a-day catch-up: on the first launch of a new day, surface the previous working
    // day's report and bring the (possibly autostart-hidden) window forward. The command
    // self-gates to once per calendar day, so this is safe to call on every launch.
    try {
      const report = await api.getPendingDailyReport();
      if (report) {
        set({ dailyReport: report });
        await api.revealWindow();
      }
    } catch {
      // A missing/failed report must never block normal startup.
    }
  },

  refresh: () =>
    guarded(set, async () => {
      const [tasks, daySummary] = await Promise.all([api.listTasks(), api.getDaySummary()]);
      set({ tasks, daySummary, daySummaryFetchedAtMs: Date.now(), error: null });
    }),

  dismissError: () => set({ error: null }),
  dismissDailyReport: () => set({ dailyReport: null }),

  createTask: (task) => guarded(set, async () => { await api.createTask(task); await get().refresh(); }),
  updateTask: (task) => guarded(set, async () => { await api.updateTask(task); await get().refresh(); }),
  deleteTask: (id) => guarded(set, async () => { await api.deleteTask(id); await get().refresh(); }),
  startTask: (id) => guarded(set, async () => { await api.startTask(id); await get().refresh(); }),
  pauseTask: (id) => guarded(set, async () => { await api.pauseTask(id); await get().refresh(); }),
  completeTask: (id, notes) =>
    guarded(set, async () => { await api.completeTask(id, notes); await get().refresh(); }),
  addTaskNotes: (id, notes) =>
    guarded(set, async () => {
      const entries: [NoteKind, string | null][] = [
        ["review", notes.what_i_did],
        ["blocker", notes.blocker],
        ["meeting", notes.for_next_meeting],
      ];
      for (const [kind, body] of entries) {
        if (body) {
          await api.addNote({ task_id: id, kind, body });
        } else {
          // Emptying a field that had a previous note clears it, rather than leaving a stale
          // saved value the (now-blank) field no longer reflects.
          await api.deleteTaskNote(id, kind);
        }
      }
      await get().refresh();
    }),
}));
