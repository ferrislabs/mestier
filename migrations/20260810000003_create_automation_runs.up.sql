-- One row per workflow run, and one row per step it takes. The run is
-- claimed and worked the same way `automation.delivery` is (see
-- infrastructure/automation/postgres/delivery.rs): `FOR UPDATE SKIP LOCKED`,
-- a per-organization quota, and a stale-claim release for a worker that
-- died. A run is durable and resumable: `run_step` is one row per
-- connector invocation, not one growing JSONB document, so a failure at
-- element 300 of a 500-element loop resumes at 300 instead of rewriting
-- everything already done.
CREATE TABLE automation.run (
    id                   UUID        PRIMARY KEY,
    org_id               UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    -- Denormalized on purpose: #201 needs it to compare on a primary-key
    -- join without reaching through workflow_version_id first.
    workflow_id          UUID        NOT NULL,
    -- Pinned at creation and never moved: the graph a run executes is the
    -- one frozen in this version, even if the workflow is edited (a new
    -- version saved) while the run is still going.
    workflow_version_id  UUID        NOT NULL REFERENCES automation.workflow_version(id),
    trigger_event_id     UUID        NULL,
    status               TEXT        NOT NULL,
    error                TEXT        NULL,
    next_attempt_at      TIMESTAMPTZ NULL,
    locked_at            TIMESTAMPTZ NULL,
    locked_by            TEXT        NULL,
    started_at           TIMESTAMPTZ NULL,
    finished_at          TIMESTAMPTZ NULL,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_automation_run_status
        CHECK (status IN ('pending', 'running', 'succeeded', 'failed', 'cancelled')),
    -- The idempotence argument: the same dispatch replayed never creates a
    -- second run for the same workflow. Mirrors
    -- `UNIQUE (event_id, subscription_id)` on automation.delivery. A NULL
    -- trigger_event_id (every run started by `start_run` before a trigger
    -- exists) is never equal to another NULL, so it does not constrain
    -- those runs against each other.
    CONSTRAINT uq_automation_run_trigger_workflow UNIQUE (trigger_event_id, workflow_id)
);

-- What the worker polls to find work: due, unclaimed (pending) or
-- claimed-but-not-yet-settled (running, for a claim-timeout scan) runs.
-- Partial so a finished run never pays rent on this index.
CREATE INDEX idx_automation_run_due
    ON automation.run (next_attempt_at)
    WHERE status IN ('pending', 'running');

CREATE INDEX idx_automation_run_org_created
    ON automation.run (org_id, created_at DESC);

-- One row per connector invocation. `iteration_path` is what lets a
-- connector inside a `flow.loop` execute many times while staying
-- reprensable and inspectable: empty outside a loop, `"c2[3]"` inside one,
-- `"c2[3].c5[1]"` in nested loops. Without it the unique constraint below
-- would forbid loops outright.
CREATE TABLE automation.run_step (
    id              UUID        PRIMARY KEY,
    run_id          UUID        NOT NULL REFERENCES automation.run(id) ON DELETE CASCADE,
    connector_id    TEXT        NOT NULL,
    iteration_path  TEXT        NOT NULL DEFAULT '',
    attempts        INTEGER     NOT NULL DEFAULT 0,
    status          TEXT        NOT NULL,
    input           JSONB       NULL,
    output          JSONB       NULL,
    error           TEXT        NULL,
    started_at      TIMESTAMPTZ NULL,
    finished_at     TIMESTAMPTZ NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_automation_run_step_status
        CHECK (status IN ('pending', 'in_flight', 'succeeded', 'failed', 'skipped', 'dead')),
    -- A step that already succeeded is never re-executed: resuming a run
    -- reads this row back instead of redoing the work it recorded.
    CONSTRAINT uq_automation_run_step UNIQUE (run_id, connector_id, iteration_path)
);

COMMENT ON COLUMN automation.run_step.iteration_path IS
    'Empty outside a loop. "<loop_connector_id>[<index>]" per enclosing flow.loop, outermost first, joined by ".".';
