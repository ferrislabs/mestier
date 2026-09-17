ALTER TABLE automation.subscription
    ADD COLUMN trigger_id TEXT NULL;

ALTER TABLE automation.run
    ADD COLUMN trigger_id TEXT NULL;

ALTER TABLE automation.run
    DROP CONSTRAINT uq_automation_run_trigger_workflow;

ALTER TABLE automation.run
    ADD CONSTRAINT uq_automation_run_trigger_workflow
        UNIQUE (trigger_event_id, workflow_id, trigger_id);

ALTER TABLE automation.workflow
    DROP COLUMN trigger_mode;
