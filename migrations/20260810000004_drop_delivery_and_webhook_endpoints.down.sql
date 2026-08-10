-- A rollback against a database that has actually used this feature must
-- delete its data before it can narrow either CHECK constraint back: both
-- `chk_automation_subscription_kind` and `chk_automation_event_actor` would
-- otherwise reject exactly the rows this feature produces, and the whole
-- statement runs in one transaction, so the ALTER fails outright rather than
-- silently keeping the wider constraint. Rolling back means the feature no
-- longer exists, so neither does its data: a workflow subscription
-- (kind = 'workflow'), the runs it dispatched (trigger_event_id is only
-- ever set by that dispatch — never by `start_run`), and any event a run
-- attributed to itself (actor_kind = 'automation'). automation.run_step
-- cascades away with its run via the FK already in place; nothing else
-- references what is deleted here.
DELETE FROM automation.run WHERE trigger_event_id IS NOT NULL;

DELETE FROM automation.subscription WHERE kind = 'workflow';

DELETE FROM automation.event WHERE actor_kind = 'automation';

-- Recreates both tables empty, identical to their definitions in
-- 20260808000004 and 20260808000002 respectively — there is no data to
-- restore, only the schema.

CREATE TABLE automation.webhook_endpoint (
    id                UUID        PRIMARY KEY,
    org_id            UUID        NOT NULL REFERENCES organizations(id),
    url               TEXT        NOT NULL,
    -- Sealed with AES-256-GCM. The nonce is not sensitive and travels with the
    -- ciphertext, but it must never repeat under one key.
    secret_nonce      BYTEA       NOT NULL,
    secret_ciphertext BYTEA       NOT NULL,
    description       TEXT        NULL,
    enabled           BOOLEAN     NOT NULL DEFAULT true,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    disabled_at       TIMESTAMPTZ NULL,

    CONSTRAINT chk_automation_webhook_endpoint_url CHECK (url ~ '^https?://')
);

COMMENT ON COLUMN automation.webhook_endpoint.secret_ciphertext IS
    'Shown once at creation and never returned by the API afterwards. Rotating means regenerating, not reading.';

CREATE INDEX idx_automation_webhook_endpoint_org
    ON automation.webhook_endpoint (org_id)
    WHERE enabled;

CREATE TABLE automation.delivery (
    id              UUID        PRIMARY KEY,
    event_id        UUID        NOT NULL REFERENCES automation.event(id) ON DELETE CASCADE,
    subscription_id UUID        NOT NULL REFERENCES automation.subscription(id) ON DELETE CASCADE,
    org_id          UUID        NOT NULL REFERENCES organizations(id),
    status          TEXT        NOT NULL DEFAULT 'pending',
    attempts        INTEGER     NOT NULL DEFAULT 0,
    next_attempt_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_error      TEXT        NULL,
    locked_at       TIMESTAMPTZ NULL,
    locked_by       TEXT        NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at    TIMESTAMPTZ NULL,

    -- Not hygiene: the correctness argument. Fan-out can be replayed after a
    -- crash, or raced by a second dispatcher, and this absorbs it — so no
    -- distributed coordination is needed anywhere.
    CONSTRAINT uq_automation_delivery_event_subscription UNIQUE (event_id, subscription_id),

    CONSTRAINT chk_automation_delivery_status CHECK (
        status IN ('pending', 'in_flight', 'succeeded', 'failed', 'dead')
    )
);

-- The delivery worker's hot query: everything due, oldest first.
CREATE INDEX idx_automation_delivery_due
    ON automation.delivery (next_attempt_at)
    WHERE status IN ('pending', 'failed');

CREATE INDEX idx_automation_delivery_org_created
    ON automation.delivery (org_id, created_at DESC);

ALTER TABLE automation.subscription
    DROP CONSTRAINT chk_automation_subscription_kind;

ALTER TABLE automation.subscription
    ADD CONSTRAINT chk_automation_subscription_kind CHECK (kind IN ('webhook'));

ALTER TABLE automation.event
    DROP CONSTRAINT chk_automation_event_actor;

ALTER TABLE automation.event
    ADD CONSTRAINT chk_automation_event_actor CHECK (
        (actor_kind = 'system' AND actor_id IS NULL)
        OR (actor_kind IN ('user', 'client') AND actor_id IS NOT NULL)
    );
