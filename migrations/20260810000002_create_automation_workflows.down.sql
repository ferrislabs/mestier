ALTER TABLE automation.workflow
    DROP CONSTRAINT IF EXISTS fk_automation_workflow_current_version;

DROP TABLE IF EXISTS automation.workflow_version;
DROP TABLE IF EXISTS automation.workflow;
