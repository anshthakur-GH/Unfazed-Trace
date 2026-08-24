import { lazy, Suspense, useEffect, useMemo, useState } from "react";
import { ReviewDialog } from "./components/ReviewDialog";
import { TaskEditorModal } from "./components/TaskEditorModal";
import { TaskRow } from "./components/TaskRow";
import { TodayCounter } from "./components/TodayCounter";
import type { Task } from "./lib/types";
import { useStore } from "./store/useStore";

// Lazy-loaded: the review/export screen isn't needed on the widget's default cold-start path.
const TodaysReview = lazy(() => import("./screens/TodaysReview"));

function Section({ label, tasks, actions }: {
  label: string;
  tasks: Task[];
  actions: {
    onStart: (id: number) => void;
    onPause: (id: number) => void;
    onStop: (task: Task) => void;
    onEdit: (task: Task) => void;
    onDelete: (id: number) => void;
  };
}) {
  if (tasks.length === 0) return null;
  return (
    <div className="flex flex-col gap-2">
      <div className="text-xs font-medium uppercase tracking-wider" style={{ color: "var(--text-muted)" }}>
        {label}
      </div>
      {tasks.map((task) => (
        <TaskRow key={task.id} task={task} {...actions} />
      ))}
    </div>
  );
}

function App() {
  const {
    tasks,
    daySummary,
    daySummaryFetchedAtMs,
    loading,
    error,
    init,
    dismissError,
    createTask,
    updateTask,
    deleteTask,
    startTask,
    pauseTask,
    completeTask,
  } = useStore();

  // undefined = closed, null = creating a new task, Task = editing that task.
  const [editorTask, setEditorTask] = useState<Task | null | undefined>(undefined);
  const [reviewTask, setReviewTask] = useState<Task | null>(null);
  const [showTodaysReview, setShowTodaysReview] = useState(false);

  useEffect(() => {
    void init();
  }, [init]);

  const activeTasks = useMemo(() => tasks.filter((t) => t.status === "active"), [tasks]);
  const pausedTasks = useMemo(() => tasks.filter((t) => t.status === "paused"), [tasks]);
  const pendingTasks = useMemo(() => tasks.filter((t) => t.status === "pending"), [tasks]);
  const doneTasks = useMemo(() => tasks.filter((t) => t.status === "done"), [tasks]);
  const activeTask = activeTasks[0] ?? null;

  const rowActions = {
    onStart: startTask,
    onPause: pauseTask,
    onStop: setReviewTask,
    onEdit: setEditorTask,
    onDelete: deleteTask,
  };

  if (loading) {
    return <main className="flex min-h-screen items-center justify-center" />;
  }

  return (
    <main className="flex min-h-screen flex-col">
      <header
        className="flex items-start justify-between border-b px-5 pb-4 pt-6"
        style={{ borderColor: "var(--border)" }}
      >
        <TodayCounter
          totalSecondsToday={daySummary?.total_seconds_today ?? 0}
          fetchedAtMs={daySummaryFetchedAtMs}
          activeTask={activeTask}
        />
        <button
          type="button"
          onClick={() => setShowTodaysReview(true)}
          className="rounded-lg px-2 py-1 text-xs"
          style={{ color: "var(--text-muted)" }}
        >
          History
        </button>
      </header>

      {error && (
        <div
          className="mx-5 mt-3 flex items-center justify-between rounded-lg px-3 py-2 text-xs"
          style={{ background: "rgba(239,68,68,0.12)", color: "var(--danger)" }}
        >
          <span>{error}</span>
          <button type="button" onClick={dismissError} className="ml-2 font-medium">
            Dismiss
          </button>
        </div>
      )}

      {tasks.length === 0 ? (
        <section className="flex flex-1 flex-col items-center justify-center px-6 text-center">
          <div className="text-lg font-medium">Fresh start</div>
          <p className="mt-1 text-sm" style={{ color: "var(--text-muted)" }}>
            Add your first task to begin tracing your time.
          </p>
        </section>
      ) : (
        <section className="flex flex-1 flex-col gap-4 overflow-y-auto px-5 py-4">
          <Section label="Active" tasks={activeTasks} actions={rowActions} />
          <Section label="Paused" tasks={pausedTasks} actions={rowActions} />
          <Section label="Pending" tasks={pendingTasks} actions={rowActions} />
          <Section label="Done today" tasks={doneTasks} actions={rowActions} />
        </section>
      )}

      <footer className="px-5 py-4">
        <button
          type="button"
          onClick={() => setEditorTask(null)}
          className="w-full rounded-xl py-3 font-medium transition-colors"
          style={{ background: "var(--accent)", color: "var(--accent-ink)" }}
        >
          + Add task
        </button>
      </footer>

      {editorTask !== undefined && (
        <TaskEditorModal
          task={editorTask}
          onClose={() => setEditorTask(undefined)}
          onSave={(data) => {
            if (editorTask) {
              void updateTask({ id: editorTask.id, ...data });
            } else {
              void createTask(data);
            }
            setEditorTask(undefined);
          }}
        />
      )}

      {reviewTask && (
        <ReviewDialog
          task={reviewTask}
          onClose={() => setReviewTask(null)}
          onSave={(notes) => {
            void completeTask(reviewTask.id, notes);
            setReviewTask(null);
          }}
        />
      )}

      {showTodaysReview && daySummary && (
        <Suspense fallback={null}>
          <TodaysReview todaySummary={daySummary} onClose={() => setShowTodaysReview(false)} />
        </Suspense>
      )}
    </main>
  );
}

export default App;
