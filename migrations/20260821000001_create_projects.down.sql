-- A plain drop, and that is the point.
--
-- The up migration copied `customer_id`, `customer_context_id` and `quote_id`
-- onto the new projects without removing them from `tasks`, so there is nothing
-- to restore here: the pre-migration state is still sitting in the columns it
-- always occupied.

ALTER TABLE tasks DROP CONSTRAINT fk_tasks_project;
DROP INDEX idx_tasks_project_id;
ALTER TABLE tasks DROP COLUMN project_id;

DROP TABLE projects;
