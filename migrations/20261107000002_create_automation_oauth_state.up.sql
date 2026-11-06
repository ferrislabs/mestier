CREATE TABLE automation.oauth_state (
    state_hash     BYTEA       PRIMARY KEY,
    org_id         UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    credential_id  UUID        NOT NULL REFERENCES automation.credential(id) ON DELETE CASCADE,
    member_id      UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    redirect_uri   TEXT        NOT NULL,
    return_to      TEXT        NOT NULL,
    payload_nonce  BYTEA,
    payload_cipher BYTEA,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at     TIMESTAMPTZ NOT NULL,

    CONSTRAINT chk_automation_oauth_state_hash CHECK (octet_length(state_hash) = 32),
    CONSTRAINT chk_automation_oauth_state_window
        CHECK (expires_at > created_at AND expires_at <= created_at + INTERVAL '10 minutes'),
    CONSTRAINT chk_automation_oauth_state_payload
        CHECK ((payload_nonce IS NULL) = (payload_cipher IS NULL))
);

COMMENT ON TABLE automation.oauth_state IS
    'One row per authorization started, keyed by the SHA-256 of the unguessable state. Consumed by a DELETE, so a state answers once.';

CREATE INDEX idx_automation_oauth_state_expiry ON automation.oauth_state (expires_at);
CREATE INDEX idx_automation_oauth_state_credential ON automation.oauth_state (credential_id);
