-- Reverses 20260807000005_rename_work_orders_to_tasks.up.sql, statement by
-- statement, in the opposite order.
--
-- Not a lossless round-trip: rows that only became possible under the
-- relaxed constraints (a subtask with no dates, a task with no customer, a
-- task under a parent) cannot be represented once those constraints are
-- restored, and the `title` backfill ('Sans titre') is not undone — this
-- mirrors the rest of this repo's migrations, which treat down-migrations as
-- best-effort dev tooling rather than a promise to preserve data created
-- under the new shape.

ALTER TABLE task_assignments RENAME CONSTRAINT uq_task_assignments_task_employee TO uq_assignments_work_order_employee;

ALTER INDEX idx_task_assignments_employee_id RENAME TO idx_assignments_employee_id;
ALTER INDEX idx_task_assignments_task_id RENAME TO idx_assignments_work_order_id;
ALTER INDEX idx_task_assignments_org_id RENAME TO idx_assignments_org_id;

ALTER INDEX idx_tasks_deleted_at RENAME TO idx_work_orders_deleted_at;
ALTER INDEX idx_tasks_quote_id RENAME TO idx_work_orders_quote_id;
ALTER INDEX idx_tasks_customer_context_id RENAME TO idx_work_orders_customer_context_id;
ALTER INDEX idx_tasks_customer_id RENAME TO idx_work_orders_customer_id;
ALTER INDEX idx_tasks_org_id_starts_at RENAME TO idx_work_orders_org_id_starts_at;

DROP INDEX IF EXISTS idx_tasks_parent_task_id;

ALTER TABLE tasks DROP CONSTRAINT chk_tasks_parent_not_self;
ALTER TABLE tasks DROP CONSTRAINT chk_tasks_customer_both_or_neither;
ALTER TABLE tasks DROP CONSTRAINT chk_tasks_ends_at_after_starts_at;
ALTER TABLE tasks DROP CONSTRAINT chk_tasks_dates_both_or_neither;
ALTER TABLE tasks DROP CONSTRAINT chk_tasks_root_has_dates;

ALTER TABLE tasks ALTER COLUMN customer_context_id SET NOT NULL;
ALTER TABLE tasks ALTER COLUMN customer_id SET NOT NULL;
ALTER TABLE tasks ALTER COLUMN ends_at SET NOT NULL;
ALTER TABLE tasks ALTER COLUMN starts_at SET NOT NULL;

ALTER TABLE tasks DROP COLUMN blocks_availability;
ALTER TABLE tasks DROP COLUMN parent_task_id;

ALTER TABLE tasks ALTER COLUMN title DROP NOT NULL;

ALTER TABLE task_assignments RENAME COLUMN task_id TO work_order_id;
ALTER TABLE task_assignments RENAME TO assignments;

ALTER TABLE tasks RENAME COLUMN description TO note;
ALTER TABLE tasks RENAME TO work_orders;

ALTER TYPE task_status RENAME TO work_order_status;

ALTER TABLE work_orders
    ADD CONSTRAINT chk_work_orders_title_not_blank_when_present
        CHECK (title IS NULL OR length(btrim(title)) > 0),
    ADD CONSTRAINT chk_work_orders_note_not_blank_when_present
        CHECK (note IS NULL OR length(btrim(note)) > 0),
    ADD CONSTRAINT chk_work_orders_ends_at_after_starts_at
        CHECK (ends_at > starts_at);
