function App() {
  return (
    <main className="flex min-h-screen flex-col">
      {/* Ambient "Today" counter — read-only placeholder until the timer engine lands (Phase 3). */}
      <header
        className="border-b px-5 pb-4 pt-6"
        style={{ borderColor: "var(--border)" }}
      >
        <div
          className="text-xs uppercase tracking-wider"
          style={{ color: "var(--text-muted)" }}
        >
          Today
        </div>
        <div className="tabular mt-1 text-3xl font-semibold">00:00:00</div>
      </header>

      {/* Friendly empty state (Architecture §6.1). */}
      <section className="flex flex-1 flex-col items-center justify-center px-6 text-center">
        <div className="text-lg font-medium">Fresh start</div>
        <p className="mt-1 text-sm" style={{ color: "var(--text-muted)" }}>
          Add your first task to begin tracing your time.
        </p>
      </section>

      <footer className="px-5 py-4">
        <button
          type="button"
          className="w-full rounded-xl py-3 font-medium transition-colors"
          style={{ background: "var(--accent)", color: "var(--accent-ink)" }}
        >
          + Add task
        </button>
      </footer>
    </main>
  );
}

export default App;
