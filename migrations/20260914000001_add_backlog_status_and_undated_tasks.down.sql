-- This revert is deliberately asymmetric, and says so rather than pretending
-- otherwise. A down migration that destroys rows, or that dies on real data,
-- is not reversible; one that is honest about what it cannot undo is.
--
-- What it cannot undo: the `'BACKLOG'` label stays in the `task_status` type.
-- PostgreSQL has no `ALTER TYPE ... DROP VALUE`. Removing the label would mean
-- recreating the type and rewriting `tasks` along with every object that
-- depends on it — a cost out of all proportion to a leftover label nothing
-- reads once the Rust `TaskStatus` enum loses the variant. The paired
-- `.up.sql` therefore adds it with `IF NOT EXISTS`, so reapplying this
-- migration over the leftover succeeds.

-- Rows first, and before anything else. `TaskRow::into_task`
-- (`libs/core/src/infrastructure/task/postgres/model.rs`) parses the column
-- through `TaskStatus::from_str`, so any row still sitting in `'BACKLOG'`
-- after the Rust enum loses the variant becomes a `CoreError::Internal` on
-- every read of that task. Remapping to `'PLANNED'` loses the backlog
-- distinction — that is the point of reverting — but keeps every task
-- readable.
UPDATE tasks SET status = 'PLANNED' WHERE status = 'BACKLOG';

-- `NOT VALID` on purpose: undated roots are exactly what this migration made
-- possible, so some may exist by now. A validating `ADD CONSTRAINT` would
-- fail on them and abort the revert; `NOT VALID` restores the rule for every
-- future write while leaving the existing rows alone. Whoever wants the
-- constraint fully validated again must first give those roots dates, then
-- run `ALTER TABLE tasks VALIDATE CONSTRAINT chk_tasks_root_has_dates;`.
ALTER TABLE tasks
    ADD CONSTRAINT chk_tasks_root_has_dates
        CHECK (parent_task_id IS NOT NULL OR starts_at IS NOT NULL) NOT VALID;
