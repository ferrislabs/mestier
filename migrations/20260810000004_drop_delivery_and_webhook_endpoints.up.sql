-- #201 wires the durable event backbone (chantier A) onto the workflow
-- engine (chantier B) and retires the path it replaces: an event now fans
-- out into automation.run rows instead of automation.delivery rows, and a
-- workflow is triggered the same way a webhook subscription used to be.
--
-- No API has ever existed to create a webhook endpoint (#203 adds one
-- later), so no production row can exist in automation.webhook_endpoint —
-- this is a DROP, not a conversion with data to carry forward.

-- Actor::Automation marks a write made by the run engine itself, carrying
-- the run id that made it. Extends chk_automation_event_actor rather than
-- replacing its existing cases (system with no id, user/client with one),
-- which are untouched from 20260808000001 — already applied everywhere, so
-- edited here instead.
ALTER TABLE automation.event
    DROP CONSTRAINT chk_automation_event_actor;

ALTER TABLE automation.event
    ADD CONSTRAINT chk_automation_event_actor CHECK (
        (actor_kind = 'system' AND actor_id IS NULL)
        OR (actor_kind IN ('user', 'client', 'automation') AND actor_id IS NOT NULL)
    );

-- The extension point 20260808000002 reserved: a subscription can now point
-- at a workflow instead of a webhook endpoint, with target_id then carrying
-- a workflow_id. Edited here for the same reason as the actor check above.
ALTER TABLE automation.subscription
    DROP CONSTRAINT chk_automation_subscription_kind;

ALTER TABLE automation.subscription
    ADD CONSTRAINT chk_automation_subscription_kind CHECK (kind IN ('webhook', 'workflow'));

DROP TABLE automation.delivery;

DROP TABLE automation.webhook_endpoint;
