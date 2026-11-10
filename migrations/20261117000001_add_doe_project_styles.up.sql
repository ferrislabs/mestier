-- A project's own DOE style. It replaces the model's and the organization's
-- style for the DOE of that project, and may exist before the DOE does.
CREATE TABLE doe_project_styles (
    project_id  UUID PRIMARY KEY,
    org_id      UUID NOT NULL,
    style       JSONB NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_doe_project_styles_project FOREIGN KEY (project_id, org_id)
        REFERENCES projects(id, org_id) ON DELETE CASCADE
);

CREATE INDEX idx_doe_project_styles_org ON doe_project_styles (org_id);
