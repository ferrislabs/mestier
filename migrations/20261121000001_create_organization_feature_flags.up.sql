CREATE TABLE organization_feature_flags (
    org_id     UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    flag       TEXT        NOT NULL,
    enabled_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    enabled_by UUID        NULL REFERENCES users(id) ON DELETE SET NULL,
    PRIMARY KEY (org_id, flag),
    CONSTRAINT chk_organization_feature_flags_flag_not_blank CHECK (length(btrim(flag)) > 0)
);

CREATE INDEX idx_organization_feature_flags_flag ON organization_feature_flags (flag);
