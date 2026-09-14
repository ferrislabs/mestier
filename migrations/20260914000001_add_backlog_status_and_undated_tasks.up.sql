-- A task can now exist with no window at all, in a `BACKLOG` status.
--
-- Status and dates are orthogonal, and stay that way: a status says how far
-- the work advanced, a window says when somebody is busy. Neither is derived
-- from the other, here or in the domain. A `CHECK ((status = 'BACKLOG') =
-- (starts_at IS NULL))` was considered and rejected: it would let a board's
-- date affordance silently rewrite the status column the user is looking at.
--
-- One file, one transaction. `ALTER TYPE ... ADD VALUE` is allowed inside a
-- transaction block on PostgreSQL 12+ (this repository pins `postgres:17` in
-- `docker-compose.yml` and in CI) as long as the new value is not *used* in
-- the same transaction. Nothing below uses it: no CHECK references it, no row
-- is backfilled to it.

-- `IF NOT EXISTS` is load-bearing, not defensive noise: the paired `.down.sql`
-- cannot remove an enum value (PostgreSQL has no `DROP VALUE`), so a
-- revert-then-reapply cycle re-runs this statement against a type that already
-- carries the label. See the down migration's own comment.
ALTER TYPE task_status ADD VALUE IF NOT EXISTS 'BACKLOG' BEFORE 'PLANNED';

-- The constraint that made an undated root impossible. It is the only one of
-- the three date constraints that goes: `chk_tasks_dates_both_or_neither` and
-- `chk_tasks_ends_at_after_starts_at` constrain the coherence of the pair, not
-- whether the pair exists, and both still hold for a backlog task (neither set)
-- exactly as they did before.
ALTER TABLE tasks DROP CONSTRAINT chk_tasks_root_has_dates;
