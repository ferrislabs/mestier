-- Rename work orders into tasks.
--
-- The planning module now models the generic "task" (a meeting, a trip, a
-- training session), of which a "chantier" is simply the special case that
-- carries a customer. See the planning module design doc
-- (2026-08-07-planning-tasks-remodel-design.md), which supersedes
-- 2026-08-07-planning-module-design.md for everything concerning
-- `work_orders`.
--
-- Existing-data reconciliation:
--  - `note` is renamed to `description`, keeping its content — the closest
--    semantic match.
--  - `title`, nullable until now, becomes NOT NULL: existing rows with no
--    title are backfilled with 'Sans titre' rather than dropped or left
--    blank.
--
-- New, in the same migration:
--  - `parent_task_id`, a self-reference enabling a two-level hierarchy. The
--    schema stays recursive on purpose; the two-level limit is enforced in
--    the domain (`validate_parent_depth`), not here, so lifting it later is
--    a validation-rule change, not a migration.
--  - `blocks_availability`, declared at creation — only tasks that carry it
--    feed `detect_conflicts`.
--  - `starts_at`/`ends_at` become nullable: NULL on a subtask means "inherit
--    the parent's window" (resolved at read time by `resolve_task_window`,
--    never copied down at creation, which would drift silently once the
--    parent is rescheduled). A root always carries its own dates — enforced
--    by `chk_tasks_root_has_dates` below.
--  - `customer_id`/`customer_context_id` become nullable, together: a task
--    without a customer is simply not a "chantier".

ALTER TYPE work_order_status RENAME TO task_status;

ALTER TABLE work_orders RENAME TO tasks;
ALTER TABLE tasks RENAME COLUMN note TO description;

ALTER TABLE assignments RENAME TO task_assignments;
ALTER TABLE task_assignments RENAME COLUMN work_order_id TO task_id;

UPDATE tasks SET title = 'Sans titre' WHERE title IS NULL;
ALTER TABLE tasks ALTER COLUMN title SET NOT NULL;

ALTER TABLE tasks ADD COLUMN parent_task_id UUID NULL REFERENCES tasks(id) ON DELETE CASCADE;
ALTER TABLE tasks ADD COLUMN blocks_availability BOOLEAN NOT NULL DEFAULT true;

ALTER TABLE tasks ALTER COLUMN starts_at DROP NOT NULL;
ALTER TABLE tasks ALTER COLUMN ends_at DROP NOT NULL;
ALTER TABLE tasks ALTER COLUMN customer_id DROP NOT NULL;
ALTER TABLE tasks ALTER COLUMN customer_context_id DROP NOT NULL;

ALTER TABLE tasks DROP CONSTRAINT chk_work_orders_ends_at_after_starts_at;
ALTER TABLE tasks DROP CONSTRAINT chk_work_orders_note_not_blank_when_present;
ALTER TABLE tasks DROP CONSTRAINT chk_work_orders_title_not_blank_when_present;

ALTER TABLE tasks
    ADD CONSTRAINT chk_tasks_root_has_dates
        CHECK (parent_task_id IS NOT NULL OR starts_at IS NOT NULL),
    ADD CONSTRAINT chk_tasks_dates_both_or_neither
        CHECK ((starts_at IS NULL) = (ends_at IS NULL)),
    ADD CONSTRAINT chk_tasks_ends_at_after_starts_at
        CHECK (starts_at IS NULL OR ends_at > starts_at),
    ADD CONSTRAINT chk_tasks_customer_both_or_neither
        CHECK ((customer_id IS NULL) = (customer_context_id IS NULL)),
    ADD CONSTRAINT chk_tasks_parent_not_self
        CHECK (parent_task_id IS NULL OR parent_task_id <> id);

CREATE INDEX idx_tasks_parent_task_id ON tasks(parent_task_id);

-- Cosmetic renames: table/type renames do not cascade to the names of their
-- indexes or constraints in Postgres.
ALTER INDEX idx_work_orders_org_id_starts_at RENAME TO idx_tasks_org_id_starts_at;
ALTER INDEX idx_work_orders_customer_id RENAME TO idx_tasks_customer_id;
ALTER INDEX idx_work_orders_customer_context_id RENAME TO idx_tasks_customer_context_id;
ALTER INDEX idx_work_orders_quote_id RENAME TO idx_tasks_quote_id;
ALTER INDEX idx_work_orders_deleted_at RENAME TO idx_tasks_deleted_at;

ALTER INDEX idx_assignments_org_id RENAME TO idx_task_assignments_org_id;
ALTER INDEX idx_assignments_work_order_id RENAME TO idx_task_assignments_task_id;
ALTER INDEX idx_assignments_employee_id RENAME TO idx_task_assignments_employee_id;

ALTER TABLE task_assignments RENAME CONSTRAINT uq_assignments_work_order_employee TO uq_task_assignments_task_employee;
