# Unfazed Trace — System Architecture & Build Spec

> A lightweight Windows workspace companion that auto-starts with your laptop, tracks time **per task**, reminds you when a task is due, and captures a short review/meeting note when you finish — so you always have something concrete to tell the team.

This document is written to be handed directly to **Claude Code** as the source of truth for building the app. It states the decisions an architect would make, the reasoning behind them, the data model, the exact behaviors, the theme, and a phased build plan.

---

## 0. Name

**Chosen: `Unfazed Trace`**

"Trace" says exactly what the app does — it traces where your time goes, task by task — while sounding a touch more ownable and premium than the plainer alternatives. It pairs cleanly with *Unfazed* (composed, in control) and reads well as a Store listing.

Alternates you liked, kept here for reference:
- **Unfazed Track** — the most literal "it's a tracker" choice.
- **Unfazed Log** — leans into the review/notes half (a log of what you did).

The rest of this doc uses **Unfazed Trace**.

---

## 1. Product in one line

An always-available, ultra-light Windows app that turns your day into a list of tasks, each with an optional estimate and reminder, tracks exact time spent per task with start/pause/stop control, and saves a short review note per task for your next meeting.

---

## 2. Non-negotiable requirements (the "why" behind every later choice)

These are the constraints you stated, promoted to first-class design principles:

1. **Featherweight.** Idle RAM and CPU must be near-invisible. This single constraint eliminates Electron and drives the stack choice.
2. **Auto-start with Windows, silently.** Launches on login, minimized to the system tray — no window stealing focus.
3. **Offline-first / local-first.** No account, no server needed for the MVP. Everything lives in a local database. Fast, private, works on a plane.
4. **Native Windows 11 notifications.** Reminders appear as real Windows toasts (and land in the Action Center), with action buttons.
5. **Accurate time even if the UI is closed.** Elapsed time is derived from timestamps, not a counter that only ticks while the window is open.
6. **One clear theme.** Near-black surfaces, white text, one orange action color — matching your existing timer.

---

## 3. Technology stack

### Recommendation: **Tauri 2 (Rust core) + a web frontend + SQLite**

Why this wins for *your* constraints:

- **Resource footprint.** Tauri uses the OS's built-in WebView2 instead of shipping a browser, so bundles are single-digit MB and idle memory is a fraction of Electron's. This directly satisfies requirement #1.
- **Everything you need is a first-party plugin.** Autostart, single-instance, system tray, notifications, and SQLite are official Tauri 2 plugins — you're not gluing together random libraries.
- **Store-ready.** Tauri has an official Microsoft Store distribution path (ship an NSIS `-setup.exe` that installs silently with `/S`, register it in Partner Center).
- **Great fit for Claude Code.** The UI is plain web tech (HTML/CSS/TS), which Claude Code produces reliably, and the Rust surface you need is small and plugin-driven.
- **Future-proof.** If you ever want macOS/Linux or even mobile, the same codebase extends there.

**Concrete stack:**

| Layer | Choice | Notes |
|---|---|---|
| Core / process | **Tauri 2** (Rust) | Window, tray, IPC, plugin host |
| Frontend | **React + Vite + TypeScript** | Default for reliability. *Swap to **Svelte** if you want the leanest possible bundle — either is fine; with Tauri the WebView dominates memory, not the framework.* |
| Styling | **Tailwind CSS** + CSS variables | Theme tokens in §8 |
| Local DB | **SQLite** via `tauri-plugin-sql` | Single file in app data dir |
| Autostart | `tauri-plugin-autostart` | Registers login start; start minimized |
| Single instance | `tauri-plugin-single-instance` | Re-launch focuses the existing window |
| Notifications | `tauri-plugin-notification` for simple toasts; a WinRT toast crate (e.g. a `tauri-winrt-notification`-style library) when you need **action buttons** + Action Center | See §7.2 |
| Tray | Tauri built-in tray API | Quick actions menu |
| Packaging | Tauri bundler (NSIS `-setup.exe`) | Silent install `/S` for Store |

### Alternative: **.NET (WinUI 3 / Windows App SDK) + SQLite**

Pick this **only if** you want the most native Windows integration with the least "glue," and you don't care about ever going cross-platform. WinUI 3 gives you first-class toast notifications (with buttons), `StartupTask`, tray, and MSIX packaging for the Store out of the box. The trade-off vs Tauri: heavier baseline memory and less flexible custom theming, and it's Windows-only forever.

### Decision rule (put this to yourself once)
- Care most about **lightness + custom UI + possible future platforms** → **Tauri**. (Recommended.)
- Care most about **deepest native Windows integration, Windows-only forever, minimal integration code** → **.NET WinUI 3**.

Everything below assumes **Tauri**, but the data model, feature spec, state machine, theme, and roadmap are stack-independent.

---

## 4. High-level architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Frontend (WebView2)  —  React + TS + Tailwind               │
│                                                              │
│  Screens: Widget · Task List · Task Editor · Active Task ·   │
│           Review dialog · Settings                           │
│  State store: tasks, activeSession, todayTotal               │
│  Timer view: single interval, updates only the digits        │
└───────────────▲──────────────────────────────┬──────────────┘
                │ invoke() commands             │ events (emit/listen)
                │                               ▼
┌───────────────┴──────────────────────────────────────────────┐
│  Rust core (Tauri)                                            │
│                                                              │
│  Commands:  create_task, update_task, delete_task,           │
│             start_task, pause_task, stop_task, add_note,      │
│             list_tasks, get_day_summary                       │
│                                                              │
│  Services:  ReminderScheduler (event-driven, no polling)     │
│             TimeService (session math, safety flush)         │
│             ToastService (native Windows toasts + actions)   │
│                                                              │
│  Plugins:   sql(SQLite) · autostart · single-instance ·      │
│             notification · tray                              │
└───────────────┬──────────────────────────────────────────────┘
                │
        ┌───────▼────────┐
        │   SQLite file   │   %APPDATA%\UnfazedTrace\unfazed.db
        └────────────────┘
```

**Principle:** the frontend is a thin, pretty view. All truth (time math, scheduling, persistence) lives in Rust, so accuracy survives the window being closed and the app being backgrounded.

---

## 5. Data model (SQLite schema)

```sql
CREATE TABLE tasks (
  id             INTEGER PRIMARY KEY AUTOINCREMENT,
  title          TEXT NOT NULL,
  description    TEXT,
  status         TEXT NOT NULL DEFAULT 'pending',   -- pending | active | paused | done
  planned_minutes INTEGER,                          -- optional estimate ("time assignment")
  remind_at      TEXT,                              -- optional ISO 8601 datetime for reminder
  reminder_fired INTEGER NOT NULL DEFAULT 0,        -- 0/1, so it only fires once
  total_seconds  INTEGER NOT NULL DEFAULT 0,        -- cached sum of time_sessions
  sort_order     INTEGER NOT NULL DEFAULT 0,
  created_at     TEXT NOT NULL,
  started_at     TEXT,                              -- first time it went active
  completed_at   TEXT
);

CREATE TABLE time_sessions (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  task_id    INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  started_at TEXT NOT NULL,                         -- ISO 8601
  ended_at   TEXT,                                  -- null while running
  seconds    INTEGER                                -- filled on pause/stop
);

CREATE TABLE notes (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  task_id    INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  kind       TEXT NOT NULL DEFAULT 'review',        -- review | meeting | blocker
  body       TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE app_state (
  key   TEXT PRIMARY KEY,
  value TEXT
);

CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_sessions_task ON time_sessions(task_id);
CREATE INDEX idx_notes_task ON notes(task_id);
```

**Why `time_sessions` instead of a single counter:** pausing/resuming creates multiple sessions; `total_seconds` is their sum. A running session has `ended_at = NULL`, and live elapsed = `now − started_at + total_seconds`. This makes time correct even after a crash or reopen.

---

## 6. Feature spec (MVP) — mapped to what you described

> Note on **"three options / time assignment"**: I interpreted the three core inputs per task as **(1) what the task is, (2) planned time (estimate), (3) reminder time**. Adjust if you meant something else — it doesn't change the architecture.

### 6.1 On launch
- App starts on login, minimized to tray (no focus theft).
- Opening the window shows the **list of pending tasks**. If there are none, an **empty state**: "Fresh start — add your first task."
- A small ambient **"Today"** counter (total tracked time today) sits at the top — this is the spiritual successor to your current always-on timer, but now it's the sum of real work, not just "laptop uptime."

### 6.2 Creating a task
A single add/edit form with three core fields:
1. **Task** (title, required) + optional longer description.
2. **Planned time** (optional estimate in minutes) — used later to show over/under.
3. **Remind me at** (optional date/time) — if the task hasn't started by then, a Windows toast fires.

### 6.3 Reminders
- If a task is still `pending` when `remind_at` arrives, a **native Windows toast** appears showing the task title, with buttons: **Start now**, **Snooze 10 min**, **Dismiss**.
- Reminders are **scheduled**, not polled — the scheduler computes the next fire time and sleeps until then. No per-second CPU burn.
- Once fired, `reminder_fired = 1` so it won't nag repeatedly (snooze reschedules).

### 6.4 Timing a task
- Each task has a **timer button**. Click → task goes `active`, a `time_session` opens, timer counts up.
- **Pause** and **Stop** are both available — you have full manual control.
- Only **one task active at a time** (starting task B auto-pauses task A — with a subtle confirmation). Keeps the day honest.
- The live timer updates the **digits only** (tabular numerals), not the whole UI, so it's cheap.

### 6.5 Saving + review note
- On **Stop**, a small **Review dialog** appears showing total time spent, with a note field:
  - **What I did** (review)
  - optional **Blocker / problem**
  - optional **For next meeting** (meeting note)
- Saving marks the task `done`, records `completed_at`, and stores the note(s). Done tasks show their total time and review.

### 6.6 Telling the team later (light, but included)
- A **"Today's review"** view lists each done task with its time and note.
- **Export / copy** as clean Markdown so you can paste it into your standup or chat. (This is the payoff of capturing notes — see roadmap for richer versions.)

---

## 7. Task lifecycle (state machine)

```
        create
          │
          ▼
      ┌────────┐   start    ┌────────┐  pause   ┌────────┐
      │pending │──────────▶│ active │─────────▶│ paused │
      └───┬────┘            └───┬────┘◀─────────└───┬────┘
          │                     │      resume       │
          │ remind_at hits      │ stop              │ stop
          │ & still pending     ▼                   ▼
          │                 ┌────────┐          ┌────────┐
          └──▶ TOAST        │  done  │◀─────────│  done  │
                            └────────┘          └────────┘
```

- **pending → active**: opens a session.
- **active → paused**: closes current session (writes `seconds`), keeps `total_seconds`.
- **paused → active**: opens a new session.
- **active/paused → done**: closes any open session, prompts review, sets `completed_at`.
- **Reminder** only fires from `pending`.

---

## 8. UI / UX

### 8.1 Theme tokens (from your timer screenshot: near-black, white, orange)

```css
:root {
  --bg:          #0E0F11;  /* app background, near-black charcoal */
  --surface:     #17181B;  /* cards / panels */
  --surface-2:   #202226;  /* raised elements, inputs */
  --border:      #2A2C31;
  --text:        #FFFFFF;  /* primary text / timer digits */
  --text-muted:  #9AA0A6;  /* secondary labels, disabled (like "Save") */
  --accent:      #F97316;  /* orange — primary action (like "Pause") */
  --accent-hover:#FB8B3C;
  --accent-ink:  #111214;  /* text on orange, for AA contrast */
  --danger:      #EF4444;
  --success:     #22C55E;
  --radius:      12px;     /* pill-ish buttons ~8–10px */
}
```

- **Typography:** `Inter` for UI. For the timer digits use **tabular / monospaced numerals** (Inter's `font-variant-numeric: tabular-nums`, or a mono like `JetBrains Mono`) so digits don't jitter as they change.
- **Orange is rare on purpose:** it marks the single primary action on any screen (Start / Pause). Everything else is grayscale. This is what makes the screenshot feel clean.

### 8.2 Screens

1. **Widget (default, compact).** Small always-handy window / tray popover: Today total, the active task with big timer + Pause/Stop, and a short list of next pending tasks with quick-start buttons.
2. **Task list (main).** Grouped: **Active**, **Pending**, **Done today**. Big "+ Add task" button.
3. **Task editor (modal).** Title, description, planned time, remind-at.
4. **Active task view.** Large timer, Pause/Stop, and a live scratch note you can jot into while working.
5. **Review dialog (on Stop).** Total time + note fields (§6.5).
6. **Today's review.** List for standup + copy-as-Markdown.
7. **Settings.** Start on login toggle, default reminder lead time, theme (locked to your palette for now), data location / export.
8. **Reminder toast (OS-level).** Title + Start now / Snooze / Dismiss.
9. **Tray menu.** Open, Start last task, Pause current, Today's review, Quit.

### 8.3 Key micro-behaviors
- Closing the window **minimizes to tray**, it does not quit (instant reopen, stays light).
- Starting a second task shows a one-line "Task A paused" toast in-app.
- Empty states are friendly, never blank.

---

## 9. Windows integration specifics

### 9.1 Autostart, silently
- Use `tauri-plugin-autostart` to register login start. Launch **minimized to tray** so it never interrupts you. (For the Store/MSIX path, use the platform StartupTask mechanism.)

### 9.2 Native toasts with action buttons
- `tauri-plugin-notification` covers simple toasts. For **buttons** (Start now / Snooze) and reliable **Action Center** presence, use a WinRT toast library from the Rust side. Toasts must reference a registered AppUserModelID (your app identity) to display correctly on Windows.

### 9.3 Single instance
- `tauri-plugin-single-instance`: if the app is already running and the exe is launched again (or user clicks the shortcut), focus/show the existing window instead of spawning a second process.

### 9.4 System tray
- Tray icon with the quick-actions menu (§8.2 #9). Left-click opens the widget; right-click opens the menu.

### 9.5 Persistence location
- SQLite file at `%APPDATA%\UnfazedTrace\unfazed.db`. Migrations run on startup.

---

## 10. Performance budget (how "light" is defined and enforced)

| Metric | Target |
|---|---|
| Installed size | < 15 MB |
| Cold start to tray | < 1 s |
| Idle RAM | as low as the shared WebView2 allows (tens of MB, not hundreds) |
| Idle CPU | ~0% (no active timer = no work) |
| Active-timer CPU | negligible (one 1s interval updating a text node) |

**How to actually hit it:**
- **No polling anywhere.** The reminder scheduler sleeps until the next `remind_at`. The timer interval runs **only while a task is active**.
- **Update digits, not the tree.** Keep elapsed in a store; the timer component re-renders a single `<span>`, not the list.
- **Derive time from timestamps.** Live elapsed = `now − session.started_at + total_seconds`. Never trust a counter that pauses when the window is hidden.
- **Safety flush.** While a session runs, write progress to `time_sessions` every ~30–60s so a hard crash loses at most a minute.
- **Minimize-to-tray, don't relaunch.** Reopening is instant and avoids repeated cold starts.
- **Lazy-load** the heavier screens (review/export) so the widget path stays tiny.

---

## 11. Build & distribution to the Microsoft Store

1. Enroll a **Microsoft developer account** (individual or company) in Partner Center.
2. Generate icons with `tauri icon` from one PNG/SVG (includes Store sizes).
3. Build the **NSIS `-setup.exe`** installer. The Store requires **silent install** — NSIS supports this with the uppercase **`/S`** flag; enter `/S` as the silent-install argument when registering the product.
4. In Partner Center: **New Product → EXE/MSI app → reserve the name** (`Unfazed Trace`). The Store entry links to your hosted installer.
5. **Code sign** the installer (recommended) to reduce SmartScreen warnings. You can *also* distribute the signed `.exe` directly from your own site/GitHub for people who don't use the Store.
6. WebView2 runtime is preinstalled on Windows 11; the installer can auto-download it on older Windows 10 if needed.

---

## 12. Roadmap (post-MVP)

Kept out of the MVP deliberately, but designed for:

- **Rich standup export** — daily/weekly review as Markdown or PDF, grouped by project, one click to copy.
- **Projects / tags** and filtering.
- **Recurring tasks** and templates.
- **Pomodoro / focus mode** built on the same timer engine.
- **Global hotkey** to start/pause the active task from anywhere.
- **Estimate vs actual analytics** (you already store `planned_minutes` and `total_seconds`).
- **Optional cloud sync** for multi-device (only if you actually need it — it's the one thing that would add a backend).
- **Team share** — export/share a review, later maybe a lightweight shared board.

---

## 13. Suggested build phases for Claude Code

Feed these to Claude Code roughly one phase at a time. Each is independently testable.

**Phase 0 — Scaffold.** Create a Tauri 2 app named **Unfazed Trace** with React + Vite + TypeScript + Tailwind. Add and wire the plugins: `sql`, `autostart`, `single-instance`, `notification`, and the tray. App launches, shows an empty window, has a tray icon, and only ever runs one instance.

**Phase 1 — Data layer.** Create the SQLite schema (§5) with a migration runner. Implement Rust commands: `create_task`, `update_task`, `delete_task`, `list_tasks`, `add_note`, `get_day_summary`. Unit-test the time math.

**Phase 2 — Task list + theme.** Build the main list (Active / Pending / Done today), the add/edit modal, and apply the theme tokens (§8.1). No timing yet.

**Phase 3 — Timer engine.** Implement `start_task`, `pause_task`, `stop_task` using `time_sessions`. Live elapsed derived from timestamps. One active task at a time (auto-pause others). Digits-only re-render + safety flush.

**Phase 4 — Review flow.** Stop → review dialog → save note(s), mark done, show total time.

**Phase 5 — Reminders + toasts.** Event-driven scheduler; native toast with Start now / Snooze / Dismiss; `reminder_fired` handling.

**Phase 6 — Widget + autostart.** Compact widget / tray popover; minimize-to-tray on close; start on login minimized.

**Phase 7 — Ship.** Today's-review export (copy as Markdown), performance pass against §10 budget, build NSIS installer, code sign, submit to the Store.

---

## 14. Decisions to confirm before Claude Code starts

1. **Stack:** Tauri (recommended) vs .NET WinUI 3. *(§3)*
2. **Frontend framework:** React (default) vs Svelte (leanest). *(§3)*
3. **"Time assignment" meaning:** did you mean an **estimate** (planned minutes), a **scheduled reminder time**, or **both**? The spec currently supports both. *(§6.2)*
4. **Keep a global "day" timer** like your current widget (ambient total tracked today)? Currently included as read-only ambient info. *(§6.1)*
5. **Multi-device sync** ever needed? If yes someday, we keep the data layer sync-friendly now; the MVP stays fully local either way. *(§12)*
