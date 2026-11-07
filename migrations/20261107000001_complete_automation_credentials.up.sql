ALTER TABLE automation.credential
    ADD COLUMN description  TEXT    NOT NULL DEFAULT '',
    ADD COLUMN status       TEXT    NOT NULL DEFAULT 'valid',
    ADD COLUMN account      TEXT,
    ADD COLUMN expires_at   TIMESTAMPTZ,
    ADD COLUMN data         JSONB   NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN secret_names TEXT[]  NOT NULL DEFAULT '{}',
    ADD COLUMN fields_split BOOLEAN NOT NULL DEFAULT FALSE;

ALTER TABLE automation.credential
    ADD CONSTRAINT chk_automation_credential_description
        CHECK (char_length(description) <= 500),
    ADD CONSTRAINT chk_automation_credential_status
        CHECK (status IN ('valid', 'expired', 'not_connected')),
    ADD CONSTRAINT chk_automation_credential_data_object
        CHECK (jsonb_typeof(data) = 'object');

UPDATE automation.credential
SET secret_names = CASE kind
        WHEN 'http_header' THEN ARRAY['header_value']
        WHEN 'http_basic' THEN ARRAY['password']
        WHEN 'bearer_token' THEN ARRAY['token']
        WHEN 'odoo_api' THEN ARRAY['api_key']
        ELSE ARRAY[]::TEXT[]
    END,
    fields_split = (origin = 'generated');

COMMENT ON COLUMN automation.credential.data IS
    'Readable (non secret) fields, returned by the API. Secrets stay sealed in data_ciphertext.';

COMMENT ON COLUMN automation.credential.fields_split IS
    'False for a row written before readable fields were stored apart: the application splits it on first read.';

CREATE TABLE automation.credential_tombstone (
    id         UUID        PRIMARY KEY,
    org_id     UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name       TEXT        NOT NULL,
    deleted_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_automation_credential_tombstone_org
    ON automation.credential_tombstone (org_id);
