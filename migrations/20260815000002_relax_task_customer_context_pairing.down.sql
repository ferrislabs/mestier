ALTER TABLE tasks DROP CONSTRAINT chk_tasks_context_requires_customer;

ALTER TABLE tasks
    ADD CONSTRAINT chk_tasks_customer_both_or_neither
        CHECK ((customer_id IS NULL) = (customer_context_id IS NULL));
