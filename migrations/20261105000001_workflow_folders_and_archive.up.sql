CREATE TABLE automation.workflow_folder (
    id         UUID        PRIMARY KEY,
    org_id     UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    parent_id  UUID        NULL,
    name       TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT fk_automation_workflow_folder_parent
        FOREIGN KEY (parent_id) REFERENCES automation.workflow_folder(id) ON DELETE RESTRICT,
    CONSTRAINT chk_automation_workflow_folder_name
        CHECK (char_length(btrim(name)) BETWEEN 1 AND 80)
);

CREATE UNIQUE INDEX uq_automation_workflow_folder_name
    ON automation.workflow_folder (org_id, parent_id, lower(name))
    NULLS NOT DISTINCT;

CREATE INDEX idx_automation_workflow_folder_parent
    ON automation.workflow_folder (parent_id)
    WHERE parent_id IS NOT NULL;

ALTER TABLE automation.workflow
    ADD COLUMN folder_id   UUID        NULL,
    ADD COLUMN tags        TEXT[]      NOT NULL DEFAULT '{}',
    ADD COLUMN archived_at TIMESTAMPTZ NULL;

ALTER TABLE automation.workflow
    ADD CONSTRAINT fk_automation_workflow_folder
        FOREIGN KEY (folder_id) REFERENCES automation.workflow_folder(id) ON DELETE RESTRICT,
    ADD CONSTRAINT chk_automation_workflow_tags
        CHECK (cardinality(tags) <= 10),
    ADD CONSTRAINT chk_automation_workflow_archived_disabled
        CHECK (archived_at IS NULL OR NOT enabled);

CREATE INDEX idx_automation_workflow_folder
    ON automation.workflow (folder_id)
    WHERE folder_id IS NOT NULL;
