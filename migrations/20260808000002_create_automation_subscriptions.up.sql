CREATE TABLE automation.subscription (
    id          UUID        PRIMARY KEY,
    org_id      UUID        NOT NULL REFERENCES organizations(id),
    kind        TEXT        NOT NULL,
    target_id   UUID        NOT NULL,
    event_names TEXT[]      NOT NULL,
    enabled     BOOLEAN     NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- 'webhook' today. The workflow engine becomes a value here and nothing
    -- else: if the dispatcher is right, it reinvents no plumbing.
    CONSTRAINT chk_automation_subscription_kind CHECK (kind IN ('webhook')),
    CONSTRAINT chk_automation_subscription_event_names CHECK (cardinality(event_names) > 0)
);

CREATE INDEX idx_automation_subscription_org_enabled
    ON automation.subscription (org_id)
    WHERE enabled;

CREATE INDEX idx_automation_subscription_event_names
    ON automation.subscription USING GIN (event_names);

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
