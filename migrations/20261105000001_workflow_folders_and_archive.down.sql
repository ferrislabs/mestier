DROP INDEX IF EXISTS automation.idx_automation_workflow_folder;

ALTER TABLE automation.workflow
    DROP CONSTRAINT IF EXISTS chk_automation_workflow_archived_disabled,
    DROP CONSTRAINT IF EXISTS chk_automation_workflow_tags,
    DROP CONSTRAINT IF EXISTS fk_automation_workflow_folder;

ALTER TABLE automation.workflow
    DROP COLUMN IF EXISTS archived_at,
    DROP COLUMN IF EXISTS tags,
    DROP COLUMN IF EXISTS folder_id;

DROP TABLE IF EXISTS automation.workflow_folder;
