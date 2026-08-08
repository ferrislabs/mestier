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
