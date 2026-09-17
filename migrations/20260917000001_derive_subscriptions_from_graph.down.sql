ALTER TABLE automation.workflow
    ADD COLUMN trigger_mode TEXT NOT NULL DEFAULT 'events';

ALTER TABLE automation.workflow
    ADD CONSTRAINT chk_automation_workflow_trigger_mode CHECK (trigger_mode IN ('events', 'manual'));

ALTER TABLE automation.run
    DROP CONSTRAINT uq_automation_run_trigger_workflow;

ALTER TABLE automation.run
    ADD CONSTRAINT uq_automation_run_trigger_workflow
        UNIQUE (trigger_event_id, workflow_id);

ALTER TABLE automation.run
    DROP COLUMN trigger_id;

ALTER TABLE automation.subscription
    DROP COLUMN trigger_id;
