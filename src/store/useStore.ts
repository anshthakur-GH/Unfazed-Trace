import { listen } from "@tauri-apps/api/event";
import { create } from "zustand";
import { api, errorMessage } from "../lib/ipc";
import type {
  DaySummary,
  NewTaskInput,
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
  loading: boolean;
  error: string | null;

  init: () => Promise<void>;
  refresh: () => Promise<void>;
  dismissError: () => void;
  createTask: (task: NewTaskInput) => Promise<void>;
  updateTask: (task: UpdateTaskInput) => Promise<void>;
  deleteTask: (id: number) => Promise<void>;
  startTask: (id: number) => Promise<void>;
  pauseTask: (id: number) => Promise<void>;
  completeTask: (id: number, notes: ReviewNotesInput) => Promise<void>;
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
  },

  refresh: () =>
    guarded(set, async () => {
      const [tasks, daySummary] = await Promise.all([api.listTasks(), api.getDaySummary()]);
      set({ tasks, daySummary, daySummaryFetchedAtMs: Date.now(), error: null });
    }),

  dismissError: () => set({ error: null }),

  createTask: (task) => guarded(set, async () => { await api.createTask(task); await get().refresh(); }),
  updateTask: (task) => guarded(set, async () => { await api.updateTask(task); await get().refresh(); }),
  deleteTask: (id) => guarded(set, async () => { await api.deleteTask(id); await get().refresh(); }),
  startTask: (id) => guarded(set, async () => { await api.startTask(id); await get().refresh(); }),
  pauseTask: (id) => guarded(set, async () => { await api.pauseTask(id); await get().refresh(); }),
  completeTask: (id, notes) =>
    guarded(set, async () => { await api.completeTask(id, notes); await get().refresh(); }),
}));
