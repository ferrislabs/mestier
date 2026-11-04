CREATE TABLE support_api_keys (
    id           UUID PRIMARY KEY,
    org_id       UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    prefix       TEXT NOT NULL UNIQUE,
    secret_hash  BYTEA NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_used_at TIMESTAMPTZ NULL,
    revoked_at   TIMESTAMPTZ NULL
);

CREATE INDEX idx_support_api_keys_org_created ON support_api_keys(org_id, created_at);
