-- Notes become an edit-in-place field per (task, kind) instead of an ever-growing history:
-- keep only the most recent row per (task_id, kind) -- earlier ones were superseded drafts --
-- then enforce that going forward with a unique index that add_note/complete_task upsert into.
DELETE FROM notes
WHERE id NOT IN (
  SELECT MAX(id) FROM notes GROUP BY task_id, kind
);

CREATE UNIQUE INDEX idx_notes_task_kind ON notes(task_id, kind);
