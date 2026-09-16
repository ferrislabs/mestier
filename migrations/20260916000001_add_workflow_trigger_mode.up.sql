ALTER TABLE automation.workflow
    ADD COLUMN trigger_mode TEXT NOT NULL DEFAULT 'events';

ALTER TABLE automation.workflow
    ADD CONSTRAINT chk_automation_workflow_trigger_mode CHECK (trigger_mode IN ('events', 'manual'));
