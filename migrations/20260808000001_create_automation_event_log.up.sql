CREATE SCHEMA IF NOT EXISTS automation;

CREATE TABLE automation.event (
    id             UUID        PRIMARY KEY,
    org_id         UUID        NOT NULL REFERENCES organizations(id),
    name           TEXT        NOT NULL,
    version        INTEGER     NOT NULL,
    subject_kind   TEXT        NOT NULL,
    subject_id     UUID        NULL,
    payload        JSONB       NOT NULL,
    actor_kind     TEXT        NOT NULL,
    actor_id       UUID        NULL,
    correlation_id UUID        NULL,
    occurred_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    dispatched_at  TIMESTAMPTZ NULL,

    -- Mirrors the `events::Actor` enum: only a system actor has no identity.
    -- The invariant is enforced on both sides so a hand-written INSERT cannot
    -- create a row the Rust type could not have produced.
    CONSTRAINT chk_automation_event_actor CHECK (
        (actor_kind = 'system' AND actor_id IS NULL)
        OR (actor_kind IN ('user', 'client') AND actor_id IS NOT NULL)
    ),

    CONSTRAINT chk_automation_event_version CHECK (version > 0)
);

-- `version` is INTEGER, not SMALLINT: the Rust side is a `u16`, and `u16` into
-- `i32` is infallible while `u16` into `i16` is not. Two bytes per row is a
-- cheaper price than a conversion that can fail.

COMMENT ON TABLE automation.event IS
    'Append-only log of domain events. Only dispatched_at is ever updated.';

CREATE INDEX idx_automation_event_org_occurred
    ON automation.event (org_id, occurred_at DESC);

CREATE INDEX idx_automation_event_org_name
    ON automation.event (org_id, name);

CREATE INDEX idx_automation_event_subject
    ON automation.event (subject_kind, subject_id);

-- The dispatcher's hot query: everything not yet fanned out, oldest first.
CREATE INDEX idx_automation_event_undispatched
    ON automation.event (occurred_at)
    WHERE dispatched_at IS NULL;
