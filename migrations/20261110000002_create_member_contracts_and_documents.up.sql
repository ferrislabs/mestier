CREATE EXTENSION IF NOT EXISTS btree_gist;

CREATE TABLE member_contracts (
    id              UUID        PRIMARY KEY,
    org_id          UUID        NOT NULL REFERENCES organizations(id),
    member_id       UUID        NOT NULL,
    type            TEXT        NOT NULL,
    start_date      DATE        NOT NULL,
    end_date        DATE        NULL,
    trial_end_date  DATE        NULL,
    schedule        JSONB       NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT fk_member_contracts_member
        FOREIGN KEY (member_id, org_id) REFERENCES organization_members(id, organization_id),
    CONSTRAINT chk_member_contracts_type
        CHECK (type IN ('CDI', 'CDD', 'APPRENTICESHIP', 'INTERIM', 'MANAGER')),
    CONSTRAINT chk_member_contracts_period
        CHECK (end_date IS NULL OR end_date >= start_date),
    CONSTRAINT chk_member_contracts_trial
        CHECK (
            trial_end_date IS NULL
            OR (trial_end_date >= start_date AND (end_date IS NULL OR trial_end_date <= end_date))
        ),
    CONSTRAINT excl_member_contracts_no_overlap
        EXCLUDE USING gist (member_id WITH =, daterange(start_date, end_date, '[]') WITH &&)
);

CREATE INDEX idx_member_contracts_org_id ON member_contracts(org_id);

CREATE TABLE member_documents (
    id          UUID        PRIMARY KEY,
    org_id      UUID        NOT NULL REFERENCES organizations(id),
    member_id   UUID        NOT NULL,
    name        TEXT        NOT NULL,
    kind        TEXT        NOT NULL,
    file_key    TEXT        NOT NULL,
    added_at    TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT fk_member_documents_member
        FOREIGN KEY (member_id, org_id) REFERENCES organization_members(id, organization_id),
    CONSTRAINT chk_member_documents_kind
        CHECK (kind IN ('CONTRACT', 'IDENTITY', 'DIPLOMA', 'MEDICAL', 'OTHER')),
    CONSTRAINT chk_member_documents_name CHECK (btrim(name) <> ''),
    CONSTRAINT chk_member_documents_file_key CHECK (btrim(file_key) <> '')
);

CREATE INDEX idx_member_documents_org_id ON member_documents(org_id);
CREATE INDEX idx_member_documents_member ON member_documents(member_id);
