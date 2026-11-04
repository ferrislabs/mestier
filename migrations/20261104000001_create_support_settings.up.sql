CREATE TABLE support_settings (
    org_id         UUID        PRIMARY KEY REFERENCES organizations(id) ON DELETE CASCADE,
    display_fields JSONB       NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
