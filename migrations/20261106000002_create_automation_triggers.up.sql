CREATE TABLE automation.webhook_route (
    path        TEXT        NOT NULL,
    org_id      UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    workflow_id UUID        NOT NULL REFERENCES automation.workflow(id) ON DELETE CASCADE,
    trigger_id  TEXT        NOT NULL,
    method      TEXT        NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT uq_automation_webhook_route_path UNIQUE (path),
    CONSTRAINT uq_automation_webhook_route_trigger UNIQUE (workflow_id, trigger_id),
    CONSTRAINT chk_automation_webhook_route_method
        CHECK (method IN ('GET', 'POST', 'PUT', 'PATCH', 'DELETE')),
    CONSTRAINT chk_automation_webhook_route_path
        CHECK (
            path ~ '^[a-z0-9_-]+(/[a-z0-9_-]+)*$'
            AND path !~ '^webhooks(/|$)'
            AND char_length(path) <= 200
        )
);

CREATE INDEX idx_automation_webhook_route_org
    ON automation.webhook_route (org_id);

CREATE TABLE automation.schedule_state (
    workflow_id UUID        NOT NULL REFERENCES automation.workflow(id) ON DELETE CASCADE,
    trigger_id  TEXT        NOT NULL,
    org_id      UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    next_run_at TIMESTAMPTZ NOT NULL,
    last_run_at TIMESTAMPTZ NULL,

    PRIMARY KEY (workflow_id, trigger_id)
);

CREATE INDEX idx_automation_schedule_state_due
    ON automation.schedule_state (next_run_at);

CREATE INDEX idx_automation_schedule_state_org
    ON automation.schedule_state (org_id);
