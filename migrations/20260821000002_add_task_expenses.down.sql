ALTER TABLE tasks
    DROP CONSTRAINT chk_tasks_expenses_label_not_blank,
    DROP CONSTRAINT chk_tasks_expenses_label_required,
    DROP CONSTRAINT chk_tasks_expenses_not_negative;

ALTER TABLE tasks
    DROP COLUMN expenses_label,
    DROP COLUMN expenses_cents;
