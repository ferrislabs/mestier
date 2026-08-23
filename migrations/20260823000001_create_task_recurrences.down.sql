DROP INDEX IF EXISTS uq_tasks_recurrence_occurrence;

ALTER TABLE tasks DROP CONSTRAINT IF EXISTS fk_tasks_recurrence;
ALTER TABLE tasks DROP COLUMN IF EXISTS occurrence_date;
ALTER TABLE tasks DROP COLUMN IF EXISTS recurrence_id;

DROP TABLE IF EXISTS task_recurrences;
DROP TYPE IF EXISTS task_recurrence_frequency;
