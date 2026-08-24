# Unfazed Trace

A lightweight Windows workspace companion that auto-starts with your laptop, tracks time **per task**, reminds you when a task is due, and captures a short review/meeting note when you finish — so you always have something concrete to tell the team.

## What it does

- Turns your day into a list of tasks, each with an optional time estimate and reminder
- Tracks exact time spent per task with start / pause / stop control
- Fires native Windows toast reminders (Start now / Snooze / Dismiss)
- Captures a short review note when a task is marked done
- Rolls done tasks + notes into a "Today's review" you can copy as Markdown for standup

## Stack

- **Core:** Tauri 2 (Rust)
- **Frontend:** React + Vite + TypeScript + Tailwind CSS
- **Storage:** SQLite (local-first, no account, no server)
- **Windows integration:** autostart, single-instance, system tray, native toast notifications

All time math, scheduling, and persistence live in Rust so accuracy survives the window being closed or the app being backgrounded. See [`Unfazed-Trace-Architecture.md`](./Unfazed-Trace-Architecture.md) for the full architecture, data model, state machine, theme spec, and phased build plan.

## Status

Pre-scaffold — architecture and spec are locked; implementation follows the phased plan in the architecture doc (Phase 0: Tauri scaffold → ... → Phase 7: Store submission).

## Theme

Near-black surfaces, white text, one orange accent reserved for the single primary action per screen.
