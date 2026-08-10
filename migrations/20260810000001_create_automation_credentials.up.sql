CREATE TABLE automation.credential (
    id                UUID        PRIMARY KEY,
    org_id            UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    kind              TEXT        NOT NULL,
    name              TEXT        NOT NULL,
    -- 'supplied' or 'generated' — see domain::automation::credential::CredentialOrigin.
    -- No CHECK on `kind`: the valid schemes are Rust code (connector::scheme)
    -- and move with the binary, so a SQL constraint would freeze them a
    -- version behind.
    origin            TEXT        NOT NULL,
    -- Sealed with AES-256-GCM. The nonce is not sensitive and travels with
    -- the ciphertext, but it must never repeat under one key.
    data_nonce        BYTEA       NOT NULL,
    data_ciphertext   BYTEA       NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT uq_automation_credential_name UNIQUE (org_id, kind, name),
    CONSTRAINT chk_automation_credential_name CHECK (length(trim(name)) > 0),
    CONSTRAINT chk_automation_credential_origin CHECK (origin IN ('supplied', 'generated'))
);

COMMENT ON COLUMN automation.credential.data_ciphertext IS
    'Never read back by the API - only the worker that uses the credential opens it (#202).';

CREATE INDEX idx_automation_credential_org
    ON automation.credential (org_id, kind);
