-- A task no longer needs a specific customer_context to carry a customer:
-- "linked to this client, no particular site yet" is now a valid state,
-- distinct from "no client at all" (an internal task/meeting). A context
-- still always implies its customer — a customer_context belongs to exactly
-- one customer, so naming one without the other makes no sense.

ALTER TABLE tasks DROP CONSTRAINT chk_tasks_customer_both_or_neither;

ALTER TABLE tasks
    ADD CONSTRAINT chk_tasks_context_requires_customer
        CHECK (customer_context_id IS NULL OR customer_id IS NOT NULL);
